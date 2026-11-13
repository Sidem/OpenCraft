// Halos round the blocks that give light (engine `halos.rs`: lamps, panel lights, torches): one camera-facing quad
// each, drawn after the world with additive blending and no depth writes, a soft round glow in the light's colour.
// Strong at night, faint by day (`sky.day`), faded by the fog. The quad is pulled towards the camera so the lamp's own
// faces don't cut it in half, but not so far that it shows through a wall in front of the lamp.
// Instance layout (`game.halo_floats()`): camera-relative centre xyz, radius, colour rgb, strength.

import { createProgram, uniforms } from './gl';
import type { Sky } from './sky';

const vert = /* glsl */ `#version 300 es
precision highp float;
layout(location = 0) in vec2 a_corner;  // -1..1
layout(location = 1) in vec4 a_i0;      // centre (camera-relative), radius
layout(location = 2) in vec4 a_i1;      // colour, strength
uniform mat4 u_view;
uniform mat4 u_proj;
out vec2 v_corner;
out vec4 v_colour;
out float v_dist;
void main() {
  vec4 at = u_view * vec4(a_i0.xyz, 1.0);
  float dist = length(at.xyz);
  // Towards the camera by most of a block, never past it.
  float pull = min(0.9, max(dist - 0.3, 0.0));
  at.xyz *= (dist - pull) / max(dist, 1e-4);
  at.xy += a_corner * a_i0.w;
  v_corner = a_corner;
  v_colour = a_i1;
  v_dist = dist;
  gl_Position = u_proj * at;
}`;

const frag = /* glsl */ `#version 300 es
precision highp float;
in vec2 v_corner;
in vec4 v_colour;
in float v_dist;
uniform float u_level;  // how strong halos are now (night 1, day low)
uniform vec2 u_fog;     // fog start, end
out vec4 o_color;
void main() {
  float r2 = dot(v_corner, v_corner);
  if (r2 > 1.0) discard;
  // A bright middle that falls off softly to nothing at the rim.
  float glow = exp(-r2 * 4.5) * (1.0 - r2) + exp(-r2 * 26.0) * 0.6;
  float fade = 1.0 - smoothstep(u_fog.x, u_fog.y, v_dist);
  o_color = vec4(v_colour.rgb * glow * v_colour.a * u_level * fade, 1.0);
}`;

/** How strong halos are by day and at night. */
const DAY_LEVEL = 0.22;
const NIGHT_LEVEL = 0.85;

export class HaloPipeline {
  private readonly prog: WebGLProgram;
  private readonly u;
  private readonly vao: WebGLVertexArrayObject;
  private readonly instances: WebGLBuffer;
  private capacity = 0;

  constructor(private readonly gl: WebGL2RenderingContext) {
    this.prog = createProgram(gl, vert, frag);
    this.u = uniforms(gl, this.prog, ['u_view', 'u_proj', 'u_level', 'u_fog'] as const);
    this.vao = gl.createVertexArray()!;
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer()!);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 8, 0);
    this.instances = gl.createBuffer()!;
    for (const loc of [1, 2]) {
      gl.enableVertexAttribArray(loc);
      gl.vertexAttribDivisor(loc, 1);
    }
    gl.bindVertexArray(null);
  }

  /** Draws `count` halos of `floats` floats each from `halos` (a view into wasm memory). Returns the draw calls made. */
  draw(view: Float32Array, proj: Float32Array, sky: Sky, fog: [number, number], halos: Float32Array, count: number, floats: number): number {
    if (count === 0) return 0;
    const gl = this.gl;
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.instances);
    if (count * floats > this.capacity) {
      this.capacity = Math.max(count * floats, this.capacity * 2, 512);
      gl.bufferData(gl.ARRAY_BUFFER, this.capacity * 4, gl.DYNAMIC_DRAW);
    }
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, halos.subarray(0, count * floats));
    gl.vertexAttribPointer(1, 4, gl.FLOAT, false, floats * 4, 0);
    gl.vertexAttribPointer(2, 4, gl.FLOAT, false, floats * 4, 16);
    gl.useProgram(this.prog);
    gl.uniformMatrix4fv(this.u.u_view, false, view);
    gl.uniformMatrix4fv(this.u.u_proj, false, proj);
    gl.uniform1f(this.u.u_level, DAY_LEVEL + (NIGHT_LEVEL - DAY_LEVEL) * (1 - sky.day));
    gl.uniform2f(this.u.u_fog, fog[0], fog[1]);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.ONE, gl.ONE);
    gl.depthMask(false);
    gl.disable(gl.CULL_FACE);
    gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, count);
    gl.bindVertexArray(null);
    gl.depthMask(true);
    gl.enable(gl.CULL_FACE);
    gl.disable(gl.BLEND);
    return 1;
  }
}
