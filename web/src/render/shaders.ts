// GLSL ES 3.00 sources. `CUTOUT` is defined for the alpha-tested (leaves) variant only, so the
// opaque pass never contains `discard` and keeps early-z. `TERRAIN` (chunk meshes only) adds a
// gentle world-anchored shift in tone across blocks; texels are always sampled exactly as drawn, so
// the pixel art stays crisp. Repetition is broken by alternates per block (the mesher) and, for natural layers,
// by turning each block's tile in place (`turnUv`, engine `block/turn.rs`), which keeps greedy quads merged.
// `WATER` is the translucent liquid variant (render/water.ts): no AO and no texture layer (a liquid vertex carries
// its surface drop, the kind of water and its flow direction instead: engine `mesher/water.rs`, which says how
// they are packed). The kind sets the look: still water shimmers slowly and glints in the sun, a laminar flow
// streaks along its direction, rapids churn and foam, a fall streaks downwards in white. Thin water is clearer.

export const TERRAIN_TINT_PERIOD = 256;
/** The texture unit of the self-light array (`u_glow`); 1 and 2 belong to the rain mask and the clouds. */
export const GLOW_UNIT = 4;
/** Held lights the shader can show at once: one a player (the engine caps them at its `MAX_PLAYERS`). */
export const HELD_LIGHT_SLOTS = 4;

