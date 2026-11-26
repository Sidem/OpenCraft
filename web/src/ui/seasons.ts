// The four seasons as the calendar UI draws them (season-chip.ts, calendar.ts): a name (tooltips only), a colour and
// a small icon, in the engine's order (`game.season()`: 0 spring to 3 winter, daytime.rs), plus the sun and moon
// icons and the arc maths the year rings share. The year runs clockwise from the top. To restyle a season, edit its row.

import './seasons.css';

const SVG = 'http://www.w3.org/2000/svg';

export const SEASONS = [
  { name: 'Spring', colour: '#7fd35c' },
  { name: 'Summer', colour: '#f2c94c' },
  { name: 'Autumn', colour: '#e8823a' },
  { name: 'Winter', colour: '#9cc8f0' },
] as const;

/** Icon bodies in a 16 × 16 box, drawn in `currentColor`. */
const ICONS = {
  spring:
    '<path d="M8 15V8" class="ln"/><path d="M8 9.5C8 6 5.5 4 2 4c0 3.5 2.5 5.5 6 5.5z"/>' +
    '<path d="M8 8c0-3.5 2.2-6 6-6 0 3.5-2.2 6-6 6z"/>',
  summer:
    '<circle cx="8" cy="8" r="3.2"/><path class="ln" d="M8 1v2.2M8 12.8V15M1 8h2.2M12.8 8H15M3 3l1.6 1.6M11.4 11.4L13 13M13 3l-1.6 1.6M4.6 11.4L3 13"/>',
  autumn: '<path d="M2.5 13.5C2.5 6.5 6.5 2.5 13.5 2.5c0 7-4 11-11 11z"/><path class="ln dark" d="M2.5 13.5l7-7"/>',
  winter:
    '<path class="ln" d="M8 1.5v13M2.4 4.75l11.2 6.5M2.4 11.25l11.2-6.5M6.3 2.6L8 4.2l1.7-1.6M6.3 13.4L8 11.8l1.7 1.6"/>',
  sun: '<circle cx="8" cy="8" r="3.5"/><path class="ln" d="M8 1v2M8 13v2M1 8h2M13 8h2M3 3l1.4 1.4M11.6 11.6L13 13M13 3l-1.4 1.4M4.4 11.6L3 13"/>',
  moon: '<path d="M10.5 1.5a6.5 6.5 0 1 0 4 10.5A5.5 5.5 0 0 1 10.5 1.5z"/>',
} as const;
const SEASON_ICONS = ['spring', 'summer', 'autumn', 'winter'] as const;

/** A small inline icon (`seasonIcon` for a season by index). Size and colour come from CSS (`.sn-icon`). */
export function icon(name: keyof typeof ICONS): SVGSVGElement {
  const svg = document.createElementNS(SVG, 'svg');
  svg.setAttribute('viewBox', '0 0 16 16');
  svg.setAttribute('class', 'sn-icon');
  svg.setAttribute('aria-hidden', 'true');
  svg.innerHTML = ICONS[name];
  return svg;
}

export function seasonIcon(season: number): SVGSVGElement {
  const svg = icon(SEASON_ICONS[season % 4]);
  svg.style.color = SEASONS[season % 4].colour;
  return svg;
}

/** An SVG element with attributes. */
export function svgEl<K extends keyof SVGElementTagNameMap>(tag: K, attrs: Record<string, string | number>): SVGElementTagNameMap[K] {
  const e = document.createElementNS(SVG, tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, String(v));
  return e;
}

/** The point at year fraction `f` (0 top, clockwise) on a circle of radius `r` round (c, c). */
export function onRing(c: number, r: number, f: number): [number, number] {
  const a = f * 2 * Math.PI;
  return [c + r * Math.sin(a), c - r * Math.cos(a)];
}

/** Path data for the arc from year fraction `f0` to `f1` (clockwise) at radius `r`. */
export function arc(c: number, r: number, f0: number, f1: number): string {
  const [x0, y0] = onRing(c, r, f0);
  const [x1, y1] = onRing(c, r, f1);
  return `M${x0.toFixed(2)} ${y0.toFixed(2)}A${r} ${r} 0 ${f1 - f0 > 0.5 ? 1 : 0} 1 ${x1.toFixed(2)} ${y1.toFixed(2)}`;
}

/** The four season arcs of a ring, a small gap between them. */
export function seasonRing(c: number, r: number, width: number, gap = 0.006): SVGPathElement[] {
  return SEASONS.map((s, i) =>
    svgEl('path', { d: arc(c, r, i / 4 + gap, (i + 1) / 4 - gap), stroke: s.colour, 'stroke-width': width, fill: 'none' }),
  );
}
