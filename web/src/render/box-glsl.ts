// The instanced boxes' vertex shader (boxes.ts draws them with `litFrag` and BOXES). Moved out of shaders.ts, whose
// light functions it shares (`LIGHT`).

import { LIGHT } from './shaders';

// Instanced boxes: dropped items, items on belts and machine parts, in two draw calls: every box from the unit cube,
// then the round bodies (engine `factory::push_round`) from a unit cylinder. Each record is 8 words packed by the
// engine's `box_pack` (its header has the layout; change both together).
export const boxVert = /* glsl */ `#version 300 es
precision highp float;

// Unit cube corner (±0.5) and face index (+X, -X, +Y, -Y, +Z, -Z); or, drawing the cylinder (u_round), a point of a
// unit cylinder and its face: 2 its top, 3 its bottom, 6 plus the share of the way round on its side.
layout(location = 0) in vec4 a_corner;
layout(location = 1) in vec3 a_pos;    // camera- or region-relative centre
layout(location = 2) in uvec4 a_w;     // words 3..6: size (halves) and yaw; pitch, roll and flags; texture layers
layout(location = 3) in uint a_w7;     // light (or a beacon's tint), taper, uv scroll's fraction

uniform mat4 u_viewProj;
uniform vec3 u_offset;
uniform vec4 u_beltScroll;
uniform vec3 u_skyLight;
uniform bool u_round; // this pass draws the round bodies

out vec3 v_uvl;
out float v_light;
out vec3 v_sun;
out vec3 v_tint;
out vec3 v_rel;
out float v_haze;
out vec4 v_beacon;
${LIGHT}
const vec3 NORMALS[6] = vec3[6](vec3(1, 0, 0), vec3(-1, 0, 0), vec3(0, 1, 0), vec3(0, -1, 0), vec3(0, 0, 1), vec3(0, 0, -1));
const float TAU = 6.28318531;
void main() {
  uint flags = a_w.z;
  if (((flags >> 31u) != 0u) != u_round) {
    // The other pass's instance: every vertex at one point outside the view, so nothing is drawn.
    v_uvl = vec3(0.0); v_light = 0.0; v_sun = vec3(0.0); v_tint = vec3(0.0); v_rel = vec3(0.0); v_haze = 0.0;
    v_beacon = vec4(0.0);
    gl_Position = vec4(0.0, 0.0, 2.0, 1.0);
    return;
  }
  vec3 size = vec3(unpackHalf2x16(a_w.x), unpackHalf2x16(a_w.y).x);
  float yaw = float(a_w.y >> 16u) * (TAU / 65536.0);
  float pitch = float(flags & 0x1fffu) * (TAU / 8192.0), rollAngle = float((flags >> 13u) & 0xfffu) * (TAU / 4096.0);
  uint kind = (flags >> 28u) & 3u; // glow: none, glowing, beacon, tinted beacon
  uint payload = a_w7 & 0xfffu;
  float topShare = float((a_w7 >> 12u) & 0xfffu) / 2048.0;
  bool side = a_corner.w > 5.5;
  int face = side ? 6 : int(a_corner.w + 0.5);
  vec3 local = a_corner.xyz * size;
  float taper = mix(1.0, topShare, a_corner.y + 0.5);
  if (u_round) local.xz *= taper; else local.x *= taper;
  // Whole texture on every face (items), or texels at world scale like terrain (machine parts).
  float worldUv = float((flags >> 30u) & 1u);
  uint beltTier = (flags >> 25u) & 7u;
  float beltScroll = beltTier == 1u ? u_beltScroll.x : beltTier == 2u ? u_beltScroll.y : beltTier == 3u ? u_beltScroll.z : beltTier == 4u ? u_beltScroll.w : 0.0;
  float scroll = float(a_w7 >> 24u) / 256.0 + beltScroll;
  vec3 q = mix(a_corner.xyz, local, worldUv) + 0.5;
  vec2 uv;
  // Round once a quarter of the way (a box's four sides), every other copy mirrored so the copies meet without a seam
  // (the mesh has vertices on those quarters); or by the circumference at world scale. The scroll slides a round body's
  // side along its axis (a stem fed through a harvester head), a box's top across it (a belt).
  if (side) {
    float around = (a_corner.w - 6.0) * 4.0, copy = floor(around), along = around - copy;
    uv = vec2(mix(mod(copy, 2.0) < 0.5 ? along : 1.0 - along, around * 0.25 * 3.14159265 * size.x, worldUv), scroll - q.y);
  }
  else if (face == 0) uv = vec2(-q.z, -q.y);
  else if (face == 1) uv = vec2(q.z, -q.y);
  else if (face == 2) uv = vec2(q.x, q.z + (u_round ? 0.0 : scroll));
  else if (face == 3) uv = vec2(q.x, -q.z);
  else if (face == 4) uv = vec2(q.x, -q.y);
  else uv = vec2(-q.x, -q.y);
  uint layers = a_w.w;
  float layer = float(face == 2 ? layers & 1023u : (face == 3 ? layers >> 20u : (layers >> 10u) & 1023u));

  // Yaw turns local -Z towards (sin, 0, -cos), matching the player's look direction.
  float s = sin(yaw), c = cos(yaw);
  float sp = sin(pitch), cp = cos(pitch), sr = sin(rollAngle), cr = cos(rollAngle);
  mat3 tilt = mat3(1.0, 0.0, 0.0, 0.0, cp, sp, 0.0, -sp, cp);
  mat3 roll = mat3(cr, sr, 0.0, -sr, cr, 0.0, 0.0, 0.0, 1.0);
  mat3 rot = mat3(c, 0.0, s, 0.0, 1.0, 0.0, -s, 0.0, c) * roll * tilt;
  // A round body's side leans in by how much narrower its top is than its foot.
  vec3 sideN = vec3(a_corner.x * 2.0 * max(size.y, 1e-4), 0.5 * size.x * (1.0 - topShare), a_corner.z * 2.0 * max(size.y, 1e-4));
  vec3 n = rot * (side ? normalize(sideN) : NORMALS[face]);
  uint light = kind == 3u ? 0u : payload & 255u;
  // A status light, lamp, beam or flame lights itself (factory/render.rs GLOWING); a beacon (2) is also seen through the fog.
  float glow = kind >= 1u ? 1.0 : 0.0;
  v_haze = kind >= 2u ? 0.0 : 1.0;
  // A tinted beacon (a 12-bit colour, factory/render.rs mark_beacon_tinted) is painted that colour, 4 bits a channel.
  v_beacon = kind == 3u ? vec4(float(payload & 15u), float((payload >> 4u) & 15u), float(payload >> 8u), 15.0) / 15.0 : vec4(0.0);
  float shade = n.y > 0.5 ? 1.0 : (n.y < -0.5 ? 0.52 : (abs(n.x) > 0.5 ? 0.72 : 0.86));
  if (u_round) {
    // The box's face shades, blended smoothly round the body.
    float across = mix(0.86, 0.72, n.x * n.x / max(1.0 - n.y * n.y, 1e-4));
    shade = n.y >= 0.0 ? mix(across, 1.0, n.y) : mix(across, 0.52, -n.y);
  }
  v_light = mix(shade * AMBIENT, 1.0, glow);
  v_sun = sunLight(n, light) * (1.0 - glow);
  v_uvl = vec3(uv, layer);
  v_tint = mix(lightTint(light, u_skyLight), vec3(1.0), glow);
  v_rel = u_offset + a_pos + rot * local;
  gl_Position = u_viewProj * vec4(v_rel, 1.0);
}
`;
