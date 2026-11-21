// Instanced fire emitters (engine `fire_view.rs`): burning trees (`kind 0`), localized ground embers (`kind 1`), and
// continuous blowtorch plumes (`kind 2.0..=3.0`, where `t = kind - 2.0` transitions from an electric-blue pressurized
// nozzle core into a billowing golden-orange flame head). One camera-facing quad per emitter drawn in a single
// instanced draw call with alpha blending and no depth writes, animated purely in GLSL from `u_time`.

import { createProgram, uniforms } from './gl';

const vert = /* glsl */ `#version 300 es
precision highp float;
layout(location = 0) in vec2 a_corner;  // x in -1..1, y in 0..1
layout(location = 1) in vec4 a_i0;      // base centre (camera-relative), width
layout(location = 2) in vec4 a_i1;      // height, seed, intensity, kind (0 blaze, 1 ember, 2..3 torch plume)
uniform mat4 u_view;
uniform mat4 u_proj;
out vec2 v_uv;
out vec3 v_meta;  // seed, intensity, kind
out float v_dist;
void main() {
  vec4 base = u_view * vec4(a_i0.xyz, 1.0);
  float dist = length(base.xyz);
  // Pull tree/ground flames slightly towards the eye so tongues wrap around bark and foliage cleanly;
  // leave torch plume slices right at their exact nozzle-to-target positions.
  if (a_i1.w < 1.5) {
    float pull = min(a_i0.w * 0.30, max(dist - 0.25, 0.0));
    base.xyz *= (dist - pull) / max(dist, 1e-4);
  }
  vec3 upV = a_i1.w >= 1.5 ? vec3(0.0, 1.0, 0.0) : normalize(vec3(u_view[1][0], u_view[1][1], u_view[1][2]));
  vec3 rightV = normalize(cross(vec3(0.0, 0.0, -1.0), upV));
  if (length(rightV) < 0.01) rightV = vec3(1.0, 0.0, 0.0);
  base.xyz += rightV * (a_corner.x * a_i0.w * 0.5) + upV * (a_corner.y * a_i1.x);
  v_uv = vec2(a_corner.x * 0.5 + 0.5, a_corner.y);
  v_meta = a_i1.yzw;
  v_dist = dist;
  gl_Position = u_proj * base;
}`;

