// Repeatable art fixture using the public wasm API and production renderer. No game saves.
// Visit /art-preview.html or ?view=materials. All layout edits use public engine actions.
import init, { Game } from '../src/wasm/engine.js';
import { Renderer } from '../src/render/renderer.ts';
import { INSTANCE_FLOATS } from '../src/render/boxes.ts';
import { Hud } from '../src/ui/hud.ts';
import { worldMaterialSamples } from './materials.js';

const view = new URLSearchParams(location.search).get('view');
const materials = view === 'materials';
const items = view === 'items';
const machines = view === 'machines';
document.body.classList.toggle('materials', materials || items);
document.querySelector('#title').textContent = items ? 'Alpine · manufactured parts' : machines ? 'Alpine · machine yard' : 'Alpine · material study';
document.querySelector('#description').textContent = items
  ? 'The inventory icons and loose items share the same box assemblies from Rust.'
  : machines ? 'A closer production-renderer view of the extractor, furnace, press, turbine, lab, routes and storage.'
  : 'Three-by-three block patches through the production terrain shader: continuous mineral grain and world-scale variation.';
const status=document.querySelector('#status');
try {
  const wasm=await init();
  const game=new Game(2024,3);
  game.toggle_fly();
  const tick=(n=2)=>{ for(let i=0;i<n;i++) game.update(1/60); };
  const stream=()=>{ game.begin_work(); while(game.work_step()); };
  tick(); stream(); tick();
  const ground=(x,z)=>{ for(let y=100;y>0;y--) if(game.block_at(x,y,z)!==0) return y; return 0; };
  // Small raised field station in a clearing; stable coordinates independent of the player's save.
  const x0=20,z0=8;
  let base=0;
  for(let x=x0;x<x0+8;x++) for(let z=z0;z<z0+6;z++) base=Math.max(base,ground(x,z));
  const ids=Object.fromEntries(Array.from({length:game.block_count()},(_,id)=>[game.block_name(id),id]));
  let equipped=0;
  const equip=(name)=>{
    const id=ids[name]; game.give(id,name==='Stone'?200:name==='Grass'?64:2); tick();
    let slot=Array.from({length:game.inventory_size()},(_,i)=>i).find(i=>game.slot_item(i)===id);
    if(slot===undefined) throw new Error(`Fixture inventory cannot equip ${name}`);
    if(slot>=game.hotbar_size()) { game.click_slot(slot,false); tick(); game.click_slot(0,false); tick(); game.close_inventory(); tick(); slot=0; }
    game.select_slot(slot); tick();
    equipped=id;
  };
  const place=(x,y,z)=>{
    if(!game.slot_count(game.selected_slot())) throw new Error('Fixture inventory exhausted');
    game.teleport(x+.5,y+2,z+.5); game.set_look(0,-1.55); tick();
    game.set_using(true); tick(); game.set_using(false); tick();
    if(game.block_at(x,y,z)!==equipped) throw new Error(`Placement failed at ${x},${y},${z}; expected ${equipped}, got ${game.block_at(x,y,z)}; slot ${game.selected_slot()} item ${game.slot_item(game.selected_slot())} count ${game.slot_count(game.selected_slot())}`);
  };
  equip('Stone');
  for(let x=x0;x<x0+8;x++) for(let z=z0;z<z0+6;z++) {
    for(let y=ground(x,z)+1;y<=base;y++) place(x,y,z);
  }
  equip('Grass');
  for(let x=x0;x<x0+8;x++) for(let z=z0;z<z0+6;z++) place(x,base+1,z);
  equip('Stone');
  for(let x=x0;x<x0+3;x++) for(let y=base+2;y<=base+3;y++) place(x,y,z0);
  for(const [name,x,z] of [['Miner Mk1',4,2],['Conveyor Belt',4,1],['Storage Box',4,0],['Smelter',6,2],['Constructor',6,4],['Coal Generator',2,4],['Power Pole',1,2]]) {
    equip(name); place(x0+x,base+2,z0+z);
  }
  if (machines) {
    for(const [name,x,z] of [['Research Lab',3,4],['Filter',5,4],['Splitter',5,3],
      ['Belt Ramp Up',4,4],['Belt Lift',7,4],['Underpass Entry',7,1]]) {
      equip(name); place(x0+x,base+2,z0+z);
    }
  }
  game.teleport(x0+(machines?8:10),base+(machines?5:7),z0+(machines?8:11));
  game.set_look(-.65,-.55); tick(); stream(); tick();
  const eye=[game.eye_x(),game.eye_y(),game.eye_z()];
  const canvas=document.querySelector('#world');
  const params = new URLSearchParams(location.search);
  const gl = canvas.getContext('webgl2', { antialias: true, alpha: false, depth: true, stencil: false });
  // Development-only A/B measurement of the terrain shader on identical geometry.
  if (params.get('tint') === 'off') {
    const source = gl.shaderSource.bind(gl);
    gl.shaderSource = (shader, code) => source(shader, code.replace('#define TERRAIN', '// tint disabled for measurement'));
  }
  const renderer=new Renderer(canvas,3);
  const tex=new Uint8Array(wasm.memory.buffer,game.texture_ptr(),game.texture_byte_len()).slice();
  renderer.setTextures(tex,game.texture_size(),game.texture_layers());
  for(let kind;(kind=game.next_event())!==0;) {
    const x=game.event_x(),y=game.event_y(),z=game.event_z();
    if(kind===1) renderer.upsertChunk(x,y,z,new Uint32Array(wasm.memory.buffer,game.mesh_ptr(),game.mesh_len()),game.mesh_opaque_quads(),game.mesh_cutout_quads());
    else renderer.removeChunk(x,y,z);
  }
  const samples = [
    ['Stone','Stone',0], ['Grass','Grass',0], ['Dirt','Dirt',0], ['Grass edge','Grass',1],
    ['Sand','Sand',0], ['Bark','Log',1], ['End grain','Log',0], ['Leaves','Leaves',0],
    ['Bedrock','Bedrock',0], ['Spent rock','Spent Rock',0],
    ['Coal seam','Coal Ore',0], ['Iron bloom','Iron Ore',0], ['Copper vein','Copper Ore',0],
  ];
  const size = game.texture_size(), layerBytes = size * size * 4;
  const rawMaterials = new URLSearchParams(location.search).has('raw');
  const worldSamples = materials && !rawMaterials ? worldMaterialSamples(
    tex,size,game.texture_layers(),samples.map(([,block,face])=>game.item_icon(ids[block])[face])) : null;
  for(const [name,block,face] of materials ? samples : items ? [] : samples.slice(0,2)) {
    const layer = game.item_icon(ids[block])[face];
    const figure=document.createElement('figure'), swatch=document.createElement('canvas'), caption=document.createElement('figcaption');
    const repeats = materials ? 3 : 1;
    const sample = worldSamples?.get(layer);
    swatch.width=swatch.height=sample?.width ?? size*repeats;
    if (sample) swatch.getContext('2d').putImageData(sample,0,0);
    else {
      const tile = new ImageData(new Uint8ClampedArray(tex.slice(layer*layerBytes,(layer+1)*layerBytes)),size,size);
      for(let y=0;y<repeats;y++) for(let x=0;x<repeats;x++) swatch.getContext('2d').putImageData(tile,x*size,y*size);
    }
    caption.textContent=`${name} · ${sample?'3 × 3 world blocks':`${size} × ${size} base tile`}`;
    figure.append(swatch,caption); document.querySelector('#swatches').append(figure);
  }
  if (items) {
    const layers = Array.from({length:game.texture_layers()},(_,layer) => {
      const canvas=document.createElement('canvas'); canvas.width=canvas.height=size;
      canvas.getContext('2d').putImageData(new ImageData(
        new Uint8ClampedArray(tex.slice(layer*layerBytes,(layer+1)*layerBytes)),size,size),0,0);
      return canvas;
    });
    const iconHud = {game,icons:new Map(),layers};
    for(let id=256;id<270;id++) {
      const name=game.item_name(id); if(!name || name.includes('Pickaxe') || name.includes('Axe') || name.includes('Shovel')) continue;
      const figure=document.createElement('figure'),caption=document.createElement('figcaption');
      figure.append(Hud.prototype.itemIcon.call(iconHud,id));
      caption.textContent=name; figure.append(caption); document.querySelector('#swatches').append(figure);
    }
  }
  let boxes=new Float32Array(wasm.memory.buffer,game.instance_ptr(),game.instance_count()*INSTANCE_FLOATS).slice();
  if (machines) {
    // A pure presentation toggle for silhouette review: omit the many thin power-wire segments.
    const solids=[];
    for(let i=0;i<boxes.length;i+=INSTANCE_FLOATS) {
      if(boxes[i+4]<0.04 && boxes[i+5]<0.04) continue;
      for(let j=0;j<INSTANCE_FLOATS;j++) solids.push(boxes[i+j]);
    }
    boxes=new Float32Array(solids);
  }
  const frame={eye,yaw:game.yaw(),pitch:game.pitch(),target:null,mineProgress:0,boxes,boxCount:boxes.length/INSTANCE_FLOATS};
  const render=()=>{ renderer.render(frame); requestAnimationFrame(render); }; render();
  status.textContent=`Seed 2024 · fixed camera ${eye.map(n=>n.toFixed(1)).join(', ')} · ${game.miners()} miner · ${game.belts()} belt · ${frame.boxCount} boxes · production materials`;
  if (params.has('benchmark')) {
    // Synthetic factory stress: 64 copies of the scene's machine instances, GPU-completed frames.
    const stress = new Float32Array(boxes.length * 64);
    for (let copy = 0; copy < 64; copy++) {
      stress.set(boxes, copy * boxes.length);
      for (let i = copy * boxes.length; i < (copy + 1) * boxes.length; i += INSTANCE_FLOATS) {
        stress[i] += (copy % 8 - 3) * 8;
        stress[i + 2] += (Math.floor(copy / 8) - 3) * 8;
      }
    }
    const timings = [];
    for (let i = 0; i < 35; i++) {
      const start = performance.now();
      renderer.render({ ...frame, boxes: stress, boxCount: frame.boxCount * 64 });
      gl.finish();
      if (i >= 5) timings.push(performance.now() - start);
    }
    timings.sort((a, b) => a - b);
    status.textContent += ` · stress ${frame.boxCount * 64} boxes: median ${timings[15].toFixed(2)} ms, p95 ${timings[28].toFixed(2)} ms · GL error ${gl.getError()} · tint ${params.get('tint') === 'off' ? 'off' : 'on'}`;
    renderer.render(frame);
  }
} catch(error) { status.textContent=String(error); console.error(error); }
