// The cloud layer's noise and its shader code. One 256 x 256 tiling picture of soft cloud noise (`bakeCloudNoise`, baked in a
// few milliseconds at start) is sampled a few times per pixel by the sky shader (sky.ts) and once per star by the star shader
// (stars.ts), so a star behind a cloud goes out and one between clouds shines. `CLOUD_GLSL` holds the shared function
// `cloudAt`. The picture is laid on a flat layer high above (`dir.xz / (dir.y + 0.25)`), drifts with the game time (`windOf`,
// wraps every 64 game days without a seam) and grows with the weather's cover. Presentation only.
// To change the look: the scales and the smoothstep widths in `CLOUD_GLSL`; the colours are in sky.ts `cloudColours`.

export const CLOUD_SIZE = 256;

/** Cloud noise that tiles: four octaves of smooth value noise, stretched to use the whole 0..255 range. */
export function bakeCloudNoise(): Uint8Array {
  let state = 0x2545f491;
  const rand = () => {
    state = (state + 0x6d2b79f5) | 0;
    let t = Math.imul(state ^ (state >>> 15), 1 | state);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
  const acc = new Float32Array(CLOUD_SIZE * CLOUD_SIZE);
  for (const [cells, weight] of [[4, 1], [8, 0.55], [16, 0.3], [32, 0.16]]) {
    const lattice = Array.from({ length: cells * cells }, rand);
    const at = (x: number, y: number) => lattice[(y % cells) * cells + (x % cells)];
    for (let y = 0; y < CLOUD_SIZE; y++) {
      const fy = (y / CLOUD_SIZE) * cells, y0 = Math.floor(fy), ty = fy - y0, sy = ty * ty * (3 - 2 * ty);
      for (let x = 0; x < CLOUD_SIZE; x++) {
        const fx = (x / CLOUD_SIZE) * cells, x0 = Math.floor(fx), tx = fx - x0, sx = tx * tx * (3 - 2 * tx);
        const top = at(x0, y0) + (at(x0 + 1, y0) - at(x0, y0)) * sx;
        const bottom = at(x0, y0 + 1) + (at(x0 + 1, y0 + 1) - at(x0, y0 + 1)) * sx;
        acc[y * CLOUD_SIZE + x] += (top + (bottom - top) * sy) * weight;
      }
    }
  }
  let lo = Infinity, hi = -Infinity;
  for (const v of acc) { lo = Math.min(lo, v); hi = Math.max(hi, v); }
  return Uint8Array.from(acc, (v) => Math.round(((v - lo) / (hi - lo)) * 255));
}

/** The cloud drift for the game time `days`: texture units, seamless every 64 days (the layers move by 16, 8 and 24 tiles). */
export function windOf(days: number): number {
  return (days % 64) * 0.25;
}

/** Uploads the noise as a repeating, mip-mapped single-channel texture. */
export function createCloudTexture(gl: WebGL2RenderingContext): WebGLTexture {
  const tex = gl.createTexture()!;
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8, CLOUD_SIZE, CLOUD_SIZE, 0, gl.RED, gl.UNSIGNED_BYTE, bakeCloudNoise());
  gl.pixelStorei(gl.UNPACK_ALIGNMENT, 4);
  gl.generateMipmap(gl.TEXTURE_2D);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.REPEAT);
  return tex;
}

/** The texture unit the cloud picture is bound to in both passes. */
export const CLOUD_UNIT = 2;

/**
 * Declares `u_cloudTex`, `cloudNoise(dir, wind, shift)` (the layer's noise towards the sky direction `dir`, about 0.2 to 0.8;
 * `shift` moves the lookup in the layer, the sky shader asks again a step towards the sun to light a cloud's side),
 * `cloudThick(d, cover)` (the thickness, 0 clear to 1 solid, once the weather's `cover` has cut the noise) and
 * `cloudAt(dir, cover, wind)` for both together. Define `CLOUD_SAMPLE(tex, uv)` first: `texture` in a fragment shader,
 * `textureLod(.., 0.0)` in a vertex one.
 */
export const CLOUD_GLSL = /* glsl */ `
uniform sampler2D u_cloudTex;
float cloudNoise(vec3 dir, float wind, vec2 shift) {
  vec2 q = dir.xz / (dir.y + 0.25) + shift;
  float a = CLOUD_SAMPLE(u_cloudTex, q * 0.22 + vec2(wind, wind * 0.5)).r;
  float b = CLOUD_SAMPLE(u_cloudTex, q * 0.7 - vec2(wind * 1.5, 0.0) + 0.37).r;
  return a * 0.62 + b * 0.38;
}
float cloudThick(float d, float cover) {
  float edge = 1.0 - cover;
  return smoothstep(edge - 0.1, edge + 0.22, d);
}
float cloudAt(vec3 dir, float cover, float wind) {
  return cloudThick(cloudNoise(dir, wind, vec2(0.0)), cover);
}
`;
