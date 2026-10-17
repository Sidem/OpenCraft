// Day and night: `skyAt` turns the engine's time of day (0 midnight, 0.25 sunrise, 0.5 noon) into the
// sun's direction, sky and fog colours, the daylight that scales sky-lit terrain and the direct light
// (sun or moon) that lights the faces turned towards it; `SkyPass` draws the
// sky behind everything: a zenith-to-horizon gradient, the sun, the moon, stars at night, and satellites in orbit
// as small gold lights (engine `sky_satellites`: x, y, z, kind per satellite).
// The night has two looks (`SkySettings.quality`, sky/settings.ts): 0 is the classic one (a full moon opposite the sun, hashed
// stars), 1 and 2 the real sky (sky/astro.ts says where everything is): the moon as a lit ball with its phase, the catalogue
// stars (sky/stars.ts draws them after this pass) and, at 2, the Milky Way picture. Until the star files have loaded the classic
// look shows. Nights are dark blue, never black (a moonlight floor). Presentation only. To change the look: the colour
// constants below, and the moon and Milky Way lines in the fragment shader.
// Weather (sky/weather.ts, clouds in sky/clouds.ts): given a `SkyLook`, `skyAt` greys, dims and shortens the sky under
// gloom (colours, light, direct sun) and warms it at dusk when asked, and the shader adds the red dusk glow and one cloud layer
// that hides the sun, moon, Milky Way and satellites (the stars pass fades each star behind a cloud). Without a look
// (weather and red dusk off) nothing changes: the sky is byte-identical to the one before step S2.

import { createProgram, uniforms } from './gl';
import type { SkyFrame } from './sky/astro';
import { CLOUD_GLSL, CLOUD_UNIT, createCloudTexture, windOf } from './sky/clouds';
import type { SkySettings } from './sky/settings';
import { StarsPass } from './sky/stars';
import type { SkyLook } from './sky/weather';

type Rgb = [number, number, number];

/** What the night sky needs besides `Sky`: the settings, where the stars and moon are, the time in seconds, the game time in
 *  days (the clouds drift with it) and the weather look (absent while the weather and red dusk are both off). */
export interface SkyView {
  settings: SkySettings;
  frame: SkyFrame;
  time: number;
  days: number;
  look?: SkyLook;
}

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
/** The red dusk setting: a redder horizon (and fog) and a warmer light on the ground at sunset and sunrise. */
const DUSK_RED: Rgb = [1, 0.38, 0.2];
const DUSK_WARM: Rgb = [1, 0.62, 0.46];

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

/** Weather and the red dusk (sky/weather.ts `SkyLook`) change the colours, the light and the fog: without a `look` the sky is the plain one. */
export function skyAt(t: number, look?: SkyLook): Sky {
  const sky = plainSky(t);
  return look ? weathered(sky, look) : sky;
}

function plainSky(t: number): Sky {
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

/** How strong the red glow of dusk and dawn is for a sun at height `h` (sine of its elevation), 0 to 1; it lingers after sunset. */
function duskGlow(h: number): number {
  return Math.exp(-(h * h) / 0.03) * smoothstep(-0.3, -0.05, h);
}

const grey = (c: Rgb, k: number, scale: number): Rgb => {
  const l = 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2];
  return [(c[0] + (l - c[0]) * k) * scale, (c[1] + (l - c[1]) * k) * scale, (c[2] + (l - c[2]) * k) * scale];
};

/** The plain sky seen through the weather: a red dusk if asked, then grey, dim and short-sighted under heavy cloud. */
function weathered(s: Sky, look: SkyLook): Sky {
  const g = look.gloom;
  let { zenith, horizon, light, direct } = s;
  if (look.dusk) {
    const glow = duskGlow(s.sun[1]) * (1 - g);
    horizon = mix(horizon, DUSK_RED, glow * 0.45);
    light = mix(light, DUSK_WARM, glow * 0.35);
  }
  if (g > 0 || look.cover > 0.3) {
    const sun = 1 - Math.min(0.9, 0.25 * Math.max(0, look.cover - 0.3) + 0.65 * g);
    zenith = grey(zenith, 0.7 * g, 1 - 0.45 * g);
    horizon = grey(horizon, 0.7 * g, 1 - 0.3 * g);
    light = grey(light, 0.3 * g, 1 - 0.3 * g);
    direct = [direct[0] * sun, direct[1] * sun, direct[2] * sun];
  }
  return { ...s, zenith, horizon, light, direct };
}

