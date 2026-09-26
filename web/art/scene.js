// Repeatable art fixture using the public wasm API and production renderer. No game saves.
// Visit /art-preview.html or ?view=materials. All layout edits use public engine actions.
import init, { Game } from '../src/wasm/engine.js';
import { Renderer } from '../src/render/renderer.ts';
import { INSTANCE_FLOATS } from '../src/render/boxes.ts';

const materials = new URLSearchParams(location.search).get('view') === 'materials';
document.body.classList.toggle('materials', materials);
document.querySelector('#title').textContent = 'Alpine · natural materials';
document.querySelector('#description').textContent = 'Blue slate, fern turf, cool earth and clustered foliage. Production Rust textures with world-anchored terrain tint.';
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
  const equip=(name)=>{
    const id=ids[name]; game.give(id,200); tick();
    let slot=Array.from({length:game.inventory_size()},(_,i)=>i).find(i=>game.slot_item(i)===id);
    if(slot>=game.hotbar_size()) { game.click_slot(slot,false); tick(); game.click_slot(0,false); tick(); game.close_inventory(); tick(); slot=0; }
    game.select_slot(slot); tick();
  };
  const place=(x,y,z)=>{
    if(!game.slot_count(game.selected_slot())) throw new Error('Fixture inventory exhausted');
    game.teleport(x+.5,y+2,z+.5); game.set_look(0,-1.55); tick();
    game.set_using(true); tick(); game.set_using(false); tick();
    if(game.block_at(x,y,z)===0) throw new Error(`Placement failed at ${x},${y},${z}; base ${base}; slot ${game.selected_slot()} item ${game.slot_item(game.selected_slot())} count ${game.slot_count(game.selected_slot())}; target ${game.target_x()},${game.target_y()},${game.target_z()}`);
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
  game.teleport(x0+10,base+7,z0+11); game.set_look(-.65,-.55); tick(); stream(); tick();
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
  ];
  const size = game.texture_size(), layerBytes = size * size * 4;
  for(const [name,block,face] of materials ? samples : samples.slice(0,2)) {
    const layer = game.item_icon(ids[block])[face];
    const figure=document.createElement('figure'), swatch=document.createElement('canvas'), caption=document.createElement('figcaption');
    const repeats = materials ? 3 : 1;
    swatch.width=swatch.height=size*repeats;
    const tile = new ImageData(new Uint8ClampedArray(tex.slice(layer*layerBytes,(layer+1)*layerBytes)),size,size);
    for(let y=0;y<repeats;y++) for(let x=0;x<repeats;x++) swatch.getContext('2d').putImageData(tile,x*size,y*size);
    caption.textContent=`${name} · ${size} × ${size}`; figure.append(swatch,caption); document.querySelector('#swatches').append(figure);
  }
  const boxes=new Float32Array(wasm.memory.buffer,game.instance_ptr(),game.instance_count()*INSTANCE_FLOATS).slice();
  const frame={eye,yaw:game.yaw(),pitch:game.pitch(),target:null,mineProgress:0,boxes,boxCount:game.instance_count()};
  const render=()=>{ renderer.render(frame); requestAnimationFrame(render); }; render();
  status.textContent=`Seed 2024 · fixed camera ${eye.map(n=>n.toFixed(1)).join(', ')} · ${game.miners()} miner · ${game.belts()} belt · ${game.instance_count()} boxes · production materials`;
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
