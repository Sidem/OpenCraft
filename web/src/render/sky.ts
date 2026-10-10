// Day and night: `skyAt` turns the engine's time of day (0 midnight, 0.25 sunrise, 0.5 noon) into the
// sun's direction, sky and fog colours, the daylight that scales sky-lit terrain and the direct light
// (sun or moon) that lights the faces turned towards it; `SkyPass` draws the
// sky behind everything: a zenith-to-horizon gradient, the sun, the moon opposite it, and stars at
// night. Nights are dark blue, never black (a moonlight floor). Presentation only. To change the look:
// the colour constants below.

import { createProgram, uniforms } from './gl';

type Rgb = [number, number, number];

const DAY_ZENITH: Rgb = [0.36, 0.58, 0.92];
const DAY_HORIZON: Rgb = [0.62, 0.79, 0.96];
const NIGHT_ZENITH: Rgb = [0.015, 0.025, 0.07];
const NIGHT_HORIZON: Rgb = [0.05, 0.08, 0.17];
const DUSK_HORIZON: Rgb = [0.96, 0.56, 0.32];
const DAY_LIGHT: Rgb = [1, 1, 1];
const DUSK_LIGHT: Rgb = [1, 0.78, 0.6];
/** Moonlight: how bright sky-lit ground stays at midnight. */
const NIGHT_LIGHT: Rgb = [0.2, 0.24, 0.38];
/** Direct light on faces turned to it (shaders.ts `faceShade`): white at noon, golden low, pale moon. */
const NOON_SUN: Rgb = [1, 0.97, 0.9];
const LOW_SUN: Rgb = [1.6, 0.95, 0.5];
const MOON: Rgb = [0.32, 0.4, 0.62];

export interface Sky {
  /** Unit vector towards the sun (the moon is opposite). */
  sun: Rgb;
  zenith: Rgb;
  /** Horizon and fog colour. */
  horizon: Rgb;
  /** Multiplier for sky-lit surfaces. */
  light: Rgb;
  /** 0 at night, 1 by day. */
  day: number;
  /** Unit vector towards the direct light: the sun by day, the moon by night. */
  lightDir: Rgb;
  /** The direct light's colour and strength (fades out as its source nears the horizon). */
  direct: Rgb;
}

export function skyAt(t: number): Sky {
  // The sun rises in the east (+x) at 0.25, stands high in the south at noon, sets in the west.
  const a = (t - 0.25) * 2 * Math.PI;
  const sun = normalize([Math.cos(a), Math.sin(a), 0.35]);
  const h = sun[1];
  const day = smoothstep(-0.12, 0.22, h);
  const dusk = Math.exp(-(h * h) / 0.018) * smoothstep(-0.25, -0.05, h);
  const light = mix(mix(NIGHT_LIGHT, DAY_LIGHT, day), DUSK_LIGHT, dusk * 0.7);
  const horizon = mix(mix(NIGHT_HORIZON, DAY_HORIZON, day), DUSK_HORIZON, dusk * 0.55);
  const lightDir: Rgb = h >= 0 ? sun : [-sun[0], -sun[1], -sun[2]];
  const rise = smoothstep(0, 0.1, Math.abs(h));
  const colour = h >= 0 ? mix(LOW_SUN, NOON_SUN, smoothstep(0.05, 0.45, h)) : MOON;
  const direct: Rgb = [colour[0] * rise, colour[1] * rise, colour[2] * rise];
  return { sun, zenith: mix(NIGHT_ZENITH, DAY_ZENITH, day), horizon, light, day, lightDir, direct };
}

/** The time of day as a clock, e.g. "14:05". */
export function clock(t: number): string {
  const minutes = Math.floor(t * 24 * 60);
  return `${String(Math.floor(minutes / 60)).padStart(2, '0')}:${String(minutes % 60).padStart(2, '0')}`;
}

export class SkyPass {
  private readonly prog: WebGLProgram;
  private readonly u;
  private readonly vao: WebGLVertexArrayObject;

