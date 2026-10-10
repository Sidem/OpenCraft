// GLSL ES 3.00 sources. `CUTOUT` is defined for the alpha-tested (leaves) variant only, so the
// opaque pass never contains `discard` and keeps early-z. `TERRAIN` (chunk meshes only) adds a
// gentle world-anchored shift in tone across blocks; texels are always sampled exactly as drawn, so
// the pixel art stays crisp. Repetition is broken in the textures instead (alternates per block).
// `WATER` is the translucent liquid variant (render/water.ts): no AO (those bits mark the water line:
// 1 lowers it a tenth of a block, 2 and 3 more for thinner flowing water), a surface that drifts over
// time and glints in the sun, and the texture's alpha kept.

export const TERRAIN_TINT_PERIOD = 256;

const FOG = /* glsl */ `
uniform vec3 u_fogColor;
uniform vec2 u_fog; // start, end (world units from the camera)
vec3 applyFog(vec3 col, vec3 rel) {
  float f = clamp((length(rel) - u_fog.x) / (u_fog.y - u_fog.x), 0.0, 1.0);
  return mix(col, u_fogColor, f * f);
}
`;

// The colour of a cell's light (sky light | block light << 4, each 0..15, from light.rs), shared by the
// terrain and the instanced boxes so machines and items are lit like the ground beside them. Each sky
// light level below 15 dims by a fifth; block light (lamps) is warm and fades a little more gently;
// caves keep a faint floor. That light is the ambient part (`AMBIENT` of a face's shade); `sunLight` adds
// the sun (the moon at night) on top, in its own colour, to faces turned towards it and only where the
// sky reaches, so a low sun still lights the slopes facing it gold while the rest falls into shade.
const LIGHT = /* glsl */ `
const vec3 BLOCK_LIGHT = vec3(1.1, 0.88, 0.6);
const float CAVE_FLOOR = 0.05;
const float AMBIENT = 0.8;
const float DIRECT = 0.34;
uniform vec3 u_sunDir; // towards the sun by day, the moon by night (render/sky.ts)
uniform vec3 u_direct; // that light's colour and strength
vec3 lightTint(uint light, vec3 skyLight) {
  float sky = pow(0.8, 15.0 - float(light & 15u));
  float lamp = pow(0.84, 15.0 - float(light >> 4u)) * step(1.0, float(light >> 4u));
  return max(sky * skyLight, lamp * BLOCK_LIGHT) + CAVE_FLOOR;
}
vec3 sunLight(vec3 n, uint light) {
  float sky = pow(0.8, 15.0 - float(light & 15u));
  return u_direct * (DIRECT * max(dot(n, u_sunDir), 0.0) * sky * sky);
}
`;

export const chunkVert = /* glsl */ `#version 300 es
precision highp float;
precision highp int;

layout(location = 0) in uint a_vert;
layout(location = 1) in uint a_light; // sky light | block light << 4, each 0..15 (light.rs)

uniform mat4 u_viewProj;
uniform vec3 u_offset; // chunk origin minus camera position
uniform vec3 u_worldOrigin; // chunk origin modulo the tint's 256-block period
uniform vec3 u_skyLight; // daylight colour (render/sky.ts)

out vec3 v_uvl;
out float v_light;
out vec3 v_sun;
out vec3 v_tint;
out vec3 v_rel;
out vec2 v_ground;

// Faces +X, -X, +Y, -Y, +Z, -Z, then a plant's two diagonal quads (lit mostly from above).
const float FACE_SHADE[8] = float[8](0.72, 0.72, 1.0, 0.52, 0.86, 0.86, 0.9, 0.9);
const vec3 FACE_NORMAL[8] = vec3[8](vec3(1, 0, 0), vec3(-1, 0, 0), vec3(0, 1, 0), vec3(0, -1, 0), vec3(0, 0, 1),
                                    vec3(0, 0, -1), vec3(0, 0.7, 0), vec3(0, 0.7, 0));
const float AO_CURVE[4] = float[4](0.40, 0.60, 0.80, 1.0);
${LIGHT}
void main() {
  vec3 p = vec3(float(a_vert & 63u), float((a_vert >> 6u) & 63u), float((a_vert >> 12u) & 63u));
  uint face = (a_vert >> 18u) & 7u;
  uint ao = (a_vert >> 21u) & 3u;
  float layer = float(a_vert >> 23u);

  // UVs come from the position, so merged quads tile their texture (wrap = REPEAT).
  vec2 uv;
  if (face == 0u) uv = vec2(-p.z, -p.y);
  else if (face == 1u) uv = vec2(p.z, -p.y);
  else if (face == 2u) uv = p.xz;
  else if (face == 3u) uv = vec2(p.x, -p.z);
  else if (face == 4u) uv = vec2(p.x, -p.y);
  else if (face == 5u) uv = vec2(-p.x, -p.y);
  else if (face == 6u) uv = vec2(p.x, -p.y);
  else uv = vec2(p.z, -p.y);

#ifdef WATER
  p.y -= ao > 0u ? 0.3 * float(ao) - 0.2 : 0.0;
  v_uvl = vec3(uv, layer);
  v_light = FACE_SHADE[face];
  v_sun = vec3(0.0);
#else
  v_uvl = vec3(uv, layer);
  v_light = (face >= 6u ? 0.84 : FACE_SHADE[face]) * AMBIENT * AO_CURVE[ao];
  v_sun = sunLight(FACE_NORMAL[face], a_light) * AO_CURVE[ao];
#endif
  v_tint = lightTint(a_light, u_skyLight);
  v_rel = u_offset + p;
  v_ground = (u_worldOrigin + p).xz;
  gl_Position = u_viewProj * vec4(v_rel, 1.0);
}
`;

