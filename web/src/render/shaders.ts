// GLSL ES 3.00 sources. `CUTOUT` is defined for the alpha-tested (leaves) variant only, so the
// opaque pass never contains `discard` and keeps early-z.

const FOG = /* glsl */ `
uniform vec3 u_fogColor;
uniform vec2 u_fog; // start, end (world units from the camera)
vec3 applyFog(vec3 col, vec3 rel) {
  float f = clamp((length(rel) - u_fog.x) / (u_fog.y - u_fog.x), 0.0, 1.0);
  return mix(col, u_fogColor, f * f);
}
`;

export const chunkVert = /* glsl */ `#version 300 es
precision highp float;
precision highp int;

layout(location = 0) in uint a_vert;

uniform mat4 u_viewProj;
uniform vec3 u_offset; // chunk origin minus camera position

out vec3 v_uvl;
out float v_light;
out vec3 v_rel;

const float FACE_SHADE[6] = float[6](0.72, 0.72, 1.0, 0.52, 0.86, 0.86);
const float AO_CURVE[4] = float[4](0.40, 0.60, 0.80, 1.0);

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
  else uv = vec2(-p.x, -p.y);

  v_uvl = vec3(uv, layer);
  v_light = FACE_SHADE[face] * AO_CURVE[ao];
  v_rel = u_offset + p;
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
in vec3 v_rel;
out vec4 o_color;

void main() {
  vec4 c = texture(u_tex, v_uvl);
#ifdef CUTOUT
  if (c.a < 0.5) discard;
#endif
  o_color = vec4(applyFog(c.rgb * v_light, v_rel), 1.0);
}
`;

// Instanced boxes: dropped items, items on belts and machine parts, all in one draw call.
// Instance layout matches `factory::INSTANCE_FLOATS` in the engine.
export const boxVert = /* glsl */ `#version 300 es
precision highp float;

layout(location = 0) in vec4 a_corner; // unit cube corner (±0.5) and face index (+X, -X, +Y, -Y, +Z, -Z)
layout(location = 1) in vec4 a_i0;     // camera-relative centre, yaw
layout(location = 2) in vec4 a_i1;     // size, uv scroll (top face)
layout(location = 3) in vec4 a_i2;     // texture layer top, side, bottom; uv mode (0 whole texture, 1 world-scaled)

uniform mat4 u_viewProj;

out vec3 v_uvl;
out float v_light;
out vec3 v_rel;

const vec3 NORMALS[6] = vec3[6](vec3(1, 0, 0), vec3(-1, 0, 0), vec3(0, 1, 0), vec3(0, -1, 0), vec3(0, 0, 1), vec3(0, 0, -1));

void main() {
  int face = int(a_corner.w + 0.5);
  vec3 local = a_corner.xyz * a_i1.xyz;
  // Whole texture on every face (items), or texels at world scale like terrain (machine parts).
  vec3 q = mix(a_corner.xyz, local, a_i2.w) + 0.5;
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
  mat3 rot = mat3(c, 0.0, s, 0.0, 1.0, 0.0, -s, 0.0, c);
  vec3 n = rot * NORMALS[face];
  v_light = n.y > 0.5 ? 1.0 : (n.y < -0.5 ? 0.52 : (abs(n.x) > 0.5 ? 0.72 : 0.86));
  v_uvl = vec3(uv, layer);
  v_rel = a_i0.xyz + rot * local;
  gl_Position = u_viewProj * vec4(v_rel, 1.0);
}
`;

export const lineVert = /* glsl */ `#version 300 es
precision highp float;
layout(location = 0) in vec3 a_pos;
uniform mat4 u_viewProj;
uniform vec3 u_offset;
void main() {
  gl_Position = u_viewProj * vec4(u_offset + a_pos * 1.004 - 0.002, 1.0);
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