const FOG = /* glsl */ `
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
const HELD_LIGHT = /* glsl */ `
${LAMP_COLOUR}
const int HELD_LIGHTS = ${HELD_LIGHT_SLOTS};
uniform vec4 u_held[HELD_LIGHTS]; // camera-relative x, y, z and the reach in blocks (0: no light)
vec3 heldGlow(vec3 rel) {
  float g = 0.0;
  for (int i = 0; i < HELD_LIGHTS; i++) {
    float f = clamp((1.0 - length(rel - u_held[i].xyz) / max(u_held[i].w, 0.001)) * 1.6, 0.0, 1.0);
    g = max(g, f * f * step(0.001, u_held[i].w));
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
const LIGHT = /* glsl */ `
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
out vec2 v_open; // how much open sky this vertex has (0..1) and 1 on an upward face: where rain wets and snow lies
#ifndef WATER
uniform uint u_turns[32]; // two bits per layer: how its blocks may turn (engine block/turn.rs)
// The face's offset in world tile cells (xy), its plane and side (z), and how it may turn (w): 0 not, 1 mirror,
// 2 mirror and flip upside down, 3 any of eight ways.
flat out vec4 v_turn;
#else
uniform float u_waterLayer; // the one tile every liquid vertex shares
flat out uvec3 v_water; // kind of water, flow direction (eighths of a turn from +X to +Z), face
out float v_depth; // the surface's height above the block's floor (0..1) on a top face
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
  float layer = float(a_vert >> 23u);
#ifdef WATER
  float drop = float(ao | (((a_vert >> 28u) & 3u) << 2u)) / 16.0;
  p.y -= drop;
  layer = u_waterLayer;
  uint info = (a_vert >> 23u) & 31u;
  v_water = uvec3(info & 3u, info >> 2u, face);
  v_depth = face == 2u ? 1.0 - drop : 1.0;
#endif

  vec2 uv = faceUv(face, p);

#ifdef WATER
  v_uvl = vec3(uv, layer);
  v_light = FACE_SHADE[face];
  v_sun = vec3(0.0);
#else
  v_uvl = vec3(uv, layer);
  v_light = (face >= 6u ? 0.84 : FACE_SHADE[face]) * AMBIENT * AO_CURVE[ao];
  v_sun = sunLight(FACE_NORMAL[face], a_light) * AO_CURVE[ao];
  // How this layer may turn, and what the hash in turnUv reads; every vertex of a quad agrees.
  uint li = a_vert >> 23u;
  vec3 w = u_worldOrigin + p;
  float plane = face < 2u ? w.x : (face < 4u ? w.y : w.z);
  uint how = (u_turns[li >> 4u] >> ((li & 15u) * 2u)) & 3u;
  v_turn = vec4(faceUv(face, u_worldOrigin), plane * 8.0 + float(face), float(how));
#endif
  v_tint = lightTint(a_light, u_skyLight);
  v_rel = u_offset + p;
  v_ground = (u_worldOrigin + p).xz;
  v_open = vec2(clamp(float(a_light & 15u) - 12.0, 0.0, 3.0) / 3.0, face == 2u ? 1.0 : 0.0);
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
#endif
#ifdef TERRAIN
in vec2 v_ground;
in vec2 v_open;
uniform vec2 u_ground; // wet ground and snow cover, 0..1 (render/precip.ts)
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
#ifdef WATER
uniform float u_time; // seconds
uniform vec3 u_sunDir;
uniform vec3 u_direct;
flat in uvec3 v_water; // kind (0 still, 1 laminar, 2 turbulent, 3 falling), direction, face
in float v_depth;
const vec2 FLOW_DIR[8] = vec2[8](vec2(1, 0), vec2(0.7071, 0.7071), vec2(0, 1), vec2(-0.7071, 0.7071), vec2(-1, 0),
                                 vec2(-0.7071, -0.7071), vec2(0, -1), vec2(0.7071, -0.7071));
const vec3 FOAM = vec3(0.86, 0.97, 0.98);
float waterHash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}
float waterNoise(vec2 p) {
  vec2 cell = floor(p), f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  return mix(mix(waterHash(cell), waterHash(cell + vec2(1, 0)), f.x), mix(waterHash(cell + vec2(0, 1)), waterHash(cell + vec2(1, 1)), f.x), f.y);
}
#endif

void main() {
  vec3 tint = max(v_tint, heldGlow(v_rel));
#ifdef WATER
  float t = u_time;
  vec2 uv = v_uvl.xy;
  float layer = v_uvl.z;
  bool top = v_water.z == 2u;
  uint kind = v_water.x;
  if (!top && kind == 2u) kind = 3u; // the wall of a drop is a fall
  vec2 d = FLOW_DIR[v_water.y];
  vec2 across = vec2(-d.y, d.x);
  vec4 a, b;
  float foam = 0.0, ripple = 0.9, shine = 1.0;
  if (kind == 0u) {
    // Still water: two copies of the texture drifting slowly apart (one mirrored, half a block off) shimmer; calm, glassy.
    a = texture(u_tex, vec3(uv + vec2(t * 0.07, t * 0.04), layer));
    b = texture(u_tex, vec3(0.5 - uv.x + t * 0.04, uv.y + 0.37 - t * 0.06, layer));
    ripple = 0.5;
    shine = 1.4;
  } else if (kind == 1u) {
    // A laminar flow: the texture stretched along the current and carried with it, in two layers a little apart in speed.
    vec2 s = vec2(dot(uv, across), dot(uv, d) * 0.4);
    a = texture(u_tex, vec3(s + vec2(0.0, -t * 0.22), layer));
    b = texture(u_tex, vec3(vec2(0.31 - s.x * 1.3, s.y * 1.3 + 0.2 - t * 0.37), layer));
    ripple = 0.7;
    foam = smoothstep(0.78, 0.95, waterNoise(s * vec2(7.0, 3.0) - vec2(0.0, t * 0.9))) * 0.35;
  } else if (kind == 2u) {
    // Rapids: two currents running through each other, broken up, with white water on the crests.
    vec2 s = vec2(dot(uv, across), dot(uv, d) * 0.6);
    a = texture(u_tex, vec3(s * 1.2 + vec2(0.0, -t * 0.9), layer));
    b = texture(u_tex, vec3(vec2(0.37 - s.x * 1.7, s.y * 1.7 + 0.2 - t * 0.6), layer));
    ripple = 1.4;
    float churn = waterNoise(uv * 5.0 + d * t * 1.4) * 0.55 + (a.g + b.g) * 0.4;
    foam = smoothstep(0.62, 0.95, churn) * 0.9;
  } else {
    // A fall: narrow streaks running down, white where they bunch.
    vec2 s = vec2(uv.x * 1.6 + uv.y * 0.08, uv.y * 0.35 - t * 1.5);
    a = texture(u_tex, vec3(s, layer));
    b = texture(u_tex, vec3(vec2(s.x * 1.9 + 0.3, s.y * 1.6 + 0.4 - t * 0.5), layer));
    ripple = 0.3;
    foam = smoothstep(0.35, 0.8, waterNoise(vec2(uv.x * 7.0, uv.y * 1.2 - t * 3.0)) * 0.7 + (a.g + b.g) * 0.3) * 0.8;
  }
  vec4 c = mix(a, b, 0.5);
  // The sun (the moon at night) glints on open still or gently flowing water: its reflection off a surface the two copies ripple.
  vec3 n = normalize(vec3((a.r - b.r) * ripple, 1.0, (a.g - b.g) * ripple));
  float glint = pow(max(dot(reflect(normalize(v_rel), n), u_sunDir), 0.0), 160.0)
              * step(0.95, v_light) * clamp(v_tint.g * 1.5 - 0.2, 0.0, 1.0) * (top && kind < 2u ? shine : 0.0);
  vec3 col = c.rgb * v_light * tint + u_direct * glint * 1.6;
  col = mix(col, FOAM * (0.35 + 0.65 * v_light) * tint, foam);
  // Thin water is clearer, so the ground shows through the tail of a flow; falling water is the most solid.
  float alpha = c.a * mix(0.45, 1.0, smoothstep(0.0, 0.5, v_depth));
  alpha = max(kind == 3u ? max(alpha, 0.7) : alpha, foam * 0.9);
  o_color = vec4(applyFog(col, v_rel), max(alpha, min(glint, 1.0)));
#else
  vec3 uvl = v_uvl;
  vec2 dx = dFdx(uvl.xy), dy = dFdy(uvl.xy);
#ifdef TERRAIN
  uvl.xy = turnUv(uvl.xy, dx, dy);
#endif
  vec4 c = textureGrad(u_tex, uvl, dx, dy);
#ifdef CUTOUT
  if (c.a < 0.5) discard;
#endif
#ifdef TERRAIN
  // Patches of about 16 and 64 blocks, a few percent brighter and warmer or darker and cooler.
  float field = terrainField(v_ground / 16.0, ${TERRAIN_TINT_PERIOD / 16}.0) * 0.65
              + terrainField(v_ground / 64.0, ${TERRAIN_TINT_PERIOD / 64}.0) * 0.35;
  c.rgb *= mix(vec3(0.93, 0.96, 1.0), vec3(1.05, 1.03, 0.95), field);
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
  vec3 shown = mix(c.rgb * (v_light * tint + v_sun), c.rgb * 1.1, glow);
#ifdef BOXES
  shown = mix(shown, v_beacon.rgb, v_beacon.a);
#endif
  o_color = vec4(applyFog(shown, hazed), 1.0);
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
out float v_haze;
out vec4 v_beacon;
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
  // A status light, lamp, beam or flame lights itself (factory/render.rs GLOWING); a beacon (2) is also seen through the fog.
  float glow = min(a_i3.w, 1.0);
  v_haze = a_i3.w > 1.5 ? 0.0 : 1.0;
  // A tinted beacon (3 + a 12-bit colour, factory/render.rs mark_beacon_tinted) is painted that colour, 4 bits a channel.
  float tint = a_i3.w - 3.0;
  v_beacon = tint < 0.0 ? vec4(0.0) : vec4(mod(tint, 16.0), mod(floor(tint / 16.0), 16.0), floor(tint / 256.0), 15.0) / 15.0;
  v_light = mix((n.y > 0.5 ? 1.0 : (n.y < -0.5 ? 0.52 : (abs(n.x) > 0.5 ? 0.72 : 0.86))) * AMBIENT, 1.0, glow);
  v_sun = sunLight(n, light) * (1.0 - glow);
  v_uvl = vec3(uv, layer);
  v_tint = mix(lightTint(light, u_skyLight), vec3(1.0), glow);
  v_rel = a_i0.xyz + rot * local;
  gl_Position = u_viewProj * vec4(v_rel, 1.0);
}
`;