/** Colours of a cloud's lit and shaded sides: white by day, golden or red low sun (strongly red with the dusk on), blue-grey moonlit, grey when gloomy. */
function cloudColours(s: Sky, look: SkyLook): { lit: Rgb; dark: Rgb } {
  const h = s.sun[1];
  let lit = mix(mix([0.14, 0.17, 0.28], [1, 0.8, 0.6], s.day), [1.05, 1.05, 1.05], smoothstep(0.05, 0.4, h) * s.day);
  let dark = mix(lit, [0.55, 0.62, 0.78], 0.5);
  dark = [dark[0] * 0.62, dark[1] * 0.62, dark[2] * 0.66];
  if (look.dusk) {
    const glow = duskGlow(h) * (1 - look.gloom);
    lit = mix(lit, [1.1, 0.45, 0.22], glow * 0.85);
    dark = mix(dark, [0.5, 0.24, 0.32], glow * 0.7);
  }
  const g = look.gloom;
  return { lit: grey(lit, 0.75 * g, 1 - 0.35 * g), dark: grey(dark, 0.75 * g, 1 - 0.4 * g) };
}

/** The time of day as a clock, e.g. "14:05". */
export function clock(t: number): string {
  const minutes = Math.floor(t * 24 * 60);
  return `${String(Math.floor(minutes / 60)).padStart(2, '0')}:${String(minutes % 60).padStart(2, '0')}`;
}

/** Most satellites drawn (the shader's array size; the engine keeps at most 24 of a kind). */
const MAX_SATELLITES = 32;

export class SkyPass {
  private readonly prog: WebGLProgram;
  private readonly u;
  private readonly vao: WebGLVertexArrayObject;

  private readonly stars: StarsPass;
  private cloudTexture: WebGLTexture | null = null; // made when clouds are first wanted

  constructor(private readonly gl: WebGL2RenderingContext) {
    this.prog = createProgram(gl, skyVert, skyFrag);
    this.u = uniforms(gl, this.prog, [
      'u_forward', 'u_right', 'u_up', 'u_sun', 'u_zenith', 'u_horizon', 'u_day', 'u_sats', 'u_satCount',
      'u_quality', 'u_eq', 'u_moon', 'u_moonR', 'u_moonU', 'u_moonL', 'u_milky', 'u_mwLod',
      'u_cloudOn', 'u_cloud', 'u_cloudTex', 'u_cloudLit', 'u_cloudDark', 'u_cloudSun', 'u_dusk',
    ] as const);
    this.vao = gl.createVertexArray()!; // a full-screen triangle made from gl_VertexID
    this.stars = new StarsPass(gl);
  }

