// Branches and leaf cards: the chunk meshes' "shaped" list (engine `mesher/shaped.rs` says how a vertex is packed:
// three u32, position, normal, light and texture together). This file owns their GPU buffers and vertex shader; the
// renderer draws them with the terrain's fragment shader (`litFrag` with SHAPED), the wood after the opaque terrain,
// the leaf cards (from both sides) after the cutout terrain.

import { LIGHT } from './shaders';

/** A chunk's shaped list on the GPU: opaque quads, then cutout ones. */
export interface ShapedPart {
  vao: WebGLVertexArrayObject;
  vbo: WebGLBuffer;
  capacity: number;
  opaque: number;
  cutout: number;
}

/** Uploads `words` (a view into wasm memory, used now and dropped) into `part`, made if null; null when empty. */
export function uploadShaped(gl: WebGL2RenderingContext, part: ShapedPart | null, quadIndex: WebGLBuffer,
  words: Uint32Array, opaque: number, cutout: number): ShapedPart | null {
  if (opaque + cutout === 0) {
    if (part) deleteShaped(gl, part);
    return null;
  }
  if (!part) {
    const vao = gl.createVertexArray()!;
    const vbo = gl.createBuffer()!;
    gl.bindVertexArray(vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribIPointer(0, 3, gl.UNSIGNED_INT, 12, 0);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, quadIndex);
    gl.bindVertexArray(null);
    part = { vao, vbo, capacity: 0, opaque: 0, cutout: 0 };
  }
  gl.bindBuffer(gl.ARRAY_BUFFER, part.vbo);
  if (words.byteLength > part.capacity) {
    gl.bufferData(gl.ARRAY_BUFFER, words, gl.STATIC_DRAW);
    part.capacity = words.byteLength;
  } else {
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, words);
  }
  part.opaque = opaque;
  part.cutout = cutout;
  return part;
}

export function deleteShaped(gl: WebGL2RenderingContext, part: ShapedPart): void {
  gl.deleteVertexArray(part.vao);
  gl.deleteBuffer(part.vbo);
}

// The outputs match chunkVert's (shaders.ts), so `litFrag` serves both; SHAPED adds the chunk-local position, the
// kind and a card's corner, from which the fragment shader rounds a leaf card's outline.
export const shapedVert = /* glsl */ `#version 300 es
precision highp float;
precision highp int;

layout(location = 0) in uvec3 a_vert;

uniform mat4 u_viewProj;
uniform vec3 u_offset; // chunk origin minus camera position
uniform vec3 u_worldOrigin; // chunk origin modulo the tint's period
uniform vec3 u_skyLight;
uniform vec3 u_pivot; // a falling tree: the pivot in the part's local space (falling.ts)
uniform mat3 u_turn; // and how far it has tipped; the identity for chunks

out vec3 v_uvl;
out float v_light;
out vec3 v_sun;
out vec3 v_tint;
out vec3 v_rel;
out vec2 v_ground;
out vec2 v_open;
out vec3 v_pos;
out vec2 v_card;
flat out uint v_kind;

const float SCALE = 100.0; // engine SHAPED_SCALE
const float OFFSET = 2.0; // engine SHAPED_OFFSET
${LIGHT}
vec3 octahedral(uint bits) {
  vec2 e = vec2(float(bits & 255u), float(bits >> 8u)) / 255.0 * 2.0 - 1.0;
  vec3 n = vec3(e.x, 1.0 - abs(e.x) - abs(e.y), e.y);
  if (n.y < 0.0) n.xz = (1.0 - abs(n.zx)) * vec2(n.x >= 0.0 ? 1.0 : -1.0, n.z >= 0.0 ? 1.0 : -1.0);
  return normalize(n);
}
void main() {
  uint w0 = a_vert.x, w1 = a_vert.y, w2 = a_vert.z;
  vec3 p = vec3(float(w0 & 4095u), float((w0 >> 12u) & 4095u), float(w1 & 4095u)) / SCALE - OFFSET;
  uint kind = (w0 >> 24u) & 7u;
  float ao = mix(0.4, 1.0, float((w0 >> 27u) & 7u) / 7.0);
  float u = float((w1 >> 12u) & 4095u) / 256.0;
  uint light = w1 >> 24u;
  vec3 n = u_turn * octahedral(w2 & 65535u);
  float layer = float((w2 >> 16u) & 511u);
  vec2 uv = vec2(0.0);
  v_card = vec2(-1.0);
  if (kind == 0u) {
    // A leaf card: its corner, which part of the texture, and the blocks the texture spans (engine foliage.rs).
    uint b = (w1 >> 12u) & 4095u;
    vec2 corner = vec2(float(b & 1u), float((b >> 1u) & 3u) * 0.5);
    uint part = (b >> 3u) & 15u;
    uv = corner * (float(b >> 7u) / 8.0) + vec2(float(part & 3u) * 0.37, float(part >> 2u) * 0.29);
    v_card = corner;
  } else if (kind >= 1u && kind <= 3u) {
    uint a = kind - 1u;
    uv = vec2(u, -p[a]); // around, then along the trunk
  } else if (kind >= 4u && kind <= 6u) {
    uint a = kind - 4u;
    uv = vec2(p[(a + 1u) % 3u], p[(a + 2u) % 3u]);
  }
  v_uvl = vec3(uv, layer);
  // The cubes' face shades (top 1.0, sides about 0.8, bottom 0.52), blended by the normal.
  float shade = n.y >= 0.0 ? mix(0.79, 1.0, n.y) : mix(0.79, 0.52, -n.y);
  v_light = shade * AMBIENT * ao;
  v_sun = sunLight(n, light) * ao;
  v_tint = lightTint(light, u_skyLight);
  v_pos = p;
  v_kind = kind;
  v_rel = u_offset + u_pivot + u_turn * (p - u_pivot);
  v_ground = (u_worldOrigin + p).xz;
  v_open = vec2(clamp(float(light & 15u) - 12.0, 0.0, 3.0) / 3.0, smoothstep(0.45, 0.8, n.y));
  gl_Position = u_viewProj * vec4(v_rel, 1.0);
}
`;
