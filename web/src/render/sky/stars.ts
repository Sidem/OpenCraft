// Draws the real stars: one `gl.POINTS` draw for the 8,920 catalogue stars (`stars.bin`, see astro.ts and CREDITS.md) and, when
// asked, one `gl.LINES` draw of the constellation figures from the same buffers, both added onto the sky that SkyPass
// has drawn. The vertex shader turns each star's fixed direction into the world with the sky's turn (astro.ts `skyFrame`)
// and onto the screen, so the CPU does nothing per star and per frame. The Milky Way picture is loaded here too and sampled by
// the sky shader (sky.ts). Both files load on first use and `ready` / `milky` say when they can be drawn; until then
// the sky falls back to the classic stars. To change the look: the brightness and size lines in the vertex shader.

import { createProgram, uniforms } from '../gl';
import { readStars, type StarCatalogue } from './astro';
import { CLOUD_GLSL, CLOUD_UNIT } from './clouds';

const STARS_URL = new URL('./stars.bin', import.meta.url).href;
const MILKY_URL = new URL('./milkyway.jpg', import.meta.url).href;

/** The camera as SkyPass works it out: unit forward, right and up, and the tangents of the half field of view. */
export interface SkyCamera {
  forward: [number, number, number];
  right: [number, number, number];
  up: [number, number, number];
  tanX: number;
  tanY: number;
}

export class StarsPass {
  ready = false;
  milky: WebGLTexture | null = null;
  private readonly prog: WebGLProgram;
  private readonly u;
  private readonly vao: WebGLVertexArrayObject;
  private starCount = 0;
  private lineCount = 0;
  private started = false;
  private startedMilky = false;

  constructor(private readonly gl: WebGL2RenderingContext) {
    this.prog = createProgram(gl, vert, frag);
    this.u = uniforms(gl, this.prog, ['u_eq', 'u_right', 'u_up', 'u_fwd', 'u_tan', 'u_night', 'u_time', 'u_twinkle', 'u_px', 'u_lines', 'u_cloudOn', 'u_cloud', 'u_cloudTex'] as const);
    this.vao = gl.createVertexArray()!;
  }

  /** Starts loading what `quality` needs (once). */
  request(quality: number): void {
    if (quality >= 1 && !this.started) {
      this.started = true;
      fetch(STARS_URL).then((r) => r.arrayBuffer()).then((b) => this.upload(readStars(b))).catch(() => (this.started = false));
    }
    if (quality >= 2 && !this.startedMilky) {
      this.startedMilky = true;
      fetch(MILKY_URL).then((r) => r.blob()).then((b) => createImageBitmap(b)).then((img) => this.uploadMilky(img)).catch(() => (this.startedMilky = false));
    }
  }