  /** Draws the sky for a camera looking along `yaw`/`pitch` with vertical field of view `fovY`; `satellites` holds
   *  at most `MAX_SATELLITES` records (x, y, z of a unit direction, then the kind). `view` says how the night sky is
   *  drawn (settings, and where the stars and moon are: sky/astro.ts); without it, or while its files load, the classic sky. */
  draw(sky: Sky, yaw: number, pitch: number, fovY: number, aspect: number, satellites: Float32Array, view?: SkyView): void {
    const gl = this.gl, u = this.u;
    const cp = Math.cos(pitch), sp = Math.sin(pitch), cy = Math.cos(yaw), sy = Math.sin(yaw);
    const ty = Math.tan(fovY / 2), tx = ty * aspect;
    // Same basis as mat4.viewRotation: forward f, right s, up = s × f.
    const f: Rgb = [sy * cp, sp, -cy * cp], s: Rgb = [cy, 0, sy];
    const up: Rgb = [s[1] * f[2] - s[2] * f[1], s[2] * f[0] - s[0] * f[2], s[0] * f[1] - s[1] * f[0]];
    const want = view ? view.settings.quality : 0;
    this.stars.request(want);
    const quality = !this.stars.ready ? 0 : want === 2 && !this.stars.milky ? 1 : want;
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
    const shown = Math.min(satellites.length >> 2, MAX_SATELLITES);
    gl.uniform1i(u.u_satCount, shown);
    if (shown > 0) gl.uniform4fv(u.u_sats, satellites.subarray(0, shown * 4));
    gl.uniform1i(u.u_quality, quality);
    const look = view?.look;
    const clouds = look ? look.clouds : 0;
    gl.uniform1i(u.u_cloudOn, clouds);
    let cloud: { cover: number; wind: number } | undefined;
    if (look && view) {
      gl.uniform1f(u.u_dusk, look.dusk ? duskGlow(sky.sun[1]) * (1 - look.gloom) : 0);
      if (clouds > 0) {
        cloud = { cover: look.cover, wind: windOf(view.days) };
        const c = cloudColours(sky, look);
        const sl = Math.hypot(sky.sun[0], sky.sun[2]) || 1;
        gl.uniform3f(u.u_cloud, cloud.cover, cloud.wind, clouds === 2 ? 1 : 0);
        gl.uniform3f(u.u_cloudLit, ...c.lit);
        gl.uniform3f(u.u_cloudDark, ...c.dark);
        gl.uniform2f(u.u_cloudSun, (sky.sun[0] / sl) * 0.05, (sky.sun[2] / sl) * 0.05);
        this.bindClouds();
        gl.uniform1i(u.u_cloudTex, CLOUD_UNIT);
      }
    } else {
      gl.uniform1f(u.u_dusk, 0);
    }
    if (view && quality > 0) {
      const m = view.frame.moon, r = normalize(cross(m, [0, 1, 0])); // the moon's right and up as seen from here
      const mu = cross(r, m), sn = view.frame.sun;
      gl.uniformMatrix3fv(u.u_eq, false, view.frame.rotation);
      gl.uniform3f(u.u_moon, ...m);
      gl.uniform3f(u.u_moonR, ...r);
      gl.uniform3f(u.u_moonU, ...mu);
      gl.uniform3f(u.u_moonL, dot(sn, r), dot(sn, mu), dot(sn, m));
      if (quality === 2) {
        gl.activeTexture(gl.TEXTURE1);
        gl.bindTexture(gl.TEXTURE_2D, this.stars.milky);
        gl.activeTexture(gl.TEXTURE0);
        gl.uniform1i(u.u_milky, 1);
        // Texels are 2π/2048 radians wide; a coarser mip when a pixel covers more than one.
        gl.uniform1f(u.u_mwLod, Math.max(0, Math.log2(((fovY / gl.drawingBufferHeight) * 2048) / (2 * Math.PI))));
      }
    }
    gl.bindVertexArray(this.vao);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    if (view && quality > 0 && sky.day < 0.98) {
      this.stars.draw(view.frame.rotation, { forward: f, right: s, up, tanX: tx, tanY: ty }, 1 - sky.day, view.settings.lines, view.settings.twinkle, gl.drawingBufferHeight, view.time, cloud);
    }
    gl.depthMask(true);
    gl.enable(gl.DEPTH_TEST);
  }

