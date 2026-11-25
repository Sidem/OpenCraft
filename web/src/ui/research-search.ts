// The research screen's search box: finds discovered techs by name or by what they unlock (undiscovered ones never
// match, so search spoils nothing). While there is a query, `matches` holds the hits; Enter steps through them.

import './research-search.css';
import { h } from './dom';

export class TechSearch {
  readonly el = h('div', 'rs-search');
  /** Tech indices matching the query, or null when empty. */
  matches: Set<number> | null = null;
  onChange: () => void = () => {};
  onGo: (t: number) => void = () => {};
  private readonly input = h('input', '') as HTMLInputElement;
  private readonly note = h('span', 'rs-search-note');
  private hits: number[] = [];
  private next = 0;

  /** `words[t]`: lower-case text tech `t` matches; `known(t)`: whether the player has discovered it. */
  constructor(
    private readonly words: string[],
    private readonly known: (t: number) => boolean,
  ) {
    this.input.type = 'search';
    this.input.placeholder = 'Search (/)';
    this.input.setAttribute('aria-label', 'Search research');
    this.input.addEventListener('input', () => this.search());
    this.input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && this.hits.length > 0) this.onGo(this.hits[this.next++ % this.hits.length]);
      else if (e.key === 'Escape') {
        e.stopPropagation();
        this.clear();
        this.input.blur();
      }
    });
    this.el.append(this.input, this.note);
  }

  focus(): void {
    this.input.focus();
    this.input.select();
  }

  clear(): void {
    this.input.value = '';
    this.search();
  }

  /** Re-runs the query, e.g. after discovery changed. */
  search(): void {
    const words = this.input.value.toLowerCase().split(/\s+/).filter(Boolean);
    this.next = 0;
    this.hits = words.length === 0 ? [] : this.words.flatMap((w, t) => (this.known(t) && words.every((q) => w.includes(q)) ? [t] : []));
    this.matches = words.length === 0 ? null : new Set(this.hits);
    this.note.textContent = words.length === 0 ? '' : this.hits.length === 0 ? 'none yet' : String(this.hits.length);
    this.onChange();
  }
}
