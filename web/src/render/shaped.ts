// Branches and leaf cards: the chunk meshes' "shaped" list (engine `mesher/shaped.rs` says how a vertex is packed:
// three u32, position, normal, light and texture together). This file owns their GPU buffers and vertex shader; the
// renderer draws them with the terrain's fragment shader (`litFrag` with SHAPED), the wood after the opaque terrain,
// the leaf cards (from both sides) after the cutout terrain.

import { LIGHT } from './shaders';

/** A chunk's shaped list on the GPU: opaque quads, then cutout, then translucent. */
export interface ShapedPart {
  vao: WebGLVertexArrayObject;
  vbo: WebGLBuffer;
  capacity: number;
  opaque: number;
  cutout: number;
  translucent: number;
}

/** Uploads `words` (a view into wasm memory, used now and dropped) into `part`, made if null; null when empty. */
export function uploadShaped(gl: WebGL2RenderingContext, part: ShapedPart | null, quadIndex: WebGLBuffer,
  words: Uint32Array, opaque: number, cutout: number, translucent = 0): ShapedPart | null {
  if (opaque + cutout + translucent === 0) {
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
    part = { vao, vbo, capacity: 0, opaque: 0, cutout: 0, translucent: 0 };
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
  part.translucent = translucent;
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
uniform uvec4 u_seasonMask[32]; // 2 bits per layer: 0 none, 1 grass, 2 woodland, 3 flower
uniform vec4 u_seasonFoliage[30]; // 0..9 leafA, 10..19 leafB (w=cover), 20..29 litter (w=cover)
uniform vec4 u_seasonGround[3]; // [0] grass, [1] woodland, [2] (budding, flowerCover, 0, 0)
uniform vec4 u_wind; // (dirX, dirZ, strength 0..1+, integrated phase) shared with GrassPass
uniform float u_hideTufts; // 1 when 3D Lush Grass replaces flat GRASS_TUFT cutout sprites

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
flat out uint v_paintId;
flat out uint v_seasonRole;
flat out uint v_leafMeta;
out float v_leafHash;
out float v_bud;

const float SCALE = 100.0; // engine SHAPED_SCALE
const float OFFSET = 2.0; // engine SHAPED_OFFSET
// Species canopy flexibility (0 oak, 1 birch, 2 pine, 3 cherry, 4 palm, 5 baobab, 6 acacia, 7 redwood, 8 eucalyptus, 9 bamboo).
const float SPECIES_FLEX[10] = float[10](0.85, 1.15, 0.55, 1.10, 1.45, 0.70, 1.05, 0.50, 1.15, 1.35);
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
  uint li = (w2 >> 16u) & 2047u;
  float layer = float(li);
  uint sw = li >> 4u;
  v_seasonRole = li < 2048u ? ((u_seasonMask[sw >> 2u][sw & 3u] >> ((li & 15u) * 2u)) & 3u) : 0u;
  v_paintId = kind == 0u ? 0u : (w2 >> 27u);
  v_leafMeta = 0u;
  v_leafHash = 0.0;
  v_bud = 1.0;
  bool hidden = false;
  vec2 uv = vec2(0.0);
  v_card = vec2(-1.0);
  if (kind == 0u) {
    // A leaf card: its corner, which part of the texture, and the blocks the texture spans (engine foliage.rs).
    uint b = (w1 >> 12u) & 4095u;
    vec2 corner = vec2(float(b & 1u), float((b >> 1u) & 3u) * 0.5);
    uint part = (b >> 3u) & 15u;
    uv = corner * (float(b >> 7u) / 8.0) + vec2(float(part & 3u) * 0.37, float(part >> 2u) * 0.29);
    v_card = corner;
    uint meta = w2 >> 27u;
    uint sp = min(meta & 15u, 9u);
    bool isLitter = (meta & 16u) != 0u;
    uint dropBits = ((w0 >> 30u) << 4u) | part;
    float cardHash = (float(((dropBits * 45u + 13u) & 63u)) + 0.5) / 64.0;
    float cover = isLitter ? u_seasonFoliage[20u + sp].w : u_seasonFoliage[10u + sp].w;
    if (cardHash >= cover) {
      hidden = true;
    } else if (!isLitter && cover < 0.999 && u_seasonGround[2].x > 0.01) {
      float age = clamp((cover - cardHash) / 0.22, 0.0, 1.0);
      v_bud = mix(1.0, mix(0.42, 1.0, age), u_seasonGround[2].x);
    }
    v_leafMeta = sp | (isLitter ? 16u : 0u) | 32u;
    v_leafHash = cardHash;
    // Tree canopy wind sway & species-tuned leaf flutter (fallen ground litter stays pinned).
    if (!hidden && !isLitter && u_wind.z > 0.001) {
      vec3 wp = u_worldOrigin + p;
      vec2 wdir = u_wind.xy;
      vec2 wperp = vec2(-wdir.y, wdir.x);
      float flex = SPECIES_FLEX[sp];
      // Travelling downwind canopy gust wave + brisk per-card leaf flutter (scaled by outer corner so bases stay anchored).
      float gust = sin(dot(wp.xz, wdir) * 0.16 - u_wind.w * 0.65 + wp.y * 0.07) * 0.65
                 + sin(dot(wp.xz, wdir * 0.6 + wperp * 0.8) * 0.31 - u_wind.w * 1.15 + cardHash * 4.5) * 0.35;
      float flutter = sin(u_wind.w * 2.9 + cardHash * 6.28318 + dot(wp, vec3(0.7, 0.4, 0.5))) * (0.35 + 0.65 * corner.y);
      float wAmp = u_wind.z * flex;
      vec2 swayXZ = wdir * (0.14 * wAmp * (0.45 + 0.55 * gust))
                  + wperp * (0.045 * wAmp * gust + 0.028 * wAmp * flutter);
      float swayY = (0.022 * wAmp * flutter - 0.03 * wAmp * max(0.0, gust)) * (0.4 + 0.6 * corner.x);
      p.xz += swayXZ;
      p.y += swayY;
    }
  } else if (kind >= 1u && kind <= 3u) {
    uint a = kind - 1u;
    uv = vec2(u, -p[a]); // around, then along the trunk
  } else if (kind >= 4u && kind <= 6u) {
    uint a = kind - 4u;
    uv = vec2(p[(a + 1u) % 3u], p[(a + 2u) % 3u]);
  } else if (kind == 7u) {
    // Ground cover (engine mesher/cover/shape.rs): 6-bit u and v in 60ths, drawn whole without the leaf-card mask.
    uint b = (w1 >> 12u) & 4095u;
    uv = vec2(float(b & 63u), float((b >> 6u) & 63u)) / 60.0;
    if (u_hideTufts > 0.5 && v_seasonRole == 1u) {
      hidden = true;
    } else if (v_seasonRole == 3u && u_seasonGround[2].y < 0.999) {
      ivec3 fc = ivec3(floor(u_worldOrigin + p));
      uint fh = uint(fc.x * 374761393 + fc.z * 668265263 + fc.y * 1274126177);
      fh = (fh ^ (fh >> 13u)) * 1274126177u;
      if (float((fh ^ (fh >> 16u)) & 1023u) / 1024.0 >= u_seasonGround[2].y) hidden = true;
    }
    if (!hidden) {
      // Dynamic wind sway & player foot parting matched to GrassPass; roots (h = 0) and rock lichen (h < 0.03) stay pinned.
      float h = clamp(p.y - floor(p.y + 0.002), 0.0, 1.0);
      float flex = h * h * smoothstep(0.03, 0.10, h);
      if (flex > 0.0005) {
        vec2 wxz = u_worldOrigin.xz + p.xz;
        vec2 cellXZ = floor(wxz);
        float cHash = fract(sin(dot(cellXZ, vec2(127.1, 311.7))) * 43758.5453);
        vec2 wdir = u_wind.xy;
        vec2 wperp = vec2(-wdir.y, wdir.x);
        float gust = sin(dot(wxz, wdir) * 0.28 - u_wind.w) * 0.65
                   + sin(dot(wxz, wdir * 0.7 + wperp * 0.7) * 0.54 - u_wind.w * 1.45 + cHash * 4.0) * 0.35;
        float flutter = sin(u_wind.w * 2.3 + cHash * 6.28318) * 0.035 * (0.4 + u_wind.z);
        // Woodland ferns & reeds flex slightly fuller than stiffer woody brush/shrubs.
        float supple = v_seasonRole == 0u ? 0.55 : 1.0;
        float wAmp = u_wind.z * (0.55 + 0.55 * gust) * supple;
        vec2 dispXZ = (wdir * (0.34 * wAmp) + wperp * (0.07 * gust * u_wind.z * supple + flutter)) * flex;
        vec3 footRel = u_offset + p;
        if (abs(footRel.y + 1.62) < 1.35) {
          float footDist = length(footRel.xz);
          float push = smoothstep(0.95, 0.15, footDist) * 0.42 * smoothstep(0.03, 0.25, h);
          dispXZ += (footRel.xz / max(footDist, 0.05)) * push;
        }
        p.xz += dispXZ;
        p.y -= min(h * 0.32, dot(dispXZ, dispXZ) * 0.45);
      }
    }
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
  if (kind == 7u && !hidden) {
    float sky = pow(0.8, 15.0 - float(light & 15u));
    float h = clamp(p.y - floor(p.y + 0.002), 0.0, 1.0);
    float backLit = pow(max(dot(normalize(v_rel), u_sunDir), 0.0), 3.0);
    v_sun += u_direct * (backLit * smoothstep(0.04, 0.35, h) * sky * 0.28);
  }
  v_ground = (u_worldOrigin + p).xz;
  v_open = vec2(clamp(float(light & 15u) - 12.0, 0.0, 3.0) / 3.0, smoothstep(0.45, 0.8, n.y));
  gl_Position = hidden ? vec4(2.0, 2.0, 2.0, 1.0) : u_viewProj * vec4(v_rel, 1.0);
}
`;