  /** Adds the stars (and the lines when `lines`) to the frame; `night` is 0 by day and 1 at night, `time` in seconds. */
  draw(eq: Float32Array | number[], cam: SkyCamera, night: number, lines: boolean, twinkle: boolean, height: number, time: number, cloud?: { cover: number; wind: number }): void {
    const gl = this.gl, u = this.u;
    gl.useProgram(this.prog);
    gl.uniform1i(u.u_cloudOn, cloud ? 1 : 0);
    if (cloud) {
      gl.uniform2f(u.u_cloud, cloud.cover, cloud.wind);
      gl.uniform1i(u.u_cloudTex, CLOUD_UNIT); // SkyPass has bound the picture to that unit for this frame
    }
    gl.uniformMatrix3fv(u.u_eq, false, eq);
    gl.uniform3f(u.u_right, ...cam.right);
    gl.uniform3f(u.u_up, ...cam.up);
    gl.uniform3f(u.u_fwd, ...cam.forward);
    gl.uniform2f(u.u_tan, cam.tanX, cam.tanY);
    gl.uniform1f(u.u_night, night);
    gl.uniform1f(u.u_time, time);
    gl.uniform1f(u.u_twinkle, twinkle ? 1 : 0);
    gl.uniform1f(u.u_px, Math.min(2, Math.max(0.75, height / 1000)));
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.ONE, gl.ONE);
    gl.bindVertexArray(this.vao);
    if (lines && this.lineCount > 0) {
      gl.uniform1i(u.u_lines, 1);
      gl.drawElements(gl.LINES, this.lineCount, gl.UNSIGNED_SHORT, 0);
    }
    gl.uniform1i(u.u_lines, 0);
    gl.drawArrays(gl.POINTS, 0, this.starCount);
    gl.disable(gl.BLEND);
  }

  private upload(c: StarCatalogue): void {
    const gl = this.gl;
    gl.bindVertexArray(this.vao);
    const data = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, data);
    gl.bufferData(gl.ARRAY_BUFFER, c.data, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);
    const colour = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, colour);
    gl.bufferData(gl.ARRAY_BUFFER, c.colour, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 4, gl.UNSIGNED_BYTE, true, 0, 0);
    const index = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, index);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, c.lines, gl.STATIC_DRAW);
    gl.bindVertexArray(null);
    this.starCount = c.count;
    this.lineCount = c.lines.length;
    this.ready = true;
  }

  private uploadMilky(img: ImageBitmap): void {
    const gl = this.gl;
    const tex = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, gl.RGBA, gl.UNSIGNED_BYTE, img);
    gl.generateMipmap(gl.TEXTURE_2D);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    img.close();
    this.milky = tex;
  }
}

const vert = /* glsl */ `#version 300 es
#define CLOUD_SAMPLE(t, uv) textureLod(t, uv, 0.0)
${CLOUD_GLSL}
layout(location = 0) in vec4 a_star; // equatorial unit vector, magnitude
layout(location = 1) in vec4 a_col;
uniform int u_cloudOn;
uniform vec2 u_cloud; // cover and wind (clouds.ts): a star behind cloud goes out
uniform mat3 u_eq; // world to equatorial (astro.ts: the turn's rows); world = eq * u_eq
uniform vec3 u_right, u_up, u_fwd;
uniform vec2 u_tan;
uniform float u_night, u_time, u_twinkle, u_px;
uniform int u_lines;
out vec3 v_col;
out float v_a;
void main() {
  vec3 d = a_star.xyz * u_eq;
  gl_Position = vec4(dot(d, u_right) / u_tan.x, dot(d, u_up) / u_tan.y, 0.0, dot(d, u_fwd));
  float low = smoothstep(-0.02, 0.2, d.y); // stars dim and vanish into the haze at the horizon
  if (u_cloudOn == 1) low *= 1.0 - cloudAt(d, u_cloud.x, u_cloud.y) * 0.94 * smoothstep(0.0, 0.2, d.y);
  if (u_lines == 1) {
    v_col = vec3(0.4, 0.55, 1.0);
    v_a = 0.22 * smoothstep(0.7, 0.95, u_night) * low;
    return;
  }
  float m = a_star.w;
  float b = clamp(1.05 - 0.1 * m, 0.3, 1.0);
  float th = 0.55 + 0.06 * m; // bright stars come out first at dusk
  float seen = smoothstep(th, th + 0.06, u_night);
  float tw = 1.0 + u_twinkle * 0.3 * sin(u_time * (4.0 + fract(a_star.x * 91.7) * 5.0) + a_star.y * 50.0) * (1.0 - 0.6 * d.y);
  v_col = a_col.rgb;
  v_a = b * seen * low * tw;
  gl_PointSize = (1.0 + 2.6 * b * b) * u_px;
}
`;

const frag = /* glsl */ `#version 300 es
precision mediump float;
uniform highp int u_lines;
in vec3 v_col;
in float v_a;
out vec4 o_color;
void main() {
  float a = v_a;
  if (u_lines == 0) {
    float r = length(gl_PointCoord - 0.5) * 2.0;
    a *= (1.0 - smoothstep(0.0, 1.0, r)) * 1.4;
  }
  o_color = vec4(v_col * a, 1.0);
}
`;
