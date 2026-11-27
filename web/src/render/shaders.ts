// GLSL ES 3.00 sources (the boxes' vertex shader is in box-glsl.ts, the trees' in shaped.ts; `SHAPED` is their
// fragment variant: no turning tiles, a canopy textured along its triangle's main axis). `CUTOUT` is defined for the alpha-tested (leaves) variant only, so the
// opaque pass never contains `discard` and keeps early-z. `TERRAIN` (chunk meshes only) adds a
// gentle world-anchored shift in tone across blocks; texels are always sampled exactly as drawn, so
// the pixel art stays crisp. Repetition is broken by alternates per block (the mesher) and, for natural layers,
// by turning each block's tile in place (`turnUv`, engine `block/turn.rs`), which keeps greedy quads merged.
// `WATER` is the translucent liquid variant (render/water.ts); its GLSL lives in water-glsl.ts.
import { WATER_FRAG, WATER_FRAG_DECL, WATER_VERT, WATER_VERT_DECL } from './water-glsl';
export { ICE_WORDS } from './water-glsl';

export const TERRAIN_TINT_PERIOD = 256;
/** The texture unit of the self-light array (`u_glow`); 1 and 2 belong to the rain mask and the clouds. */
export const GLOW_UNIT = 4;
/** Dynamic point lights the shader can show at once: up to 4 held torches + up to 12 fire lights (engine `held_light::MAX_LIGHTS`). */
export const HELD_LIGHT_SLOTS = 16;

export const FOG = /* glsl */ `
uniform vec3 u_fogColor;
uniform vec2 u_fog; // start, end (world units from the camera)
vec3 applyFog(vec3 col, vec3 rel) {
  float f = clamp((length(rel) - u_fog.x) / (u_fog.y - u_fog.x), 0.0, 1.0);
  return mix(col, u_fogColor, f * f);
}
`;

const LAMP_COLOUR = /* glsl */ `
const vec3 BLOCK_LIGHT = vec3(1.1, 0.88, 0.6);
const float CAVE_FLOOR = 0.05;
`;

// The lights players hold (engine held_light.rs; render/held-light.ts fills the uniform): a torch in a hand lights what
// is near it, in the lamps' colour, as bright as the baked light there or brighter. Added per fragment, not per vertex,
// because one merged quad can span the whole lit circle.
export const HELD_LIGHT = /* glsl */ `
${LAMP_COLOUR}
const int HELD_LIGHTS = ${HELD_LIGHT_SLOTS};
uniform vec4 u_held[HELD_LIGHTS]; // camera-relative x, y, z and the reach in blocks (0: no light)
vec3 heldGlow(vec3 rel) {
  float g = 0.0;
  for (int i = 0; i < HELD_LIGHTS; i++) {
    float w = u_held[i].w;
    if (w <= 0.001) break;
    float f = clamp((1.0 - length(rel - u_held[i].xyz) / w) * 1.6, 0.0, 1.0);
    g = max(g, f * f);
  }
  return g * BLOCK_LIGHT + CAVE_FLOOR;
}
`;