const frag = /* glsl */ `#version 300 es
precision highp float;
in vec2 v_uv;
in vec3 v_meta;
in float v_dist;
uniform float u_time;
uniform vec2 u_fog;
out vec4 o_color;

float hash21(vec2 p) {
  p = fract(p * vec2(123.34, 456.21));
  p += dot(p, p + 45.32);
  return fract(p.x * p.y);
}

float vnoise(vec2 p) {
  vec2 i = floor(p), f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(hash21(i), hash21(i + vec2(1.0, 0.0)), f.x),
    mix(hash21(i + vec2(0.0, 1.0)), hash21(i + vec2(1.0, 1.0)), f.x),
    f.y
  );
}

float fbm(vec2 p) {
  return 0.55 * vnoise(p) + 0.3 * vnoise(p * 2.13 + 3.7) + 0.15 * vnoise(p * 4.31 - 1.9);
}

void main() {
  float seed = v_meta.x;
  float intensity = v_meta.y;
  float kind = v_meta.z;
  float fade = 1.0 - smoothstep(u_fog.x, u_fog.y, v_dist);
  vec2 uv = v_uv;
  float x = uv.x * 2.0 - 1.0;

  // Continuous blowtorch plume (kind in 2.0..=3.0): electric-blue nozzle core -> golden-orange roaring flame tip.
  if (kind >= 1.5) {
    float t = clamp(kind - 2.0, 0.0, 1.0);
    vec2 c = uv * 2.0 - 1.0;
    float r = length(c);
    if (r > 1.0) discard;
    float turb = mix(0.26, 0.52, t);
    float n = fbm(uv * mix(3.2, 4.6, t) + vec2(seed - u_time * 10.5, -u_time * 6.5));
    float body = smoothstep(0.96, 0.14, r + (n - 0.5) * turb);
    if (body < 0.02) discard;

    // Electric-blue pressurized combustion cone near the chrome bell (t = 0..0.44).
    vec3 blueOuter = vec3(0.14, 0.46, 1.0);
    vec3 blueCore = vec3(0.80, 0.95, 1.0);
    vec3 blueCol = mix(blueOuter, blueCore, smoothstep(0.18, 0.82, body));

    // Incandescent orange-gold billowing flame head (t = 0.36..1.0).
    vec3 orangeOuter = vec3(0.96, 0.28, 0.03);
    vec3 orangeMid = vec3(1.0, 0.66, 0.12);
    vec3 whiteHot = vec3(1.0, 0.97, 0.80);
    vec3 fireCol = mix(orangeOuter, orangeMid, smoothstep(0.06, 0.58, body));
    fireCol = mix(fireCol, whiteHot, smoothstep(0.55, 0.92, body) * (1.0 - t * 0.35));

    float orangeMix = smoothstep(0.18, 0.46, t + (n - 0.5) * 0.16);
    vec3 col = mix(blueCol, fireCol, orangeMix);
    float alpha = body * intensity * fade * mix(0.76, 0.64, t) * (1.0 - smoothstep(0.82, 1.0, t) * 0.40);
    o_color = vec4(col, clamp(alpha, 0.0, 1.0));
    return;
  }

  // Tree blaze (kind == 0) or localized weed-torch ground ember (kind == 1): rising flame tongues + ember sparks.
  float speed = kind > 0.5 ? 4.4 : 3.4;
  float freq = kind > 0.5 ? 5.2 : 4.2;
  float sway = sin(u_time * 3.6 + seed + uv.y * 5.5) * 0.14 * uv.y;
  float nx = x + sway;
  float warp = (fbm(vec2(nx * freq * 0.7 + seed, uv.y * freq * 0.8 - u_time * speed * 0.85)) - 0.5) * 0.55;
  float n = fbm(vec2((nx + warp * 0.45) * freq + seed, uv.y * freq - u_time * speed));
  // Rounded teardrop envelope: smooth base (no flat bottom edge) tapering into licking tongues above.
  float baseRound = sqrt(clamp(smoothstep(0.0, 0.24, uv.y), 0.0, 1.0));
  float width = mix(0.86, 0.12, pow(uv.y, 0.78)) * baseRound;
  float tongues = 0.18 * sin(nx * 9.5 + seed * 3.0 + uv.y * 4.0 - u_time * 6.0) * smoothstep(0.15, 0.8, uv.y);
  float shape = (1.0 - abs(nx) / max(width, 0.04)) + (n - 0.46) * 1.15 + tongues - uv.y * 0.48;
  float flame = smoothstep(0.08, 0.55, shape) * smoothstep(0.0, 0.12, uv.y) * smoothstep(1.0, 0.82, uv.y);
  float core = smoothstep(0.45, 0.92, shape) * (1.0 - uv.y * 0.55);

  // Drifting ember sparks in the upper region.
  vec2 sg = vec2((nx + sway * 0.8) * 6.5 + seed, uv.y * 8.5 - u_time * 3.0);
  vec2 cell = floor(sg), f = fract(sg) - 0.5;
  float h = hash21(cell + floor(seed * 7.0));
  float spark = h > 0.80 ? smoothstep(0.24, 0.05, length(f)) * smoothstep(0.18, 0.55, uv.y) * (1.0 - uv.y) : 0.0;

  float a = clamp(max(flame * 0.90, spark * 1.1) * intensity * fade, 0.0, 1.0);
  if (a < 0.02) discard;
  vec3 col = mix(vec3(0.78, 0.12, 0.02), vec3(1.0, 0.44, 0.06), smoothstep(0.05, 0.55, flame));
  col = mix(col, vec3(1.0, 0.82, 0.22), smoothstep(0.35, 0.85, flame));
  col = mix(col, vec3(1.0, 0.97, 0.78), core);
  col = mix(col, vec3(1.0, 0.78, 0.28), clamp(spark * 1.4, 0.0, 1.0));
  o_color = vec4(col, a);
}`;

export class FirePipeline {
  private readonly prog: WebGLProgram;
  private readonly u;
  private readonly vao: WebGLVertexArrayObject;
  private readonly instances: WebGLBuffer;
  private capacity = 0;

  constructor(private readonly gl: WebGL2RenderingContext) {
    this.prog = createProgram(gl, vert, frag);
    this.u = uniforms(gl, this.prog, ['u_view', 'u_proj', 'u_time', 'u_fog'] as const);
    this.vao = gl.createVertexArray()!;
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer()!);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, 0, 1, 0, -1, 1, 1, 1]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 8, 0);
    this.instances = gl.createBuffer()!;
    for (const loc of [1, 2]) {
      gl.enableVertexAttribArray(loc);
      gl.vertexAttribDivisor(loc, 1);
    }
    gl.bindVertexArray(null);
  }

  /** Draws `count` fire emitters (`floats` floats each) from `data` (a view into wasm memory). Returns draw calls. */
  draw(view: Float32Array, proj: Float32Array, time: number, fog: [number, number], data: Float32Array, count: number, floats: number): number {
    if (count === 0) return 0;
    const gl = this.gl;
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.instances);
    if (count * floats > this.capacity) {
      this.capacity = Math.max(count * floats, this.capacity * 2, 512);
      gl.bufferData(gl.ARRAY_BUFFER, this.capacity * 4, gl.DYNAMIC_DRAW);
    }
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, data.subarray(0, count * floats));
    gl.vertexAttribPointer(1, 4, gl.FLOAT, false, floats * 4, 0);
    gl.vertexAttribPointer(2, 4, gl.FLOAT, false, floats * 4, 16);
    gl.useProgram(this.prog);
    gl.uniformMatrix4fv(this.u.u_view, false, view);
    gl.uniformMatrix4fv(this.u.u_proj, false, proj);
    gl.uniform1f(this.u.u_time, time);
    gl.uniform2f(this.u.u_fog, fog[0], fog[1]);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
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
