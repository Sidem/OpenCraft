// What the renderer is handed each frame (`FrameInfo`, filled by main.ts from the engine) and what it reports back
// (`RenderStats`). A new per-frame input is a field here, read in renderer.ts or the pass that draws it; `boxViews`
// reads the box instance lists (views into wasm memory, valid for this frame only).

import type { Game } from '../wasm/engine.js';
import type { Weather } from './sky/weather';

/** The box lists of this frame from `game`: the world's, the ghost's and the port marks', each with its round bodies. */
export function boxViews(game: Game, memory: WebAssembly.Memory, floats: number) {
  const view = (ptr: number, count: number) => new Float32Array(memory.buffer, ptr, count * floats);
  return {
    boxes: view(game.instance_ptr(), game.instance_count()),
    boxCount: game.instance_count(),
    round: view(game.round_ptr(), game.round_count()),
    roundCount: game.round_count(),
    ghost: view(game.ghost_ptr(), game.ghost_count()),
    ghostCount: game.ghost_count(),
    ghostRound: view(game.ghost_round_ptr(), game.ghost_round_count()),
    ghostRoundCount: game.ghost_round_count(),
    ghostBlocked: game.ghost_blocked(),
    marks: view(game.marks_ptr(), game.marks_count()),
    marksCount: game.marks_count(),
    marksRound: view(game.marks_round_ptr(), game.marks_round_count()),
    marksRoundCount: game.marks_round_count(),
    xray: view(game.xray_ptr(), game.xray_count()),
    xrayCount: game.xray_count(),
    xrayRound: view(game.xray_round_ptr(), game.xray_round_count()),
    xrayRoundCount: game.xray_round_count(),
    xrayRadius: game.xray_radius(),
    xrayFade: game.xray_fade_from(),
    beltScroll: [game.belt_scroll(0), game.belt_scroll(1), game.belt_scroll(2), game.belt_scroll(3)] as [number, number, number, number],
  };
}

/** Water's constants from the engine, read once: its texture layer (water.ts). */
export interface WaterConstants {
  layer: number;
}

export function waterConstants(game: Game): WaterConstants {
  return { layer: game.water_layer() };
}

export interface FrameInfo {
  eye: [number, number, number];
  yaw: number;
  pitch: number;
  target: [number, number, number] | null;
  /** Where the target's model sits in its cell (lowest corner, size) so the outline hugs a pole or belt; empty: whole cell. */
  targetFit: number[];
  /** The selected belt item's box in the world (lowest corner, then size), or empty. */
  selectedItem: number[];
  mineProgress: number;
  /** The engine's time of day (0 midnight, 0.5 noon). */
  time: number;
  /** Days since the world began as a fraction (0 at the first midnight); moves the stars and the moon's phase. */
  days: number;
  /** The engine's weather at the camera (`game.weather_here()`, sky/weather.ts), before the ease. */
  weather: Weather;
  /** Downwind push vector and speed at the camera: `[dirX, dirZ, speedMs]` from `game.weather_here()`. */
  wind: [number, number, number];
  /** Seasonal foliage (`game.season_foliage()`, 120 floats) and ground (`game.season_ground()`, 12 floats) uniforms. */
  seasonFoliage: Float32Array;
  seasonGround: Float32Array;
  /** Box instances (dropped items, belt items, machine parts), `INSTANCE_FLOATS` each. */
  boxes: Float32Array;
  boxCount: number;
  /** The round bodies among them, copied out (`round_ptr`), drawn with the cylinder mesh. */
  round: Float32Array;
  roundCount: number;
  /** See-through boxes (a held machine's ghost, belt stubs at ports), their round bodies, and whether the ghost is in the way. */
  ghost: Float32Array;
  ghostCount: number;
  ghostRound: Float32Array;
  ghostRoundCount: number;
  ghostBlocked: boolean;
  /** Port marks (arrows, riding items) and their round bodies: solid, drawn after the ghost so they show through it. */
  marks: Float32Array;
  marksCount: number;
  marksRound: Float32Array;
  marksRoundCount: number;
  /** The X-ray goggles' view (engine `xray.rs`): the factory's boxes near the eye, empty unless worn and on; drawn behind the terrain only. */
  xray: Float32Array;
  xrayCount: number;
  xrayRound: Float32Array;
  xrayRoundCount: number;
  /** Blocks the view reaches, and the share of it where it starts to fade. */
  xrayRadius: number;
  xrayFade: number;
  /** Fractional UV scroll for belt tiers Mk1..Mk4 at this frame's render time. */
  beltScroll: [number, number, number, number];
  /** A belt line being dragged out: x, y, z and 1 (will be built) or 0 (past the belts in hand) per cell. */
  lineCells: Int32Array;
  /** Quarry boxes to outline, as lowest and highest cells (6 numbers each). */
  quarryBoxes: Int32Array;
  /** The felling mark on the tree under the crosshair (engine `chop_mark_at`): x, y, z, or empty. */
  chopMark: ArrayLike<number>;
  /** Blocks quarries are digging: x, y, z and progress in thousandths each. */
  cracks: Int32Array;
  underwater: boolean; // the engine's `eye_in_water`
  /** Satellites in orbit as the sky draws them (engine `sky_satellites`: x, y, z, kind each). */
  satellites: Float32Array;
  /** Lights held in hands this frame: camera-relative x, y, z and reach in blocks each (held-light.ts). */
  held: Float32Array;
  /** Halos round light-giving blocks (engine `halos.rs`): `haloFloats` floats each, `haloCount` of them. */
  halos: Float32Array;
  haloCount: number;
  haloFloats: number;
  /** Fire emitters (engine `fire_view.rs`): `fireFloats` floats each, `fireCount` of them, plus render time for animation. */
  fire: Float32Array;
  fireCount: number;
  fireFloats: number;
  /** Smoke emitters (engine `factory/smoke.rs`): `smokeFloats` u32 words each (7 f32 + 1 u32 stream mask), `smokeCount` of them. */
  smoke: Uint32Array;
  smokeCount: number;
  smokeFloats: number;
  /** Engine-synchronized plume phase matching each emitter's 32-bit stream history mask. */
  smokePhase: number;
  renderTime: number;
}

export interface RenderStats {
  meshes: number;
  visible: number;
  drawCalls: number;
  quads: number;
}