export const litFrag = /* glsl */ `#version 300 es
precision highp float;
precision highp sampler2DArray;

uniform sampler2DArray u_tex;
${FOG}
in vec3 v_uvl;
in float v_light;
in vec3 v_sun;
in vec3 v_tint;
in vec3 v_rel;
out vec4 o_color;

#ifdef TERRAIN
in vec2 v_ground;
float terrainHash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}
// Smooth value noise with 'cells' lattice cells per period, so it wraps with u_worldOrigin.
float terrainField(vec2 p, float cells) {
  vec2 cell = floor(p), f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  float a = terrainHash(mod(cell, cells)), b = terrainHash(mod(cell + vec2(1, 0), cells));
  float c = terrainHash(mod(cell + vec2(0, 1), cells)), d = terrainHash(mod(cell + vec2(1, 1), cells));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}
#endif
#ifdef WATER
uniform float u_time; // seconds
uniform vec3 u_sunDir;
uniform vec3 u_direct;
#endif

void main() {
#ifdef WATER
  // Two copies of the texture drifting apart (one mirrored, half a block off) make the surface shimmer.
  vec4 a = texture(u_tex, v_uvl + vec3(u_time * 0.21, u_time * 0.13, 0.0));
  vec4 b = texture(u_tex, vec3(0.5 - v_uvl.x + u_time * 0.11, v_uvl.y + 0.37 - u_time * 0.17, v_uvl.z));
  vec4 c = mix(a, b, 0.5);
  // The sun (the moon at night) glints on open water: its reflection off a surface the two copies ripple.
  vec3 n = normalize(vec3((a.r - b.r) * 0.9, 1.0, (a.g - b.g) * 0.9));
  float glint = pow(max(dot(reflect(normalize(v_rel), n), u_sunDir), 0.0), 160.0)
              * step(0.95, v_light) * clamp(v_tint.g * 1.5 - 0.2, 0.0, 1.0);
  vec3 col = c.rgb * v_light * v_tint + u_direct * glint * 1.6;
  o_color = vec4(applyFog(col, v_rel), max(c.a, min(glint, 1.0)));
#else
  vec4 c = texture(u_tex, v_uvl);
#ifdef CUTOUT
  if (c.a < 0.5) discard;
#endif
#ifdef TERRAIN
  // Patches of about 16 and 64 blocks, a few percent brighter and warmer or darker and cooler.
  float field = terrainField(v_ground / 16.0, ${TERRAIN_TINT_PERIOD / 16}.0) * 0.65
              + terrainField(v_ground / 64.0, ${TERRAIN_TINT_PERIOD / 64}.0) * 0.35;
  c.rgb *= mix(vec3(0.93, 0.96, 1.0), vec3(1.05, 1.03, 0.95), field);
#endif
  o_color = vec4(applyFog(c.rgb * (v_light * v_tint + v_sun), v_rel), 1.0);
#endif
}
`;