  constructor(private readonly gl: WebGL2RenderingContext) {
    this.prog = createProgram(gl, skyVert, skyFrag);
    this.u = uniforms(gl, this.prog, ['u_forward', 'u_right', 'u_up', 'u_sun', 'u_zenith', 'u_horizon', 'u_day'] as const);
    this.vao = gl.createVertexArray()!; // a full-screen triangle made from gl_VertexID
  }

  /** Draws the sky for a camera looking along `yaw`/`pitch` with vertical field of view `fovY`. */
  draw(sky: Sky, yaw: number, pitch: number, fovY: number, aspect: number): void {
    const gl = this.gl, u = this.u;
    const cp = Math.cos(pitch), sp = Math.sin(pitch), cy = Math.cos(yaw), sy = Math.sin(yaw);
    const ty = Math.tan(fovY / 2), tx = ty * aspect;
    // Same basis as mat4.viewRotation: forward f, right s, up = s × f.
    const f: Rgb = [sy * cp, sp, -cy * cp], s: Rgb = [cy, 0, sy];
    const up: Rgb = [s[1] * f[2] - s[2] * f[1], s[2] * f[0] - s[0] * f[2], s[0] * f[1] - s[1] * f[0]];
    gl.disable(gl.DEPTH_TEST);
    gl.depthMask(false);
    gl.useProgram(this.prog);
    gl.uniform3f(u.u_forward, ...f);
    gl.uniform3f(u.u_right, s[0] * tx, s[1] * tx, s[2] * tx);
    gl.uniform3f(u.u_up, up[0] * ty, up[1] * ty, up[2] * ty);
    gl.uniform3f(u.u_sun, ...sky.sun);
    gl.uniform3f(u.u_zenith, ...sky.zenith);
    gl.uniform3f(u.u_horizon, ...sky.horizon);
    gl.uniform1f(u.u_day, sky.day);
    gl.bindVertexArray(this.vao);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    gl.depthMask(true);
    gl.enable(gl.DEPTH_TEST);
  }
}

const skyVert = /* glsl */ `#version 300 es
out vec2 v_ndc;
void main() {
  v_ndc = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2)) * 2.0 - 1.0;
  gl_Position = vec4(v_ndc, 0.0, 1.0);
}
`;

const skyFrag = /* glsl */ `#version 300 es
precision highp float;
uniform vec3 u_forward, u_right, u_up, u_sun, u_zenith, u_horizon;
uniform float u_day;
in vec2 v_ndc;
out vec4 o_color;
float hash(vec3 p) { return fract(sin(dot(p, vec3(12.9898, 78.233, 37.719))) * 43758.5453); }
void main() {
  vec3 dir = normalize(u_forward + v_ndc.x * u_right + v_ndc.y * u_up);
  vec3 col = mix(u_horizon, u_zenith, pow(clamp(dir.y, 0.0, 1.0), 0.55));
  float s = dot(dir, u_sun);
  // Sun: a crisp disc with a soft glow; it fades as it sets.
  float up = smoothstep(-0.1, 0.05, u_sun.y);
  col += vec3(1.0, 0.85, 0.6) * pow(max(s, 0.0), 48.0) * 0.35 * up;
  col = mix(col, vec3(1.0, 0.96, 0.84), smoothstep(0.9990, 0.9994, s) * up);
  // Moon opposite the sun, and stars, both only at night.
  float night = 1.0 - u_day;
  col = mix(col, vec3(0.86, 0.9, 1.0), smoothstep(0.9993, 0.9996, -s) * night);
  vec3 cell = floor(dir * 240.0);
  float star = step(0.997, hash(cell)) * smoothstep(0.0, 0.25, dir.y) * smoothstep(0.8, 1.0, night);
  col += vec3(star * 0.8);
  o_color = vec4(col, 1.0);
}
`;

function smoothstep(a: number, b: number, x: number): number {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
}

function mix(a: Rgb, b: Rgb, t: number): Rgb {
  return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
}

function normalize(v: Rgb): Rgb {
  const l = Math.hypot(v[0], v[1], v[2]);
  return [v[0] / l, v[1] / l, v[2] / l];
}
