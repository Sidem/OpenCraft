// A blueprint seen from above, for its card in the library (blueprints.ts): each machine is its icon on its
// footprint, each belt or router a track with an arrow the way items go, higher pieces over lower ones. The
// numbers come from the engine (`blueprint_plan`: the box size, then `blueprint_plan_fields()` numbers per piece,
// lowest first). `layoutPlan` is pure and tested (web/test/blueprint-plan.test.mjs); `drawPlan` paints it.
// To draw a new kind of piece differently: a flag in the engine's plan row (`blueprint/plan.rs`), then a branch here.

export interface PlanPiece {
  x: number;
  y: number;
  w: number;
  h: number;
  item: number;
  /** 0 north (up), 1 east, 2 south, 3 west. */
  facing: number;
  /** A belt or router: its facing is the way items go. */
  flow: boolean;
}

export interface PlanLayout {
  /** Pixels per block column. */
  cell: number;
  /** Where the box's corner column lands, and the box's size in pixels. */
  ox: number;
  oy: number;
  width: number;
  height: number;
  pieces: PlanPiece[];
}

/** Room left around the box, in pixels. */
const PAD = 4;
/** A cell is never drawn bigger than this, so a tiny blueprint does not look like a giant one. */
const MAX_CELL = 18;

/** Fits the plan's box, seen from above (x right, z down), into `pw` × `ph` pixels. */
export function layoutPlan(plan: ArrayLike<number>, fields: number, pw: number, ph: number): PlanLayout {
  const [sx, sz] = [Math.max(1, plan[0] ?? 1), Math.max(1, plan[2] ?? 1)];
  const fit = Math.min(Math.floor((pw - 2 * PAD) / sx), Math.floor((ph - 2 * PAD) / sz));
  const cell = Math.max(1, Math.min(MAX_CELL, fit));
  const width = sx * cell;
  const height = sz * cell;
  const ox = Math.floor((pw - width) / 2);
  const oy = Math.floor((ph - height) / 2);
  const pieces: PlanPiece[] = [];
  for (let i = 3; i + fields <= plan.length; i += fields) {
    const [x0, z0, x1, z1, , item, facing, flow] = Array.from({ length: fields }, (_, k) => plan[i + k]);
    pieces.push({
      x: ox + x0 * cell,
      y: oy + z0 * cell,
      w: (x1 - x0 + 1) * cell,
      h: (z1 - z0 + 1) * cell,
      item,
      facing,
      flow: flow === 1,
    });
  }
  return { cell, ox, oy, width, height, pieces };
}

const GROUND = 'rgba(255, 255, 255, 0.05)';
const GRID = 'rgba(255, 255, 255, 0.06)';
const TRACK = '#2a313b';
const ARROW = '#fa9549';
const MACHINE = '#3b4552';
const EDGE = 'rgba(255, 255, 255, 0.22)';

/** Paints the plan on `canvas`, `pw` × `ph` CSS pixels (its pixel size is set here). */
export function drawPlan(
  canvas: HTMLCanvasElement,
  plan: ArrayLike<number>,
  fields: number,
  [pw, ph]: [number, number],
  icon: (item: number) => HTMLCanvasElement,
): void {
  const dpr = Math.max(1, Math.round(window.devicePixelRatio || 1));
  [canvas.width, canvas.height] = [pw * dpr, ph * dpr];
  [canvas.style.width, canvas.style.height] = [`${pw}px`, `${ph}px`];
  const ctx = canvas.getContext('2d')!;
  ctx.scale(dpr, dpr);
  const lay = layoutPlan(plan, fields, pw, ph);
  ctx.fillStyle = GROUND;
  ctx.fillRect(lay.ox, lay.oy, lay.width, lay.height);
  if (lay.cell >= 6) {
    ctx.strokeStyle = GRID;
    ctx.lineWidth = 1;
    ctx.beginPath();
    for (let x = lay.ox; x <= lay.ox + lay.width; x += lay.cell) ctx.rect(x + 0.5, lay.oy, 0, lay.height);
    for (let y = lay.oy; y <= lay.oy + lay.height; y += lay.cell) ctx.rect(lay.ox, y + 0.5, lay.width, 0);
    ctx.stroke();
  }
  const inset = lay.cell >= 6 ? 1 : 0;
  for (const p of lay.pieces) {
    const [x, y, w, h] = [p.x + inset, p.y + inset, p.w - 2 * inset, p.h - 2 * inset];
    ctx.fillStyle = p.flow ? TRACK : MACHINE;
    ctx.fillRect(x, y, w, h);
    if (p.flow) {
      arrow(ctx, x + w / 2, y + h / 2, Math.min(w, h) * 0.42, p.facing);
    } else {
      const s = Math.min(w, h);
      ctx.imageSmoothingQuality = 'high';
      ctx.drawImage(icon(p.item), x + (w - s) / 2, y + (h - s) / 2, s, s);
      if (inset) {
        ctx.strokeStyle = EDGE;
        ctx.strokeRect(x + 0.5, y + 0.5, w - 1, h - 1);
      }
    }
  }
}

/** A chevron centred on (cx, cy) pointing `facing`, `r` from centre to tip. */
function arrow(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number, facing: number): void {
  const angle = (facing * Math.PI) / 2; // 0 points up
  const point = (along: number, across: number): [number, number] => [
    cx + Math.sin(angle) * along + Math.cos(angle) * across,
    cy - Math.cos(angle) * along + Math.sin(angle) * across,
  ];
  ctx.fillStyle = ARROW;
  ctx.beginPath();
  ctx.moveTo(...point(r, 0));
  ctx.lineTo(...point(-r * 0.7, r * 0.8));
  ctx.lineTo(...point(-r * 0.25, 0));
  ctx.lineTo(...point(-r * 0.7, -r * 0.8));
  ctx.closePath();
  ctx.fill();
}
