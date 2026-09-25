// Tiny DOM helpers shared by the UI panels.

/** Creates an element with an optional class name and text. */
export function h<K extends keyof HTMLElementTagNameMap>(tag: K, className = '', text = ''): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  if (className) e.className = className;
  if (text) e.textContent = text;
  return e;
}

/** A `type="button"` button that calls `onClick` with itself. */
export function button(className: string, text: string, onClick: (b: HTMLButtonElement) => void): HTMLButtonElement {
  const b = h('button', className, text);
  b.type = 'button';
  b.addEventListener('click', () => onClick(b));
  return b;
}
