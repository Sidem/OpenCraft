// The instanced boxes' vertex shader (boxes.ts draws them with `litFrag` and BOXES). Moved out of shaders.ts, whose
// light functions it shares (`LIGHT`).

import { LIGHT } from './shaders';

// Instanced boxes: dropped items, items on belts and machine parts, in two draw calls: every box from the unit cube,
// then the round bodies (the ROUND_BIT mode bit, engine `factory::push_round`) from a unit cylinder; each pass drops
// the other's instances. Instance layout matches `factory::INSTANCE_FLOATS` in the engine.
export const boxVert = /* glsl */ `#version 300 es
precision highp float;

// Unit cube corner (±0.5) and face index (+X, -X, +Y, -Y, +Z, -Z); or, drawing the cylinder (u_round), a point of a
// unit cylinder and its face: 2 its top, 3 its bottom, 6 plus the share of the way round on its side.
layout(location = 0) in vec4 a_corner;
layout(location = 1) in vec4 a_i0;     // camera-relative centre, yaw
layout(location = 2) in vec4 a_i1;     // size, uv scroll (a box's top face, a round body's side)
layout(location = 3) in vec4 a_i2;     // texture layer top, side, bottom; mode bits (1 world-scaled uv, ROUND_BIT) + 4 * light
layout(location = 4) in vec4 a_i3;     // pitch, roll, top-width taper (a round body's top-radius share), glow (0 or 1)

uniform mat4 u_viewProj;
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

void main() {
  uint mode = uint(a_i2.w + 0.5);
  if (((mode & ROUND_BIT) != 0u) != u_round) {
    // The other pass's instance: every vertex at one point outside the view, so nothing is drawn.
    v_uvl = vec3(0.0); v_light = 0.0; v_sun = vec3(0.0); v_tint = vec3(0.0); v_rel = vec3(0.0); v_haze = 0.0;
    v_beacon = vec4(0.0);
    gl_Position = vec4(0.0, 0.0, 2.0, 1.0);
    return;
  }
  bool side = a_corner.w > 5.5;
  int face = side ? 6 : int(a_corner.w + 0.5);
  vec3 local = a_corner.xyz * a_i1.xyz;
  float taper = mix(1.0, a_i3.z, a_corner.y + 0.5);
  if (u_round) local.xz *= taper; else local.x *= taper;
  // Whole texture on every face (items), or texels at world scale like terrain (machine parts).
  float worldUv = float(mode & 1u);
  vec3 q = mix(a_corner.xyz, local, worldUv) + 0.5;
  vec2 uv;
  // Round once a quarter of the way (a box's four sides), every other copy mirrored so the copies meet without a seam
  // (the mesh has vertices on those quarters); or by the circumference at world scale. The scroll slides a round body's
  // side along its axis (a stem fed through a harvester head), a box's top across it (a belt).
  if (side) {
    float around = (a_corner.w - 6.0) * 4.0, copy = floor(around), along = around - copy;
    uv = vec2(mix(mod(copy, 2.0) < 0.5 ? along : 1.0 - along, around * 0.25 * 3.14159265 * a_i1.x, worldUv), a_i1.w - q.y);
  }
  else if (face == 0) uv = vec2(-q.z, -q.y);
  else if (face == 1) uv = vec2(q.z, -q.y);
  else if (face == 2) uv = vec2(q.x, q.z + (u_round ? 0.0 : a_i1.w));
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
  // A round body's side leans in by how much narrower its top is than its foot.
  vec3 sideN = vec3(a_corner.x * 2.0 * max(a_i1.y, 1e-4), 0.5 * a_i1.x * (1.0 - a_i3.z), a_corner.z * 2.0 * max(a_i1.y, 1e-4));
  vec3 n = rot * (side ? normalize(sideN) : NORMALS[face]);
  uint light = mode >> 2u;
  // A status light, lamp, beam or flame lights itself (factory/render.rs GLOWING); a beacon (2) is also seen through the fog.
  float glow = min(a_i3.w, 1.0);
  v_haze = a_i3.w > 1.5 ? 0.0 : 1.0;
  // A tinted beacon (3 + a 12-bit colour, factory/render.rs mark_beacon_tinted) is painted that colour, 4 bits a channel.
  float tint = a_i3.w - 3.0;
  v_beacon = tint < 0.0 ? vec4(0.0) : vec4(mod(tint, 16.0), mod(floor(tint / 16.0), 16.0), floor(tint / 256.0), 15.0) / 15.0;
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
  v_rel = a_i0.xyz + rot * local;
  gl_Position = u_viewProj * vec4(v_rel, 1.0);
}
`;