// The colour of a cell's light (sky light | block light << 4, each 0..15, from light.rs), shared by the
// terrain and the instanced boxes so machines and items are lit like the ground beside them. Each sky
// light level below 15 dims by a fifth; block light (lamps) is warm and fades a little more gently;
// caves keep a faint floor. That light is the ambient part (`AMBIENT` of a face's shade); `sunLight` adds
// the sun (the moon at night) on top, in its own colour, to faces turned towards it and only where the
// sky reaches, so a low sun still lights the slopes facing it gold while the rest falls into shade.
export const LIGHT = /* glsl */ `
${LAMP_COLOUR}
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
layout(location = 1) in uvec2 a_light; // .x = sky light | block light << 4, .y = paint (0..5) | (layer >> 9) << 5

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
out vec2 v_open; // how much open sky this vertex has (0..1) and 1 on an upward face: where rain wets and snow lies
#ifndef WATER
flat out uint v_paintId;
flat out uint v_seasonRole;
uniform uvec4 u_turns[32]; // two bits per layer (2048 layers): how its blocks may turn (engine block/turn.rs)
uniform uvec4 u_seasonMask[32]; // two bits per layer: 0 none, 1 grass, 2 woodland, 3 flower
uniform vec4 u_wind; // (dirX, dirZ, strength, phase) shared with GrassPass & shapedVert for saplings
// The face's offset in world tile cells (xy), its plane and side (z), and how it may turn (w): 0 not, 1 mirror,
// 2 mirror and flip upside down, 3 any of eight ways.
flat out vec4 v_turn;
#else
${WATER_VERT_DECL}
#endif

// Faces +X, -X, +Y, -Y, +Z, -Z, then a plant's two diagonal quads (lit mostly from above).
const float FACE_SHADE[8] = float[8](0.72, 0.72, 1.0, 0.52, 0.86, 0.86, 0.9, 0.9);
const vec3 FACE_NORMAL[8] = vec3[8](vec3(1, 0, 0), vec3(-1, 0, 0), vec3(0, 1, 0), vec3(0, -1, 0), vec3(0, 0, 1),
                                    vec3(0, 0, -1), vec3(0, 0.7, 0), vec3(0, 0.7, 0));
const float AO_CURVE[4] = float[4](0.40, 0.60, 0.80, 1.0);
${LIGHT}
// UVs come from the position, so merged quads tile their texture (wrap = REPEAT).
vec2 faceUv(uint face, vec3 p) {
  if (face == 0u) return vec2(-p.z, -p.y);
  if (face == 1u) return vec2(p.z, -p.y);
  if (face == 2u) return p.xz;
  if (face == 3u) return vec2(p.x, -p.z);
  if (face == 4u) return vec2(p.x, -p.y);
  if (face == 5u) return vec2(-p.x, -p.y);
  if (face == 6u) return vec2(p.x, -p.y);
  return vec2(p.z, -p.y);
}
void main() {
  vec3 p = vec3(float(a_vert & 63u), float((a_vert >> 6u) & 63u), float((a_vert >> 12u) & 63u));
  uint face = (a_vert >> 18u) & 7u;
  uint ao = (a_vert >> 21u) & 3u;
#ifdef WATER
${WATER_VERT}
#endif

  vec2 uv = faceUv(face, p);

#ifdef WATER
  v_uvl = vec3(uv, u_waterLayer);
  v_light = FACE_SHADE[face];
  v_sun = vec3(0.0);
#else
  uint light = a_light.x;
  uint li = (a_vert >> 23u) | ((a_light.y >> 5u) << 9u);
  v_paintId = a_light.y & 31u;
  v_uvl = vec3(uv, float(li));
  v_light = (face >= 6u ? 0.84 : FACE_SHADE[face]) * AMBIENT * AO_CURVE[ao];
  v_sun = sunLight(FACE_NORMAL[face], light) * AO_CURVE[ao];
  // How this layer may turn, and what the hash in turnUv reads; every vertex of a quad agrees.
  vec3 w = u_worldOrigin + p;
  float plane = face < 2u ? w.x : (face < 4u ? w.y : w.z);
  uint tw = li >> 4u;
  uint how = li < 2048u ? ((u_turns[tw >> 2u][tw & 3u] >> ((li & 15u) * 2u)) & 3u) : 0u;
  v_seasonRole = li < 2048u ? ((u_seasonMask[tw >> 2u][tw & 3u] >> ((li & 15u) * 2u)) & 3u) : 0u;
  v_turn = vec4(faceUv(face, u_worldOrigin), plane * 8.0 + float(face), float(how));
  // Sway sapling tops (face >= 6, ao == 3; torches emit block light 14 in their own cell and stay rigid).
  if (face >= 6u && ao == 3u && (light >> 4u) < 14u) {
    vec2 wdir = u_wind.xy;
    vec2 wperp = vec2(-wdir.y, wdir.x);
    float gust = sin(dot(w.xz, wdir) * 0.28 - u_wind.w) * 0.65
               + sin(dot(w.xz, wdir * 0.7 + wperp * 0.7) * 0.54 - u_wind.w * 1.45) * 0.35;
    vec2 disp = (wdir * (0.18 * u_wind.z * (0.55 + 0.55 * gust)) + wperp * (0.04 * gust * u_wind.z));
    vec3 fRel = u_offset + p;
    if (abs(fRel.y + 1.62) < 1.35) {
      float fd = length(fRel.xz);
      disp += (fRel.xz / max(fd, 0.05)) * smoothstep(0.95, 0.15, fd) * 0.26;
    }
    p.xz += disp;
    p.y -= min(0.12, dot(disp, disp) * 0.35);
  }
#endif
  v_tint = lightTint(a_light.x, u_skyLight);
  v_rel = u_offset + p;
  v_ground = (u_worldOrigin + p).xz;
  v_open = vec2(clamp(float(a_light.x & 15u) - 12.0, 0.0, 3.0) / 3.0, face == 2u ? 1.0 : 0.0);
  gl_Position = u_viewProj * vec4(v_rel, 1.0);
}
`;

