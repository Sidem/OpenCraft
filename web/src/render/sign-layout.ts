// Where a note sign's lines go on its face (render/sign-faces.ts paints them): one font size for every line, as large as
// fits a row of `rows` and the widest line, the same on every sign that fits, so a row of signs reads alike; the lines in
// use are centred on the face as a block (trailing empty lines don't count). Pure, tested in web/test/sign.test.mjs.

export interface SignLayout {
  /** Font size in pixels. */
  size: number;
  /** Each line's middle, in pixels from the top. */
  middles: number[];
}

/** Share of the face's height left as margin at each edge, and of a row the letters fill. */
const MARGIN = 0.08;
const FILL = 0.74;

/** Lays out `lines` on a face `w` × `h` pixels with room for `rows` lines; `measure(text, size)` is a line's width. */
export function layoutSign(lines: string[], rows: number, w: number, h: number, measure: (text: string, size: number) => number): SignLayout {
  const margin = h * MARGIN;
  const row = (h - 2 * margin) / rows;
  let size = row * FILL;
  const widest = Math.max(0, ...lines.map((l) => measure(l, size)));
  const room = w - 2 * margin;
  if (widest > room) size *= room / widest;
  let used = lines.length;
  while (used > 1 && lines[used - 1] === '') used--;
  const top = margin + ((rows - used) * row) / 2;
  return { size, middles: lines.slice(0, used).map((_, i) => top + row * (i + 0.5)) };
}
