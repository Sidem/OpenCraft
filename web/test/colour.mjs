// Colour maths for the tests: sRGB to CIE L*a*b*, CIEDE2000, colour-blindness simulation (Machado et al.'s matrices at full
// severity, applied in linear sRGB) and WCAG contrast. `controls.test.mjs` checks the key caps with it, `tier-palette.test.mjs`
// the tier colours. Plain functions, no dependencies.

export const PROTAN = [[0.152286, 1.052583, -0.204868], [0.114503, 0.786281, 0.099216], [-0.003882, -0.048116, 1.051998]];
export const DEUTAN = [[0.367322, 0.860646, -0.227968], [0.280085, 0.672501, 0.047413], [-0.01182, 0.04294, 0.968881]];
export const TRITAN = [[1.255528, -0.076749, -0.178779], [-0.078411, 0.930809, 0.147602], [0.004733, 0.691367, 0.3039]];

const linear = (c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
/** Linear sRGB of a '#rrggbb' string. */
export const rgb = (hex) => [1, 3, 5].map((i) => linear(parseInt(hex.slice(i, i + 2), 16) / 255));
/** '#rrggbb' of a 0xRRGGBB number. */
export const hex = (n) => `#${n.toString(16).padStart(6, '0')}`;
/** What a colour-blind eye sees of linear sRGB `v`, through the matrix `m`. */
export const mul = (m, v) => m.map((r) => Math.min(1, Math.max(0, r[0] * v[0] + r[1] * v[1] + r[2] * v[2])));

/** CIE L*a*b* (D65) of linear sRGB. */
export function lab([r, g, b]) {
  const xyz = [
    0.4124564 * r + 0.3575761 * g + 0.1804375 * b,
    0.2126729 * r + 0.7151522 * g + 0.072175 * b,
    0.0193339 * r + 0.119192 * g + 0.9503041 * b,
  ];
  const f = (t) => (t > 216 / 24389 ? Math.cbrt(t) : (24389 / 27 * t + 16) / 116);
  const [fx, fy, fz] = [f(xyz[0] / 0.95047), f(xyz[1]), f(xyz[2] / 1.08883)];
  return [116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)];
}

/** CIEDE2000 (Sharma, Wu and Dalal's formulation). */
export function deltaE2000([L1, a1, b1], [L2, a2, b2]) {
  const rad = (d) => (d * Math.PI) / 180;
  const deg = (r) => (r * 180) / Math.PI;
  const C1 = Math.hypot(a1, b1), C2 = Math.hypot(a2, b2);
  const Cm = (C1 + C2) / 2;
  const G = 0.5 * (1 - Math.sqrt(Cm ** 7 / (Cm ** 7 + 25 ** 7)));
  const [ap1, ap2] = [(1 + G) * a1, (1 + G) * a2];
  const [Cp1, Cp2] = [Math.hypot(ap1, b1), Math.hypot(ap2, b2)];
  const hp = (b, a) => (b === 0 && a === 0 ? 0 : (deg(Math.atan2(b, a)) + 360) % 360);
  const [hp1, hp2] = [hp(b1, ap1), hp(b2, ap2)];
  const dL = L2 - L1, dC = Cp2 - Cp1;
  let dh = 0;
  if (Cp1 * Cp2 !== 0) {
    dh = hp2 - hp1;
    if (dh > 180) dh -= 360;
    else if (dh < -180) dh += 360;
  }
  const dH = 2 * Math.sqrt(Cp1 * Cp2) * Math.sin(rad(dh / 2));
  const Lm = (L1 + L2) / 2, Cpm = (Cp1 + Cp2) / 2;
  let hm = hp1 + hp2;
  if (Cp1 * Cp2 !== 0) {
    if (Math.abs(hp1 - hp2) > 180) hm += hp1 + hp2 < 360 ? 360 : -360;
    hm /= 2;
  }
  const T = 1 - 0.17 * Math.cos(rad(hm - 30)) + 0.24 * Math.cos(rad(2 * hm)) + 0.32 * Math.cos(rad(3 * hm + 6)) - 0.2 * Math.cos(rad(4 * hm - 63));
  const dTheta = 30 * Math.exp(-(((hm - 275) / 25) ** 2));
  const Rc = 2 * Math.sqrt(Cpm ** 7 / (Cpm ** 7 + 25 ** 7));
  const Sl = 1 + (0.015 * (Lm - 50) ** 2) / Math.sqrt(20 + (Lm - 50) ** 2);
  const Sc = 1 + 0.045 * Cpm, Sh = 1 + 0.015 * Cpm * T;
  const Rt = -Math.sin(rad(2 * dTheta)) * Rc;
  return Math.sqrt((dL / Sl) ** 2 + (dC / Sc) ** 2 + (dH / Sh) ** 2 + Rt * (dC / Sc) * (dH / Sh));
}

/** WCAG contrast ratio of two linear-sRGB colours. */
export const contrast = (a, b) => {
  const y = (c) => 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
  const [hi, lo] = [y(a), y(b)].sort((p, q) => q - p);
  return (hi + 0.05) / (lo + 0.05);
};

/** The smallest CIEDE2000 between any two of `colours` (0xRRGGBB) as seen through `sim` (a matrix, or null for normal sight). */
export function worstPair(colours, sim) {
  const seen = colours.map((c) => lab(sim ? mul(sim, rgb(hex(c))) : rgb(hex(c))));
  let worst = { d: Infinity, i: 0, j: 0 };
  for (let i = 0; i < seen.length; i++) {
    for (let j = i + 1; j < seen.length; j++) {
      const d = deltaE2000(seen[i], seen[j]);
      if (d < worst.d) worst = { d, i, j };
    }
  }
  return worst;
}
