// How a tip's text (the engine's `hint_text`, `lesson_text`) is shown: a lead-in before the first colon becomes the title,
// `*words*` are highlighted and `[keys]` drawn as key caps. Used by the tip card in the game (hints.ts) and on the Home page
// (home.ts). `tipParts` is pure (tested in web/test/home-text.test.mjs); `richText` only builds elements, never parses HTML.
// (No imports, so the test can load it.)

/** A lead-in this short before the first ": " is the tip's title. */
const TITLE_MAX = 40;

/** Splits a tip into its title (when it has a short lead-in) and the body, whose first letter is a capital. */
export function tipParts(text: string): { title: string; body: string } {
  const lead = text.indexOf(': ');
  if (lead > 0 && lead <= TITLE_MAX && !/[*[]/.test(text.slice(0, lead))) {
    const rest = text.slice(lead + 2);
    return { title: text.slice(0, lead), body: rest.charAt(0).toUpperCase() + rest.slice(1) };
  }
  return { title: '', body: text };
}

/** Appends `text` to `el`, with `*words*` highlighted and `[keys]` as key caps. */
export function richText(el: HTMLElement, text: string): void {
  const marks = /\*([^*]+)\*|\[([^\]]+)\]/g;
  let at = 0;
  for (const m of text.matchAll(marks)) {
    if (m.index > at) el.append(text.slice(at, m.index));
    const mark = document.createElement(m[1] !== undefined ? 'strong' : 'kbd');
    mark.className = m[1] !== undefined ? 'hint-em' : 'hint-cap';
    mark.textContent = m[1] ?? m[2];
    el.append(mark);
    at = m.index + m[0].length;
  }
  if (at < text.length) el.append(text.slice(at));
}
