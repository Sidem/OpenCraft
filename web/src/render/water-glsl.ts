// The `WATER` variant's GLSL, spliced into shaders.ts's chunk shaders (render/water.ts draws it). A liquid vertex carries
// its surface drop, the kind of water, its flow direction and its ice instead of AO and a texture layer (engine
// `mesher/water.rs` says how they are packed). The kind sets the look: still water shimmers slowly and glints in the
// sun, a laminar flow streaks along its direction, rapids churn and foam, a fall streaks downwards in white; thin
// water is clearer. Ice (engine `weather/ice.rs`) is looked up per vertex in the engine's table and drawn over it.
// To change a look: the matching branch of `WATER_FRAG`.

/** `uvec4`s holding the engine's ice table, four levels a word (`weather/ice.rs`: `KINDS` × `COLD_LEVELS` bytes). */
export const ICE_WORDS = 12;

/** Vertex declarations (in place of the terrain's texture-layer outputs). */
export const WATER_VERT_DECL = /* glsl */ `
uniform float u_waterLayer; // the one tile every liquid vertex shares
// Ice: the table's levels four to a word (rows lake, flowing, sea), and sea level, cold indices a row, the level that
// bears a body (0..1).
uniform uvec4 u_ice[${ICE_WORDS}];
uniform vec4 u_iceShape;
flat out float v_ice; // the ice on this surface, 0 open water to 1 the thickest
flat out uvec3 v_water; // kind of water, flow direction (eighths of a turn from +X to +Z), face
out float v_depth; // the surface's height above the block's floor (0..1) on a top face
`;

/** Vertex main, after `p`, `face` and `ao` are read: the surface drop, the ice, kind and flow. */
export const WATER_VERT = /* glsl */ `
  float drop = float(ao | (((a_vert >> 28u) & 3u) << 2u)) / 16.0;
  // A top face's cold index and flowing bit (mesher/water.rs, fn ice); still water at or below sea level is sea.
  uint code = a_light.y & 127u, levels = uint(u_iceShape.y), cold = code & 63u;
  uint row = (code & 64u) != 0u ? 1u : (u_worldOrigin.y + p.y - 1.0 <= u_iceShape.x ? 2u : 0u);
  uint at = row * levels + cold, word = at >> 2u;
  v_ice = face == 2u && cold < levels - 1u ? float((u_ice[word >> 2u][word & 3u] >> ((at & 3u) * 8u)) & 255u) / 255.0 : 0.0;
  p.y -= drop;
  uint info = (a_vert >> 23u) & 31u;
  v_water = uvec3(info & 3u, info >> 2u, face);
  v_depth = face == 2u ? 1.0 - drop : 1.0;
`;

/** Fragment declarations: the light, the water's own inputs and its noise. */
export const WATER_FRAG_DECL = /* glsl */ `
uniform float u_time; // seconds
uniform vec3 u_sunDir;
uniform vec3 u_direct;
uniform vec4 u_wind; // (dirX, dirZ, strength 0..1+, integrated phase)
flat in uvec3 v_water; // kind (0 still, 1 laminar, 2 turbulent, 3 falling), direction, face
in float v_depth;
flat in float v_ice;
in vec2 v_ground;
in vec2 v_open;
uniform vec4 u_iceShape;
uniform vec2 u_ground; // wet ground and snow cover, 0..1 (render/precip.ts): snow lies on ice too
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
`;

