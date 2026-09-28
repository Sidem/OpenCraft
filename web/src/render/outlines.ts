// Overlays drawn after the world: mining cracks (the player's own and the blocks quarries are digging),
// the outlines of a belt line being dragged out (the engine's `line_cells`: a flat box under each cell,
// cyan where a belt will go, the kit's tier colour where one will be upgraded, red past what the hand holds), and quarry boxes (amber: the one a held
// quarry would dig, or the one whose panel is open), drawn through terrain so they show whole.
// Uses the renderer's line program (shaders.ts `lineVert`, scaled by `u_scale`) with its box-edge VAO,
// and its crack program with the cube VAO.

import type { Uniforms } from './gl';

type LineProgram = { prog: WebGLProgram; u: Uniforms<'u_viewProj' | 'u_offset' | 'u_scale' | 'u_color'> };
type CrackProgram = { prog: WebGLProgram; u: Uniforms<'u_viewProj' | 'u_offset' | 'u_progress'> };
type Eye = [number, number, number];

/** Draws the outlines of `cells` (x, y, z, then 0 short, 1 built or 0xRRGGBB upgraded) relative to `eye`; returns the draw calls. */
export function drawLineCells(
  gl: WebGL2RenderingContext,
  line: LineProgram,
  edges: WebGLVertexArrayObject,
  viewProj: Float32Array,
  cells: Int32Array,
  eye: Eye,
): number {
  if (cells.length === 0) return 0;
  beginOverlay(gl, line, edges, viewProj);
  gl.uniform3f(line.u.u_scale, 1, 0.25, 1);
  for (let i = 0; i < cells.length; i += 4) {
    const v = cells[i + 3];
    if (v > 1) gl.uniform4f(line.u.u_color, (v >> 16) / 255, ((v >> 8) & 255) / 255, (v & 255) / 255, 0.95);
    else gl.uniform4f(line.u.u_color, v ? 0.3 : 1, v ? 0.9 : 0.3, v ? 1 : 0.25, 0.9);
    gl.uniform3f(line.u.u_offset, cells[i] - eye[0], cells[i + 1] - eye[1], cells[i + 2] - eye[2]);
    gl.drawArrays(gl.LINES, 0, 24);
  }
  endOverlay(gl);
  return cells.length / 4;
}

/** Draws amber outlines of boxes given as lowest and highest cells (x0, y0, z0, x1, y1, z1 each). */
export function drawBoxes(
  gl: WebGL2RenderingContext,
  line: LineProgram,
  edges: WebGLVertexArrayObject,
  viewProj: Float32Array,
  boxes: Int32Array,
  eye: Eye,
): number {
  if (boxes.length === 0) return 0;
  beginOverlay(gl, line, edges, viewProj);
  gl.uniform4f(line.u.u_color, 1, 0.68, 0.18, 0.95);
  for (let i = 0; i + 5 < boxes.length; i += 6) {
    const [x0, y0, z0, x1, y1, z1] = boxes.subarray(i, i + 6);
    gl.uniform3f(line.u.u_offset, x0 - eye[0], y0 - eye[1], z0 - eye[2]);
    gl.uniform3f(line.u.u_scale, x1 - x0 + 1, y1 - y0 + 1, z1 - z0 + 1);
    gl.drawArrays(gl.LINES, 0, 24);
  }
  endOverlay(gl);
  return boxes.length / 6;
}

/** Draws the mining crack on blocks given as x, y, z and progress in thousandths each. */
export function drawCracks(
  gl: WebGL2RenderingContext,
  crack: CrackProgram,
  cube: WebGLVertexArrayObject,
  viewProj: Float32Array,
  cracks: Int32Array,
  eye: Eye,
): number {
  if (cracks.length === 0) return 0;
  gl.enable(gl.BLEND);
  gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
  gl.depthMask(false);
  gl.enable(gl.POLYGON_OFFSET_FILL);
  gl.polygonOffset(-1, -1);
  gl.useProgram(crack.prog);
  gl.uniformMatrix4fv(crack.u.u_viewProj, false, viewProj);
  gl.bindVertexArray(cube);
  for (let i = 0; i + 3 < cracks.length; i += 4) {
    gl.uniform3f(crack.u.u_offset, cracks[i] - eye[0], cracks[i + 1] - eye[1], cracks[i + 2] - eye[2]);
    gl.uniform1f(crack.u.u_progress, cracks[i + 3] / 1000);
    gl.drawArrays(gl.TRIANGLES, 0, 36);
  }
  gl.disable(gl.POLYGON_OFFSET_FILL);
  gl.depthMask(true);
  gl.disable(gl.BLEND);
  return cracks.length / 4;
}

function beginOverlay(gl: WebGL2RenderingContext, line: LineProgram, edges: WebGLVertexArrayObject, viewProj: Float32Array) {
  gl.enable(gl.BLEND);
  gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
  gl.disable(gl.DEPTH_TEST);
  gl.useProgram(line.prog);
  gl.uniformMatrix4fv(line.u.u_viewProj, false, viewProj);
  gl.bindVertexArray(edges);
}

function endOverlay(gl: WebGL2RenderingContext) {
  gl.bindVertexArray(null);
  gl.enable(gl.DEPTH_TEST);
  gl.disable(gl.BLEND);
}