  private bindClouds(): void {
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE0 + CLOUD_UNIT);
    this.cloudTexture ??= createCloudTexture(gl);
    gl.bindTexture(gl.TEXTURE_2D, this.cloudTexture);
    gl.activeTexture(gl.TEXTURE0);
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
#define CLOUD_SAMPLE(t, uv) texture(t, uv)
${CLOUD_GLSL}
uniform vec3 u_forward, u_right, u_up, u_sun, u_zenith, u_horizon;
uniform float u_day, u_mwLod, u_dusk;
uniform int u_cloudOn; // 0 no clouds, 1 low, 2 full
uniform vec3 u_cloud, u_cloudLit, u_cloudDark; // cover, wind and 1 for side lighting; the lit and shaded cloud colours
uniform vec2 u_cloudSun; // a short step in the cloud layer towards the sun
uniform int u_quality; // 0 classic, 1 real stars and moon phases, 2 and the Milky Way
uniform mat3 u_eq; // world to equatorial (sky/astro.ts)
uniform vec3 u_moon, u_moonR, u_moonU, u_moonL; // the moon's direction, its right and up, and the sun in that frame
uniform sampler2D u_milky;
uniform vec4 u_sats[${MAX_SATELLITES}]; // unit direction towards each satellite, then its kind
uniform int u_satCount;
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
  float night = 1.0 - u_day;
  if (u_quality == 0) {
    // The classic night: a full moon opposite the sun and hashed stars.
    col = mix(col, vec3(0.86, 0.9, 1.0), smoothstep(0.9993, 0.9996, -s) * night);
    vec3 cell = floor(dir * 240.0);
    float star = step(0.997, hash(cell)) * smoothstep(0.0, 0.25, dir.y) * smoothstep(0.8, 1.0, night);
    col += vec3(star * 0.8);
  } else {
    // The Milky Way (the stars are drawn after this pass, stars.ts), then the moon as a lit ball with its phase.
    if (u_quality == 2 && night > 0.5) {
      vec3 e = u_eq * dir;
      vec2 uv = vec2(0.5 - atan(e.y, e.x) * 0.15915494, 0.5 - asin(clamp(e.z, -1.0, 1.0)) * 0.31830989);
      vec3 mw = pow(textureLod(u_milky, uv, u_mwLod).rgb, vec3(1.8));
      mw = mix(vec3(dot(mw, vec3(0.3, 0.55, 0.15))), mw, 0.6); // the naked eye sees it paler than the camera does
      col += mw * vec3(0.4, 0.46, 0.6) * smoothstep(0.75, 1.0, night) * smoothstep(-0.02, 0.2, dir.y);
    }
    vec2 p = vec2(dot(dir, u_moonR), dot(dir, u_moonU)) / 0.0316;
    float r2 = dot(p, p);
    float mz = dot(dir, u_moon);
    float seen = mix(0.6, 1.0, night);
    col += vec3(0.45, 0.55, 0.9) * pow(max(mz, 0.0), 160.0) * 0.1 * (1.0 - u_moonL.z) * 0.5 * night;
    if (mz > 0.0 && r2 < 1.2) {
      vec3 n = vec3(p, -sqrt(max(1.0 - r2, 0.0))); // the face turned to us points back along the line of sight
      float lit = smoothstep(-0.03, 0.1, dot(n, u_moonL));
      float maria = 0.8 + 0.2 * smoothstep(0.35, 0.65, fract(sin(dot(floor(p * 5.0), vec2(12.9898, 78.233))) * 43758.5453));
      vec3 disc = vec3(0.88, 0.9, 1.0) * maria * lit + vec3(0.05, 0.065, 0.1) * (1.0 - lit); // earthshine on the dark part
      col = mix(col, disc, smoothstep(1.0, 0.94, sqrt(r2)) * seen);
    }
  }
  // The red dusk (a setting): a glow low on the horizon towards the sun, a rose band opposite it.
  if (u_dusk > 0.0) {
    vec2 around = normalize(dir.xz + 1e-5), toSun = normalize(u_sun.xz + 1e-5);
    float az = dot(around, toSun), low = exp(-max(dir.y, 0.0) * 5.0);
    col += vec3(1.0, 0.3, 0.1) * u_dusk * (pow(max(az, 0.0), 4.0) * 0.55 + pow(max(az, 0.0), 1.5) * 0.12) * low;
    col += vec3(0.5, 0.2, 0.35) * u_dusk * pow(max(-az, 0.0), 2.0) * 0.12 * exp(-max(dir.y, 0.0) * 3.0);
  }
  // Clouds: one flat layer from the noise picture, lit from the sun's side when the setting is Full. Satellites sit above it.
  float hidden = 0.0;
  if (u_cloudOn > 0) {
    float fade = smoothstep(0.0, 0.2, dir.y);
    if (fade > 0.0) {
      float n = cloudNoise(dir, u_cloud.y, vec2(0.0));
      float thick = cloudThick(n, u_cloud.x);
      float lit = 0.5 + 0.7 * (n - 0.5); // the cloud's own mottling, even under a solid sheet
      if (u_cloud.z > 0.5) lit += 2.5 * (thick - cloudThick(cloudNoise(dir, u_cloud.y, u_cloudSun), u_cloud.x));
      lit = clamp(lit, 0.0, 1.0);
      hidden = thick * fade * 0.94;
      col = mix(col, mix(u_cloudDark, u_cloudLit, lit), hidden);
    }
  }
  // Satellites: sunlit gold points, faint by day. The distance between unit vectors is the angle, without cancellation.
  for (int i = 0; i < u_satCount; i++) {
    float d = length(dir - u_sats[i].xyz);
    col += vec3(1.0, 0.82, 0.42) * (smoothstep(0.012, 0.002, d) + 0.5 * smoothstep(0.03, 0.0, d)) * mix(1.0, 0.35, u_day) * (1.0 - hidden);
  }
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

function dot(a: Rgb, b: Rgb): number {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

function cross(a: Rgb, b: Rgb): Rgb {
  return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
}

function normalize(v: Rgb): Rgb {
  const l = Math.hypot(v[0], v[1], v[2]);
  return [v[0] / l, v[1] / l, v[2] / l];
}