export const litFrag = /* glsl */ `#version 300 es
precision highp float;
precision highp int;
precision highp sampler2DArray;

uniform sampler2DArray u_tex;
uniform sampler2DArray u_glow; // self-light 0..1 a texel (art pack): lit as if by itself, whatever the daylight
${FOG}
${HELD_LIGHT}
in vec3 v_uvl;
in float v_light;
in vec3 v_sun;
in vec3 v_tint;
in vec3 v_rel;
out vec4 o_color;

#ifdef BOXES
in float v_haze; // 0 for a beacon, which the fog doesn't hide
in vec4 v_beacon; // a tinted beacon's colour and 1 (a flat light, whatever the texture), else 0
uniform vec4 u_ghost; // a see-through pass (render/boxes.ts drawGhost): tint and opacity; opacity 0 draws solid
uniform vec2 u_xray; // the X-ray pass (drawXray): its radius in blocks (0: not that pass) and where the fade starts, as a share
#endif
#ifdef TERRAIN
in vec2 v_ground;
in vec2 v_open;
flat in uint v_paintId;
flat in uint v_seasonRole;
uniform vec2 u_ground; // wet ground and snow cover, 0..1 (render/precip.ts)
uniform vec3 u_colourPalette[17]; // 0 = unpainted, 1..=16 = surface paint hues (engine colour/mod.rs)
uniform vec4 u_seasonGround[3]; // [0] grass tint, [1] woodland tint, [2] (budding, flowerCover, 0, 0)
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
#ifndef SHAPED
flat in vec4 v_turn;
uint turnHash(ivec3 c) {
  uvec3 u = uvec3(c + 4096);
  uint h = (u.x * 0x8da6b343u) ^ (u.y * 0xd8163841u) ^ (u.z * 0xcb1ab31fu);
  h ^= h >> 16u;
  h *= 0x7feb352du;
  h ^= h >> 15u;
  h *= 0x846ca68bu;
  return h ^ (h >> 16u);
}
// A turning block's tile in one of its orientations, picked by the block's cell; the gradients turn with it so the
// mip level stays the same across block edges.
vec2 turnUv(vec2 uv, inout vec2 dx, inout vec2 dy) {
  if (v_turn.w == 0.0) return uv;
  vec2 cell = floor(uv);
  uint h = turnHash(ivec3(cell + v_turn.xy, v_turn.z));
  mat2 m = mat2((h & 1u) == 1u ? -1.0 : 1.0, 0.0, 0.0, 1.0);
  if (v_turn.w > 2.5) {
    uint r = (h >> 1u) & 3u;
    vec2 cs = r == 0u ? vec2(1, 0) : (r == 1u ? vec2(0, 1) : (r == 2u ? vec2(-1, 0) : vec2(0, -1)));
    m = mat2(cs.x, cs.y, -cs.y, cs.x) * m;
  } else if (v_turn.w > 1.5 && (h & 2u) == 2u) {
    m[1][1] = -1.0;
  }
  dx = m * dx;
  dy = m * dy;
  return cell + 0.5 + m * (uv - cell - 0.5);
}
#endif
#endif
#ifdef SHAPED
// Branches and leaf cards (render/shaped.ts); cards round off and scale with spring buds.
in vec2 v_card;
flat in uint v_leafMeta;
in float v_leafHash;
in float v_bud;
uniform vec4 u_seasonFoliage[30];
float cardMask() {
  if (v_card.x < 0.0) return 1.0;
  vec2 d = (v_card * 2.0 - 1.0) / max(v_bud, 0.35);
  return 1.0 - smoothstep(0.8, 1.0, dot(d, d));
}
#endif
#ifdef WATER
${WATER_FRAG_DECL}
#endif

void main() {
  vec3 tint = max(v_tint, heldGlow(v_rel));
#ifdef WATER
${WATER_FRAG}
#else
  vec3 uvl = v_uvl;
  vec2 dx = dFdx(uvl.xy), dy = dFdy(uvl.xy);
#if defined(TERRAIN) && !defined(SHAPED)
  uvl.xy = turnUv(uvl.xy, dx, dy);
#endif
  vec4 c = textureGrad(u_tex, uvl, dx, dy);
#ifdef SHAPED
  c.a *= cardMask();
#endif
#ifdef CUTOUT
  if (c.a < 0.5) discard;
#endif
#ifdef TERRAIN
  if (v_paintId > 0u && v_paintId < 17u) {
    vec3 paint = u_colourPalette[v_paintId];
    float lum = dot(c.rgb, vec3(0.299, 0.587, 0.114));
    float k = clamp(lum / 0.58, 0.25, 1.45);
    c.rgb = k <= 1.0 ? paint * k : mix(paint, vec3(1.0), clamp((k - 1.0) * 0.55, 0.0, 0.55));
  }
  // Patches of about 16 and 64 blocks, a few percent brighter and warmer or darker and cooler.
  float field = terrainField(v_ground / 16.0, ${TERRAIN_TINT_PERIOD / 16}.0) * 0.65
              + terrainField(v_ground / 64.0, ${TERRAIN_TINT_PERIOD / 64}.0) * 0.35;
  c.rgb *= mix(vec3(0.93, 0.96, 1.0), vec3(1.05, 1.03, 0.95), field);
#ifdef SHAPED
  if ((v_leafMeta & 32u) != 0u) {
    uint sp = v_leafMeta & 15u;
    float lum = dot(c.rgb, vec3(0.299, 0.587, 0.114));
    if ((v_leafMeta & 16u) != 0u) {
      vec3 lc = u_seasonFoliage[20u + sp].rgb;
      if (lc.r >= 0.0) c.rgb = clamp(lc * mix(vec3(0.88, 0.94, 0.96), vec3(1.12, 1.04, 0.92), v_leafHash) * lum, 0.0, 1.0);
    } else {
      vec4 la = u_seasonFoliage[sp], lb = u_seasonFoliage[10u + sp];
      if (la.w > 0.001) {
        vec3 target = mix(la.rgb, lb.rgb, clamp(field * 0.58 + v_leafHash * 0.52 - 0.06, 0.0, 1.0));
        c.rgb = mix(c.rgb, clamp(target * lum, 0.0, 1.0), clamp(la.w + (field - 0.5) * 0.96 * la.w * (1.0 - la.w), 0.0, 1.0));
      }
    }
  }
#endif
  if (v_seasonRole == 1u) {
    c.rgb = mix(c.rgb, clamp(c.rgb * u_seasonGround[0].rgb, 0.0, 1.0), smoothstep(-0.02, 0.08, c.g - c.r * 0.88) * u_seasonGround[0].w);
  } else if (v_seasonRole == 2u) {
    c.rgb = clamp(c.rgb * u_seasonGround[1].rgb, 0.0, 1.0);
  }
  // Rain darkens the ground that sees the sky; snow settles in patches on the faces that look up, thicker as it builds.
  c.rgb *= 1.0 - 0.3 * u_ground.x * v_open.x;
  float cover = smoothstep(field * 0.7, field * 0.7 + 0.3, u_ground.y) * v_open.x * v_open.y;
  c.rgb = mix(c.rgb, vec3(0.9, 0.93, 0.98), cover * 0.92);
#endif
#ifdef BOXES
  vec3 hazed = v_rel * v_haze;
#else
  vec3 hazed = v_rel;
#endif
  float glow = textureGrad(u_glow, uvl, dx, dy).r;
  // A self-lit texel is brighter than any lit one, hottest where its colour is strongest, so a lamp reads as giving
  // light even at noon (its halo, render/halos.ts, does the rest).
  vec3 shown = mix(c.rgb * (v_light * tint + v_sun), c.rgb * 1.2 + c.rgb * c.rgb * 0.75, glow);
#ifdef BOXES
  shown = mix(shown, v_beacon.rgb, v_beacon.a);
#endif
#ifdef TRANSLUCENT
  o_color = vec4(applyFog(shown, hazed), c.a);
#else
  o_color = vec4(applyFog(shown, hazed), 1.0);
#endif
#ifdef BOXES
  if (u_xray.x > 0.0) {
    // X-ray goggles (render/boxes.ts drawXray): what sits behind the ground is unlit, so draw it flat in the texture's
    // own colours, tinted, and fade it out towards the radius so the edge is soft.
    float edge = 1.0 - smoothstep(u_xray.x * u_xray.y, u_xray.x, length(v_rel));
    o_color = vec4(mix(c.rgb * 0.9, u_ghost.rgb, 0.4), u_ghost.a * edge);
  } else if (u_ghost.a > 0.0) o_color = vec4(mix(o_color.rgb, u_ghost.rgb, 0.4), u_ghost.a);
#endif
#endif
}
`;
