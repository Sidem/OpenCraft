// The lights players hold (a torch in a hand): the engine reports them each frame (`game.held_light_ptr()`, camera-relative
// x, y, z and reach in blocks per lit player) and the lit programs (terrain, boxes, water) add them in the fragment
// shader (`HELD_LIGHT` in shaders.ts). One shared array, set once a frame by the renderer and uploaded by each program.

import { HELD_LIGHT_SLOTS } from './shaders';

const lights = new Float32Array(HELD_LIGHT_SLOTS * 4);

/** Takes this frame's lights (a view into wasm memory is fine: it is copied); slots past the end are off. */
export function setHeldLights(src: Float32Array): void {
  lights.fill(0);
  lights.set(src.subarray(0, lights.length));
}

/** Uploads the lights to a program's `u_held`. */
export function uploadHeldLights(gl: WebGL2RenderingContext, loc: WebGLUniformLocation | null): void {
  gl.uniform4fv(loc, lights);
}
