// What the renderer is handed each frame (`FrameInfo`, filled by main.ts from the engine) and what it reports back
// (`RenderStats`). A new per-frame input is a field here, read in renderer.ts or the pass that draws it.

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
  /** The world's seed and the biome the player stands in (engine order): what the weather is a function of (sky/weather.ts). */
  seed: number;
  biome: number;
  /** Box instances (dropped items, belt items, machine parts), `INSTANCE_FLOATS` each. */
  boxes: Float32Array;
  boxCount: number;
  /** The round bodies among them, copied out (`round_ptr`), drawn with the cylinder mesh. */
  round: Float32Array;
  roundCount: number;
  /** A belt line being dragged out: x, y, z and 1 (will be built) or 0 (past the belts in hand) per cell. */
  lineCells: Int32Array;
  /** Quarry boxes to outline, as lowest and highest cells (6 numbers each). */
  quarryBoxes: Int32Array;
  /** Blocks quarries are digging: x, y, z and progress in thousandths each. */
  cracks: Int32Array;
  underwater: boolean; // the engine's `eye_in_water`
  /** Satellites in orbit as the sky draws them (engine `sky_satellites`: x, y, z, kind each). */
  satellites: Float32Array;
  /** Lights held in hands this frame: camera-relative x, y, z and reach in blocks each (held-light.ts). */
  held: Float32Array;
}

export interface RenderStats {
  meshes: number;
  visible: number;
  drawCalls: number;
  quads: number;
}
