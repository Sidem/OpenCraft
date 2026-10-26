// Inline markup for the patch notes: **bold** and `code` in a line of text. Pure (no DOM), so it is tested in
// `web/test/inline-text.test.mjs`; `ui/patch-notes.ts` turns the pieces into elements. Anything else is plain text,
// and an unclosed marker stays as typed.

export interface Piece {
  text: string;
  /** How it is shown: plain, strong or code. */
  kind: 'plain' | 'bold' | 'code';
}

const MARKUP = /\*\*(.+?)\*\*|`([^`]+)`/g;

export function inlinePieces(line: string): Piece[] {
  const pieces: Piece[] = [];
  let from = 0;
  for (const m of line.matchAll(MARKUP)) {
    if (m.index > from) pieces.push({ text: line.slice(from, m.index), kind: 'plain' });
    pieces.push(m[1] === undefined ? { text: m[2], kind: 'code' } : { text: m[1], kind: 'bold' });
    from = m.index + m[0].length;
  }
  if (from < line.length) pieces.push({ text: line.slice(from), kind: 'plain' });
  return pieces;
}
