// Overlays drawn after the world (`Overlays`, owned by the renderer): mining cracks (the player's own and the blocks
// quarries are digging), the target's outline (hugging a pole or belt's model when the engine gives its fit), the
// outlines of a belt line being dragged out (the engine's `line_cells`: a flat box under each cell, cyan where a belt
// will go, the kit's tier colour where one will be upgraded, red past what the hand holds), and placement boxes (amber:
// the one a held quarry would dig or the one whose panel is open, a held multi-block machine's footprint with red cells
// in the way), drawn through terrain so they show whole. One line program (`lineVert`, scaled by `u_scale`) draws
// every outline with the box-edge VAO; the crack program draws on the cube VAO. To add an overlay: a function here,
// called from `Overlays.draw` in the order it should layer.

import type { FrameInfo } from './renderer';
import { boxEdges, cubeVertices } from './geometry';
import { createProgram, uniforms, type Uniforms } from './gl';

type Overlaid = Pick<
  FrameInfo,
  'cracks' | 'target' | 'targetFit' | 'selectedItem' | 'mineProgress' | 'lineCells' | 'quarryBoxes'
>;
/** How far the selected item's outline stands off its box, so it clears the item. */
const SELECT_PAD = 0.02;

export class Overlays {
  private readonly line: LineProgram;
  private readonly crack: CrackProgram;
  private readonly cubeVao: WebGLVertexArrayObject;
  private readonly lineVao: WebGLVertexArrayObject;

  constructor(private readonly gl: WebGL2RenderingContext) {
    const lineProg = createProgram(gl, lineVert, lineFrag);
    const crackProg = createProgram(gl, crackVert, crackFrag);
    this.line = { prog: lineProg, u: uniforms(gl, lineProg, ['u_viewProj', 'u_offset', 'u_scale', 'u_color'] as const) };
    this.crack = { prog: crackProg, u: uniforms(gl, crackProg, ['u_viewProj', 'u_offset', 'u_progress'] as const) };

    // Cube geometry for the mining-crack overlay.
    const cube = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, cube);
    gl.bufferData(gl.ARRAY_BUFFER, cubeVertices(), gl.STATIC_DRAW);
    this.cubeVao = gl.createVertexArray()!;
    gl.bindVertexArray(this.cubeVao);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 28, 0);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 4, gl.FLOAT, false, 28, 12);

    this.lineVao = gl.createVertexArray()!;
    gl.bindVertexArray(this.lineVao);
    const lines = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, lines);
    gl.bufferData(gl.ARRAY_BUFFER, boxEdges(), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
    gl.bindVertexArray(null);
  }

  /** Draws every overlay of the frame relative to `eye`; returns the draw calls. */
  draw(viewProj: Float32Array, f: Overlaid, eye: Eye): number {
    const gl = this.gl;
    let drawCalls = drawCracks(gl, this.crack, this.cubeVao, viewProj, f.cracks, eye);
    if (f.target) {
      const [tx, ty, tz] = f.target;
      const ox = tx - eye[0], oy = ty - eye[1], oz = tz - eye[2];
      if (f.mineProgress > 0) {
        const own = new Int32Array([tx, ty, tz, Math.round(f.mineProgress * 1000)]);
        drawCalls += drawCracks(gl, this.crack, this.cubeVao, viewProj, own, eye);
      }
      gl.enable(gl.BLEND);
      gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
      gl.depthMask(false);
      gl.useProgram(this.line.prog);
      gl.uniformMatrix4fv(this.line.u.u_viewProj, false, viewProj);
      const fit = f.targetFit;
      if (fit.length === 6) {
        gl.uniform3f(this.line.u.u_offset, ox + fit[0], oy + fit[1], oz + fit[2]);
        gl.uniform3f(this.line.u.u_scale, fit[3], fit[4], fit[5]);
      } else {
        gl.uniform3f(this.line.u.u_offset, ox, oy, oz);
        gl.uniform3f(this.line.u.u_scale, 1, 1, 1);
      }
      gl.uniform4f(this.line.u.u_color, 0.05, 0.05, 0.08, 0.6);
      gl.bindVertexArray(this.lineVao);
      gl.drawArrays(gl.LINES, 0, 24);
      gl.depthMask(true);
      gl.disable(gl.BLEND);
      drawCalls++;
    }
    const item = f.selectedItem;
    if (item.length === 6) {
      beginOverlay(gl, this.line, this.lineVao, viewProj);
      gl.uniform4f(this.line.u.u_color, 1, 0.68, 0.18, 0.95);
      gl.uniform3f(this.line.u.u_offset, item[0] - eye[0] - SELECT_PAD, item[1] - eye[1] - SELECT_PAD, item[2] - eye[2] - SELECT_PAD);
      gl.uniform3f(this.line.u.u_scale, item[3] + 2 * SELECT_PAD, item[4] + 2 * SELECT_PAD, item[5] + 2 * SELECT_PAD);
      gl.drawArrays(gl.LINES, 0, 24);
      endOverlay(gl);
      drawCalls++;
    }
    drawCalls += drawLineCells(gl, this.line, this.lineVao, viewProj, f.lineCells, eye);
    drawCalls += drawBoxes(gl, this.line, this.lineVao, viewProj, f.quarryBoxes, eye);
    return drawCalls;
  }
}

