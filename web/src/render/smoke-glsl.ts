// GLSL ES 3.00 shaders for volumetric wind-reactive smoke (`smoke.ts`, engine `factory/smoke.rs`).
// The vertex shader expands each 8-float emitter record into 16 camera-facing billowing cloudlets with smooth
// bit-reversed distance LOD (16 -> 8 -> 4), buoyant rise, crossflow wind bending, world-space travelling gusts,
// superlinear wind-speed chaos and velocity-aligned shear. The fragment shader performs two domain-warped lookups into
// the precomputed tileable 4-channel FBM/Worley noise texture for 3D volumetric shading, sun/moon wrap lighting,
// backlit silver-lining scatter and atmospheric fog.

import { FOG, LIGHT } from './shaders';

export const SMOKE_VERT = /* glsl */ `#version 300 es
precision highp float;
precision highp int;

layout(location = 0) in vec3 a_quad; // .xy corner (-1..1), .z puff index (0..15)
layout(location = 1) in vec4 a_i0;   // camera-relative mouth xyz, scale
layout(location = 2) in vec3 a_i1;   // seed (0..1), intensity (or burst age), packed light + kind * 256
layout(location = 3) in uint a_mask; // 32-bit sliding stream ON/OFF history (bit 0 = live mouth, bits 1..31 = rising stream)

uniform mat4 u_view;
uniform mat4 u_proj;
uniform vec3 u_eyeMod;    // camera world position mod 256 (for world-anchored travelling gusts)
uniform vec4 u_wind;      // (dirX, dirZ, speedMs, gustPhase)
uniform vec4 u_phase;     // (plumePhase, turbPhase, locoPhase, 0)
uniform vec3 u_skyLight;

out vec2 v_uv;
out vec2 v_quad;
out float v_age;
out vec4 v_meta;  // seed, opacity, chaos, kind
out vec3 v_light; // ambient world + sky light
out float v_sunAmt;
out vec3 v_rel;

// Bit-reversed slots: prefixes 0..3, 0..7 and 0..15 are each evenly spaced over [0, 1) for pop-free GPU LOD.
const float SLOTS[16] = float[16](
  0.0000, 0.5000, 0.2500, 0.7500,
  0.1250, 0.6250, 0.3750, 0.8750,
  0.0625, 0.5625, 0.3125, 0.8125,
  0.1875, 0.6875, 0.4375, 0.9375
);

${LIGHT}

vec3 hash31(float p) {
  vec3 p3 = fract(vec3(p) * vec3(0.1031, 0.1030, 0.0973));
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.xxy + p3.yzz) * p3.zyx);
}

void main() {
  int idx = int(a_quad.z + 0.5);
  float slot = SLOTS[idx];
  vec3 mouth = a_i0.xyz;
  float scale = a_i0.w;
  float seed = a_i1.x;
  float intensity = a_i1.y;
  uint packedLk = uint(clamp(a_i1.z, 0.0, 4095.0) + 0.5);
  uint lightByte = packedLk & 255u;
  float kind = float(packedLk >> 8u);

  float mouthDist = length(mouth);
  // Smooth distance LOD: 16 puffs close, 8 mid-range, 4 far; collapsed quads produce zero fragments.
  float lodFade = 1.0;
  float lodSizeBoost = 1.0;
  float lodAlphaBoost = 1.0;
  if (kind < 3.5) {
    if (idx >= 8) {
      lodFade = 1.0 - smoothstep(34.0, 62.0, mouthDist);
    } else if (idx >= 4) {
      lodFade = 1.0 - smoothstep(68.0, 108.0, mouthDist);
    }
    if (lodFade <= 0.01) {
      gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
      return;
    }
    float midT = smoothstep(34.0, 62.0, mouthDist);
    float farT = smoothstep(68.0, 108.0, mouthDist);
    lodSizeBoost = 1.0 + 0.22 * midT + 0.30 * farT;
    lodAlphaBoost = 1.0 + 0.26 * midT + 0.32 * farT;
  }

  vec2 windDir = u_wind.xy;
  vec2 windCross = vec2(-windDir.y, windDir.x);
  float windSpeed = max(u_wind.z, 0.0);
  float windNorm = clamp(windSpeed / 14.0, 0.0, 1.6);
  // Superlinear chaos: silky laminar rise at low wind, turbulent vortex breakup at high wind.
  float chaos = clamp(pow(windSpeed / 10.5, 1.38), 0.0, 1.85);

  // World-space travelling wind gusts so neighbouring chimneys ripple sequentially as a gust passes.
  vec2 worldXZ = u_eyeMod.xz + mouth.xz;
  float along = dot(worldXZ, windDir);
  float across = dot(worldXZ, windCross);
  float g1 = 0.5 + 0.5 * sin(along * 0.075 - u_wind.w * 1.45 + sin(across * 0.055 + u_wind.w * 0.42) * 1.35);
  float g2 = 0.5 + 0.5 * sin(along * 0.165 - u_wind.w * 2.65 + across * 0.09);
  float gust = mix(1.0, mix(0.68, 1.42, g1 * 0.65 + g2 * 0.35), clamp(windNorm * 1.1, 0.0, 1.0));

  vec3 rnd = hash31(seed * 97.31 + slot * 41.17 + float(idx) * 3.13);
  vec3 puffRel;
  float age;
  float puffRadius;
  float opacity;
  vec3 flowVel = vec3(0.0, 1.0, 0.0);

  if (kind > 3.5) {
    // Radial launch/landing shock ring (orbit_fx.rs): 16 billowing puffs rolling outward and drifting in wind.
    age = clamp(intensity, 0.0, 1.0);
    float ang = slot * 6.2831853 + seed * 6.2831853 + (rnd.x - 0.5) * 0.32;
    float spread = 1.0 - pow(1.0 - age, 2.2);
    float r = scale * (0.35 + 2.55 * spread) * (0.85 + 0.3 * rnd.y);
    float up = scale * (0.18 + 1.05 * pow(age, 0.85)) * (0.8 + 0.4 * rnd.z);
    vec2 drift = windDir * (windSpeed * gust * 0.32 * age * age);
    vec2 swirl = vec2(sin(u_phase.y * 2.0 + rnd.x * 18.0), cos(u_phase.y * 1.8 + rnd.y * 18.0))
      * (0.25 + 0.55 * chaos) * age * scale;
    puffRel = mouth + vec3(cos(ang) * r + drift.x + swirl.x, up, sin(ang) * r + drift.y + swirl.y);
    puffRadius = scale * (0.68 + 1.35 * pow(age, 0.72)) * (0.88 + 0.24 * rnd.z);
    opacity = 0.72 * (1.0 - smoothstep(0.62, 1.0, age));
    flowVel = vec3(cos(ang) + windDir.x * windNorm, 0.4, sin(ang) + windDir.y * windNorm);
  } else {
    float rateMul = kind > 2.5 ? 1.45 : (kind > 1.5 ? 1.28 : (kind > 0.5 ? 0.88 : 1.0));
    age = fract(u_phase.x * rateMul + seed + slot);
    float streamMask = 1.0;
    if (a_mask != 0xFFFFFFFFu) {
      float sub = fract(u_phase.x * rateMul * 31.0);
      float bitPos = clamp(age * 31.0 - sub + 0.5, 0.0, 31.0);
      uint b0 = uint(floor(bitPos));
      uint b1 = min(b0 + 1u, 31u);
      float s0 = float((a_mask >> b0) & 1u);
      float s1 = float((a_mask >> b1) & 1u);
      streamMask = mix(s0, s1, fract(bitPos));
      if (streamMask <= 0.01) {
        gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
        return;
      }
    }
    float ageRise = pow(age, 0.78);
    float ageTurb = pow(age, 1.25);

    float riseBase = kind > 2.5 ? (2.2 + 0.8 * scale)
                   : kind > 1.5 ? (3.8 + 1.6 * scale)
                   : kind > 0.5 ? (4.8 + 2.2 * scale)
                   : (3.2 + 2.1 * scale);
    float riseAtten = 1.0 / (1.0 + 0.115 * windSpeed * gust);
    float riseY = riseBase * riseAtten * ageRise;

    // Crossflow drag entrainment: exits vertically at the lip, then curves smoothly downwind.
    float effWind = windSpeed * gust * (kind > 2.5 ? 0.42 : (kind > 0.5 && kind < 1.5 ? 0.62 : 0.66));
    float advect = effWind * (0.72 + 0.28 * clamp(scale, 0.35, 1.6)) * (age - 0.24 * (1.0 - exp(-4.5 * age)));

    // Gentle laminar thermal helix in calm air so still smoke curls lazily and blooms near the top.
    float calmAmp = (0.16 + 0.18 * scale) * pow(age, 1.35);
    vec2 calmCurl = vec2(
      sin(u_phase.y * 0.65 + age * 4.4 + seed * 6.283 + rnd.x * 2.4),
      cos(u_phase.y * 0.52 + age * 3.7 + seed * 11.4 + rnd.y * 2.4)
    ) * calmAmp;

    // Wind-driven multi-frequency chaotic eddies: crosswind vortex shedding, vertical downdrafts & gust surges.
    float t = u_phase.y;
    float lat1 = sin(t * 2.0 + age * (6.5 + 5.0 * chaos) + seed * 19.3 + rnd.x * 6.283);
    float lat2 = sin(t * 4.4 - age * 13.5 + rnd.y * 17.1) * 0.55;
    float crossDisp = (lat1 + lat2 * chaos) * chaos * (0.48 + 0.46 * scale) * ageTurb;

    float v1 = cos(t * 1.85 + age * (6.0 + 4.5 * chaos) + seed * 13.7 + rnd.z * 6.283);
    float v2 = sin(t * 3.9 + age * 11.2 + rnd.x * 11.3) * 0.5;
    float vertDisp = (v1 + v2 * chaos) * chaos * (0.36 + 0.34 * scale) * pow(age, 1.32);

    float alongDisp = sin(t * 2.65 + age * 8.4 + rnd.y * 9.7) * chaos * (0.46 + 0.32 * scale) * ageTurb;

    // Rocket exhaust boils radially outward off the launch pad.
    vec2 rocketSpread = vec2(0.0);
    if (kind > 2.5) {
      float ra = seed * 6.2831853 + (rnd.x - 0.5) * 1.6;
      rocketSpread = vec2(cos(ra), sin(ra)) * scale * 0.95 * pow(age, 0.7);
    }

    vec2 horiz = windDir * (advect + alongDisp) + windCross * crossDisp + calmCurl + rocketSpread;
    float finalY = max(riseY + vertDisp, 0.02 * scale);
    puffRel = mouth + vec3(horiz.x, finalY, horiz.y);

    float expand = (kind > 2.5 ? 1.75 : 1.58) + 0.95 * chaos;
    puffRadius = scale * (0.34 + expand * pow(age, 0.82)) * (0.86 + 0.28 * rnd.z) * lodSizeBoost;

    // Density modulation: locomotive chug rhythm, faster dilution in high-wind dispersion.
    float chug = kind > 1.5 && kind < 2.5
      ? (0.72 + 0.28 * sin(u_phase.z * 16.0 + seed * 12.0 - age * 14.0))
      : 1.0;
    float windDilute = 1.0 / (1.0 + 0.18 * chaos * age);
    float baseOpacity = kind > 2.5 ? 0.68 : (kind > 0.5 && kind < 1.5 ? 0.62 : 0.52);
    opacity = baseOpacity * intensity * chug * windDilute * lodFade * lodAlphaBoost * streamMask;

    float dAdvect = effWind * (1.0 - exp(-4.5 * age));
    float dRise = riseBase * riseAtten * 0.78 * pow(max(age, 0.05), -0.22);
    flowVel = vec3(windDir.x * dAdvect, dRise, windDir.y * dAdvect);
  }

  vec4 viewPos = u_view * vec4(puffRel, 1.0);
  float dist = length(viewPos.xyz);
  // Soft fade right next to the camera so walking/flying through smoke never pops quads in your face.
  float nearFade = smoothstep(0.55, 1.75, dist);
  opacity *= nearFade;
  if (opacity <= 0.008) {
    gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
    return;
  }

  // Slight pull towards eye so mouth puffs wrap smoothly around narrow chimney caps without slicing.
  float pull = min(puffRadius * 0.28, max(dist - 0.35, 0.0));
  viewPos.xyz *= (dist - pull) / max(dist, 1e-4);

  // Subtle anisotropic wind-shear elongation along the plume flow direction at high wind speeds.
  vec2 flowView = (u_view * vec4(flowVel, 0.0)).xy;
  float flowLen = length(flowView);
  vec2 stretchCorner = a_quad.xy;
  if (flowLen > 0.05 && chaos > 0.15) {
    vec2 fDir = flowView / flowLen;
    float stretch = 1.0 + 0.42 * min(chaos, 1.4) * smoothstep(0.06, 0.65, age) * smoothstep(0.08, 0.55, flowLen);
    float alongQ = dot(stretchCorner, fDir);
    vec2 perpQ = stretchCorner - fDir * alongQ;
    stretchCorner = fDir * (alongQ * stretch) + perpQ * (1.0 / sqrt(stretch));
  }

  viewPos.xy += stretchCorner * puffRadius;

  // Per-puff vorticity spin (alternating directions, spinning faster in chaotic high wind).
  float spinSign = rnd.x > 0.5 ? 1.0 : -1.0;
  float spin = rnd.y * 6.2831853 + spinSign * age * (1.1 + 2.8 * chaos);
  float cs = cos(spin), sn = sin(spin);
  vec2 rotCorner = mat2(cs, -sn, sn, cs) * a_quad.xy;

  v_uv = rotCorner * 0.5 + 0.5;
  v_quad = a_quad.xy;
  v_age = age;
  v_meta = vec4(seed + slot * 0.73, opacity, chaos, kind);
  v_light = lightTint(lightByte, u_skyLight);
  float skyVis = pow(0.8, 15.0 - float(lightByte & 15u));
  v_sunAmt = skyVis * skyVis;
  v_rel = puffRel;
  gl_Position = u_proj * viewPos;
}`;