/** Fragment main, after `tint`: the water's look by kind, then ice over it; writes `o_color`. */
export const WATER_FRAG = /* glsl */ `
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
  float openWind = top ? v_open.x * clamp(1.0 - v_ice / max(u_iceShape.z, 0.05), 0.0, 1.0) : 0.0;
  float wStr = u_wind.z * openWind;
  vec2 wdir = u_wind.xy;
  vec2 wperp = vec2(-wdir.y, wdir.x);
  if (kind == 0u) {
    // Still water: two copies drifting downwind with chop and gale whitecaps scaled by wind strength on open unfrozen water.
    float drift = 0.05 + 0.14 * wStr;
    vec2 wuv = vec2(dot(uv, wperp), dot(uv, wdir));
    a = texture(u_tex, vec3(wuv + vec2(t * 0.03, -t * drift), layer));
    b = texture(u_tex, vec3(vec2(0.5 - wuv.x * 1.15 - t * 0.025, wuv.y * 1.15 + 0.37 - t * (drift * 1.35)), layer));
    ripple = 0.45 + 0.95 * wStr;
    shine = 1.4;
    if (wStr > 0.15) {
      float chop = waterNoise(vec2(wuv.x * 2.2, wuv.y * 4.4 - u_wind.w * 0.9));
      float chop2 = waterNoise(vec2(wuv.x * 4.8 + 1.7, wuv.y * 8.5 - u_wind.w * 1.45));
      float crest = chop * 0.65 + chop2 * 0.35;
      foam = smoothstep(0.86 - 0.16 * min(wStr, 1.4), 0.97, crest) * smoothstep(0.60, 1.25, wStr) * 0.65;
    }
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
  // The sun (the moon at night) glints on open still or gently flowing water: choppy wind broadens the glint.
  vec3 n = normalize(vec3((a.r - b.r) * ripple, 1.0, (a.g - b.g) * ripple));
  float specPow = mix(160.0, 58.0, clamp(wStr * 0.75, 0.0, 1.0));
  float glint = pow(max(dot(reflect(normalize(v_rel), n), u_sunDir), 0.0), specPow)
              * step(0.95, v_light) * clamp(v_tint.g * 1.5 - 0.2, 0.0, 1.0) * (top && kind < 2u ? shine : 0.0);
  vec3 col = c.rgb * v_light * tint + u_direct * glint * 1.6;
  col = mix(col, FOAM * (0.35 + 0.65 * v_light) * tint, foam);
  // Thin water is clearer, so the ground shows through the tail of a flow; falling water is the most solid.
  float alpha = c.a * mix(0.45, 1.0, smoothstep(0.0, 0.5, v_depth));
  alpha = max(kind == 3u ? max(alpha, 0.7) : alpha, foam * 0.9);
  alpha = max(alpha, min(glint, 1.0));
  if (v_ice > 0.0) {
    // Ice creeps in from patches (a grey skin while it is too thin to bear), then a pale sheet with frosted spots and
    // faint cracks that whitens as it thickens; snow settles on it like on the ground.
    float form = clamp(v_ice / u_iceShape.z, 0.0, 1.0), thick = clamp((v_ice - u_iceShape.z) / (1.0 - u_iceShape.z), 0.0, 1.0);
    float frost = waterNoise(v_ground * 0.35) * 0.6 + waterNoise(v_ground * 2.3) * 0.4;
    float cover = form >= 1.0 ? 1.0 : smoothstep(frost, frost + 0.25, form * 1.25);
    float crack = smoothstep(0.93, 0.99, 1.0 - abs(waterNoise(v_ground * 0.8 + 7.1) * 2.0 - 1.0)) * (1.0 - thick * 0.6);
    vec3 ice = mix(vec3(0.5, 0.62, 0.68), vec3(0.74, 0.86, 0.93), form);
    ice = mix(ice, vec3(0.9, 0.94, 0.98), thick * 0.55 + frost * thick * 0.3) * (1.0 - crack * 0.3);
    ice = mix(ice, vec3(0.92, 0.95, 0.99), smoothstep(0.2, 0.8, u_ground.y) * 0.9);
    col = mix(col, ice * v_light * tint, cover);
    alpha = mix(alpha, mix(0.55, 0.97, max(form * 0.6, thick)), cover);
  }
  o_color = vec4(applyFog(col, v_rel), alpha);
`;