type LineProgram = { prog: WebGLProgram; u: Uniforms<'u_viewProj' | 'u_offset' | 'u_scale' | 'u_color'> };
type CrackProgram = { prog: WebGLProgram; u: Uniforms<'u_viewProj' | 'u_offset' | 'u_progress'> };
type Eye = [number, number, number];

/** Draws the outlines of `cells` (x, y, z, then 0 short, 1 built or 0xRRGGBB upgraded) relative to `eye`; returns the draw calls. */
function drawLineCells(
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

/** Draws outlines of boxes given as lowest and highest cells and a colour (x0, y0, z0, x1, y1, z1, 0xRRGGBB
 * each; 0 is amber). */
function drawBoxes(
  gl: WebGL2RenderingContext,
  line: LineProgram,
  edges: WebGLVertexArrayObject,
  viewProj: Float32Array,
  boxes: Int32Array,
  eye: Eye,
): number {
  if (boxes.length === 0) return 0;
  beginOverlay(gl, line, edges, viewProj);
  for (let i = 0; i + 6 < boxes.length; i += 7) {
    const [x0, y0, z0, x1, y1, z1, c] = boxes.subarray(i, i + 7);
    if (c > 0) gl.uniform4f(line.u.u_color, (c >> 16) / 255, ((c >> 8) & 255) / 255, (c & 255) / 255, 0.95);
    else gl.uniform4f(line.u.u_color, 1, 0.68, 0.18, 0.95);
    gl.uniform3f(line.u.u_offset, x0 - eye[0], y0 - eye[1], z0 - eye[2]);
    gl.uniform3f(line.u.u_scale, x1 - x0 + 1, y1 - y0 + 1, z1 - z0 + 1);
    gl.drawArrays(gl.LINES, 0, 24);
  }
  endOverlay(gl);
  return boxes.length / 7;
}

/** Draws the mining crack on blocks given as x, y, z and progress in thousandths each. */
function drawCracks(
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

const lineVert = /* glsl */ `#version 300 es
precision highp float;
layout(location = 0) in vec3 a_pos;
uniform mat4 u_viewProj;
uniform vec3 u_offset;
uniform vec3 u_scale;
void main() {
  gl_Position = u_viewProj * vec4(u_offset + a_pos * u_scale * 1.004 - 0.002, 1.0);
}
`;

const lineFrag = /* glsl */ `#version 300 es
precision mediump float;
uniform vec4 u_color;
out vec4 o_color;
void main() { o_color = u_color; }
`;

const crackVert = /* glsl */ `#version 300 es
precision highp float;
layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec4 a_face;
uniform mat4 u_viewProj;
uniform vec3 u_offset;
out vec2 v_uv;
void main() {
  v_uv = a_face.xy;
  gl_Position = u_viewProj * vec4(u_offset + 0.5 + a_pos * 1.002, 1.0);
}
`;

const crackFrag = /* glsl */ `#version 300 es
precision highp float;
uniform float u_progress;
in vec2 v_uv;
out vec4 o_color;
float hash(vec2 p) { return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453); }
void main() {
  // Pixel-art cracks that spread outward from the centre as mining progresses.
  vec2 cell = floor(fract(v_uv) * 16.0);
  float d = length(cell - 7.5) / 10.6;
  if (hash(cell) * 0.55 + d * 0.65 > u_progress * 1.3) discard;
  o_color = vec4(0.0, 0.0, 0.0, 0.55);
}
`;
