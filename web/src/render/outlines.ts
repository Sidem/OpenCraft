// Outlines of a belt line being dragged out (the engine's `line_cells`): a flat box under each cell, cyan
// where a belt will go and red past the belts in hand, drawn through terrain so the whole path shows.
// Uses the renderer's line program (shaders.ts `lineVert`, scaled by `u_scale`) and its box-edge VAO.

import type { Uniforms } from './gl';

type LineProgram = { prog: WebGLProgram; u: Uniforms<'u_viewProj' | 'u_offset' | 'u_scale' | 'u_color'> };

/** Draws the outlines of `cells` (x, y, z, will-be-built per cell) relative to `eye`; returns the draw calls. */
export function drawLineCells(
  gl: WebGL2RenderingContext,
  line: LineProgram,
  edges: WebGLVertexArrayObject,
  viewProj: Float32Array,
  cells: Int32Array,
  eye: [number, number, number],
): number {
  if (cells.length === 0) return 0;
  gl.enable(gl.BLEND);
  gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
  gl.disable(gl.DEPTH_TEST);
  gl.useProgram(line.prog);
  gl.uniformMatrix4fv(line.u.u_viewProj, false, viewProj);
  gl.uniform3f(line.u.u_scale, 1, 0.25, 1);
  gl.bindVertexArray(edges);
  for (let i = 0; i < cells.length; i += 4) {
    const ok = cells[i + 3] === 1;
    gl.uniform4f(line.u.u_color, ok ? 0.3 : 1, ok ? 0.9 : 0.3, ok ? 1 : 0.25, 0.9);
    gl.uniform3f(line.u.u_offset, cells[i] - eye[0], cells[i + 1] - eye[1], cells[i + 2] - eye[2]);
    gl.drawArrays(gl.LINES, 0, 24);
  }
  gl.bindVertexArray(null);
  gl.enable(gl.DEPTH_TEST);
  gl.disable(gl.BLEND);
  return cells.length / 4;
}