export const SMOKE_FRAG = /* glsl */ `#version 300 es
precision highp float;

in vec2 v_uv;
in vec2 v_quad;
in float v_age;
in vec4 v_meta;
in vec3 v_light;
in float v_sunAmt;
in vec3 v_rel;

uniform sampler2D u_noise;
uniform vec4 u_phase;
uniform vec3 u_sunDir;
uniform vec3 u_sunView;
uniform vec3 u_direct;
${FOG}

out vec4 o_color;

void main() {
  float r2 = dot(v_quad, v_quad);
  if (r2 >= 1.0) discard;
  float r = sqrt(r2);

  float seed = v_meta.x;
  float opacity = v_meta.y;
  float chaos = v_meta.z;
  float kind = v_meta.w;
  float age = v_age;

  // Two domain-warped mipmapped lookups into the precomputed tileable 4-channel FBM/Worley noise texture.
  float turbScroll = u_phase.y;
  vec2 uv1 = v_uv * (0.78 + 0.26 * age) + vec2(seed * 7.31, -turbScroll * 0.06 - age * 0.28);
  vec4 n1 = texture(u_noise, uv1);

  float warpAmp = (0.14 + 0.17 * min(chaos, 1.5)) * (0.45 + 0.55 * age);
  vec2 warp = (n1.ra - 0.5) * warpAmp;
  vec2 uv2 = v_uv * (1.18 + 0.42 * min(chaos, 1.5)) + warp + vec2(-seed * 5.17, -turbScroll * 0.11 - age * 0.52);
  vec4 n2 = texture(u_noise, uv2);

  // Plump rolling billows at low wind; fine shredded wisps mixed in at high wind.
  float billow = n1.r * 0.52 + n2.g * 0.48;
  float wispMix = clamp(chaos * 0.48, 0.0, 0.62) * (0.35 + 0.65 * age);
  float field = mix(billow, billow * 0.58 + n2.b * 0.52, wispMix);

  // Noise-eroded radial envelope: puffs emerge softly, billow fully, then dissolve into wispy tendrils.
  float radial = 1.0 - smoothstep(0.14, 1.0, r);
  float erode = mix(0.12, 0.68, pow(age, 1.35)) + 0.07 * min(chaos, 1.5) * age;
  float softEdge = 0.34 + 0.12 * age;
  float density = smoothstep(erode, erode + softEdge, radial * (0.36 + 0.86 * field));

  float birthFade = kind > 3.5 ? smoothstep(0.0, 0.05, age) : smoothstep(0.0, 0.055, age);
  float tailFade = 1.0 - smoothstep(0.68, 1.0, age);
  float farFade = 1.0 - smoothstep(u_fog.x * 1.1, u_fog.y, length(v_rel));
  float alpha = clamp(density * opacity * birthFade * tailFade * farFade, 0.0, 0.92);
  if (alpha < 0.008) discard;

  // Cheap 3D volumetric normal: spherical puff profile perturbed by billow lobe gradients.
  vec3 nSphere = vec3(v_quad.x * 0.72, v_quad.y * 0.72, sqrt(max(0.06, 1.0 - r2 * 0.52)));
  vec3 nBillow = normalize(nSphere + vec3((n1.r - n2.a) * 0.92, (n2.g - n1.g) * 0.92 + 0.24, 0.0));

  // Wrapped sun/moon directional lighting + core self-shadowing + backlit silver-lining forward scatter.
  float ndl = dot(nBillow, u_sunView) * 0.5 + 0.5;
  float wrapSun = pow(clamp(ndl, 0.0, 1.0), 1.3);
  float thickness = clamp(density * (1.0 - r * 0.35), 0.0, 1.0);
  float selfShadow = mix(1.0, 0.70, thickness * (1.0 - wrapSun));
  vec3 viewDir = normalize(v_rel);
  float silverLining = pow(max(dot(viewDir, u_sunDir), 0.0), 4.0) * (1.0 - thickness * 0.52);

  // Smoke albedo by archetype: hot dark soot near mouth cools and lightens as it entrains ambient air.
  vec3 albedo;
  if (kind > 3.5) {
    albedo = mix(vec3(0.84, 0.80, 0.74), vec3(0.86, 0.87, 0.89), smoothstep(0.0, 0.8, age));
  } else if (kind > 2.5) {
    albedo = mix(vec3(0.96, 0.93, 0.87), vec3(0.90, 0.91, 0.94), smoothstep(0.0, 0.8, age));
  } else if (kind > 1.5) {
    albedo = mix(vec3(0.54, 0.56, 0.60), vec3(0.86, 0.88, 0.91), smoothstep(0.0, 0.72, age));
  } else if (kind > 0.5) {
    albedo = mix(vec3(0.20, 0.19, 0.19), vec3(0.50, 0.49, 0.50), smoothstep(0.0, 0.85, age));
  } else {
    albedo = mix(vec3(0.40, 0.41, 0.44), vec3(0.78, 0.80, 0.84), smoothstep(0.0, 0.75, age));
  }

  // Slight internal tonal depth (brighter sunlit billow crests, deeper shaded hollows).
  albedo *= mix(0.86, 1.08, n2.g);
  vec3 ambient = v_light * mix(0.66, 1.04, nBillow.y * 0.5 + 0.5) * selfShadow;
  vec3 direct = u_direct * v_sunAmt * (0.46 * wrapSun + 0.44 * silverLining);
  vec3 col = albedo * (ambient + direct);

  // Subtle warm orange fire-bounce illumination on the underside of wildfire smoke near the flames.
  if (kind > 0.5 && kind < 1.5 && age < 0.42) {
    float glow = (1.0 - smoothstep(0.0, 0.42, age)) * clamp(-v_quad.y * 0.55 + 0.45, 0.0, 1.0);
    col += vec3(0.95, 0.34, 0.06) * glow * 0.38 * (0.5 + 0.5 * n1.r);
  }

  col = applyFog(col, v_rel);
  o_color = vec4(col, alpha);
}`;