// Instanced boxes: dropped items, items on belts and machine parts, all in one draw call.
// Instance layout matches `factory::INSTANCE_FLOATS` in the engine.
export const boxVert = /* glsl */ `#version 300 es
precision highp float;

layout(location = 0) in vec4 a_corner; // unit cube corner (±0.5) and face index (+X, -X, +Y, -Y, +Z, -Z)
layout(location = 1) in vec4 a_i0;     // camera-relative centre, yaw
layout(location = 2) in vec4 a_i1;     // size, uv scroll (top face)
layout(location = 3) in vec4 a_i2;     // texture layer top, side, bottom; uv mode (0 whole texture, 1 world-scaled) + 2 * light
layout(location = 4) in vec4 a_i3;     // pitch, roll, top-width taper, glow (0 or 1)

uniform mat4 u_viewProj;
uniform vec3 u_skyLight;

out vec3 v_uvl;
out float v_light;
out vec3 v_sun;
out vec3 v_tint;
out vec3 v_rel;
${LIGHT}
const vec3 NORMALS[6] = vec3[6](vec3(1, 0, 0), vec3(-1, 0, 0), vec3(0, 1, 0), vec3(0, -1, 0), vec3(0, 0, 1), vec3(0, 0, -1));

void main() {
  int face = int(a_corner.w + 0.5);
  vec3 local = a_corner.xyz * a_i1.xyz;
  local.x *= mix(1.0, a_i3.z, a_corner.y + 0.5);
  // Whole texture on every face (items), or texels at world scale like terrain (machine parts).
  float worldUv = mod(a_i2.w, 2.0);
  vec3 q = mix(a_corner.xyz, local, worldUv) + 0.5;
  vec2 uv;
  if (face == 0) uv = vec2(-q.z, -q.y);
  else if (face == 1) uv = vec2(q.z, -q.y);
  else if (face == 2) uv = vec2(q.x, q.z + a_i1.w);
  else if (face == 3) uv = vec2(q.x, -q.z);
  else if (face == 4) uv = vec2(q.x, -q.y);
  else uv = vec2(-q.x, -q.y);
  float layer = face == 2 ? a_i2.x : (face == 3 ? a_i2.z : a_i2.y);

  // Yaw turns local -Z towards (sin, 0, -cos), matching the player's look direction.
  float s = sin(a_i0.w), c = cos(a_i0.w);
  float sp = sin(a_i3.x), cp = cos(a_i3.x), sr = sin(a_i3.y), cr = cos(a_i3.y);
  mat3 tilt = mat3(1.0, 0.0, 0.0, 0.0, cp, sp, 0.0, -sp, cp);
  mat3 roll = mat3(cr, sr, 0.0, -sr, cr, 0.0, 0.0, 0.0, 1.0);
  mat3 rot = mat3(c, 0.0, s, 0.0, 1.0, 0.0, -s, 0.0, c) * roll * tilt;
  vec3 n = rot * NORMALS[face];
  uint light = uint(a_i2.w * 0.5);
  float glow = a_i3.w; // a status light, lamp, beam or flame lights itself (factory/render.rs GLOWING)
  v_light = mix((n.y > 0.5 ? 1.0 : (n.y < -0.5 ? 0.52 : (abs(n.x) > 0.5 ? 0.72 : 0.86))) * AMBIENT, 1.0, glow);
  v_sun = sunLight(n, light) * (1.0 - glow);
  v_uvl = vec3(uv, layer);
  v_tint = mix(lightTint(light, u_skyLight), vec3(1.0), glow);
  v_rel = a_i0.xyz + rot * local;
  gl_Position = u_viewProj * vec4(v_rel, 1.0);
}
`;

export const lineVert = /* glsl */ `#version 300 es
precision highp float;
layout(location = 0) in vec3 a_pos;
uniform mat4 u_viewProj;
uniform vec3 u_offset;
uniform vec3 u_scale;
void main() {
  gl_Position = u_viewProj * vec4(u_offset + a_pos * u_scale * 1.004 - 0.002, 1.0);
}
`;

export const lineFrag = /* glsl */ `#version 300 es
precision mediump float;
uniform vec4 u_color;
out vec4 o_color;
void main() { o_color = u_color; }
`;

export const crackVert = /* glsl */ `#version 300 es
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

export const crackFrag = /* glsl */ `#version 300 es
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
