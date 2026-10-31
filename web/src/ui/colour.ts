// Colour and text helpers for the sign editor (ui/sign.ts) and its colour wheel (ui/colour-wheel.ts): colours as the
// engine keeps them (0xRRGGBB numbers), hue/saturation/value for the wheel, where a hue and saturation sit on a wheel of a
// given radius, and cutting a line to the bytes a sign holds. Pure functions, tested in web/test/sign.test.mjs.

/** `rgb` (0xRRGGBB) as a CSS colour, `#rrggbb`. */
export function hex(rgb: number): string {
  return `#${(rgb & 0xffffff).toString(16).padStart(6, '0')}`;
}

/** Hue (0..1, red at 0), saturation and value (0..1) as 0xRRGGBB. */
export function hsvToRgb(h: number, s: number, v: number): number {
  const f = (n: number) => {
    const k = (n + h * 6) % 6;
    return Math.round(255 * (v - v * s * Math.max(0, Math.min(k, 4 - k, 1))));
  };
  return (f(5) << 16) | (f(3) << 8) | f(1);
}

/** 0xRRGGBB as hue, saturation and value (0..1 each; a grey has hue 0). */
export function rgbToHsv(rgb: number): [number, number, number] {
  const r = ((rgb >> 16) & 255) / 255, g = ((rgb >> 8) & 255) / 255, b = (rgb & 255) / 255;
  const max = Math.max(r, g, b), d = max - Math.min(r, g, b);
  let h = 0;
  if (d > 0) {
    if (max === r) h = ((g - b) / d + 6) % 6;
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
  }
  return [h / 6, max === 0 ? 0 : d / max, max];
}

/** The hue and saturation at (`x`, `y`) from the centre of a wheel of radius `r` (hue round, saturation out; past the rim
 * counts as the rim). */
export function wheelPick(x: number, y: number, r: number): [number, number] {
  const h = (Math.atan2(y, x) / (2 * Math.PI) + 1) % 1;
  return [h, Math.min(1, Math.hypot(x, y) / r)];
}

/** Where a hue and saturation sit from the centre of a wheel of radius `r` (`wheelPick`'s inverse). */
export function wheelPoint(h: number, s: number, r: number): [number, number] {
  return [Math.cos(h * 2 * Math.PI) * s * r, Math.sin(h * 2 * Math.PI) * s * r];
}

/** `text` without control characters, cut to the whole characters that fit in `max` bytes of UTF-8 (a sign's line). */
export function fitBytes(text: string, max: number): string {
  let out = '', used = 0;
  for (const c of text) {
    if (/\p{Cc}/u.test(c)) continue;
    const n = new TextEncoder().encode(c).length;
    if (used + n > max) break;
    out += c;
    used += n;
  }
  return out;
}
