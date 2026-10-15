// The research screen's search box: finds techs by name or by what they unlock ("steel", "pole"). While there is
// a query, `matches` is the set of hits (the screen lights them and dims the rest); Enter jumps to the next hit.
// "/" focuses the box (research.ts), Esc empties it.

import './research-search.css';
import { h } from './dom';
import type { TechView } from './research-node';

export class TechSearch {
  readonly el = h('div', 'rs-search');
  /** The techs matching the query, or null when there is none. */
  matches: Set<number> | null = null;
  /** Called after the query changed. */
  onChange: () => void = () => {};
  /** Called with the tech to show when Enter picks a hit. */
  onGo: (t: number) => void = () => {};
  private readonly input = h('input', '') as HTMLInputElement;
  private readonly note = h('span', 'rs-search-note');
  private next = 0;

  constructor(private readonly views: TechView[]) {
    this.input.type = 'search';
    this.input.placeholder = 'Search techs and what they unlock';
    this.input.setAttribute('aria-label', 'Search research');
    this.input.addEventListener('input', () => this.search());
    this.input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && this.matches && this.matches.size > 0) {
        const hits = [...this.matches];
        this.onGo(hits[this.next++ % hits.length]);
      } else if (e.key === 'Escape') {
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

  private search(): void {
    const words = this.input.value.toLowerCase().split(/\s+/).filter(Boolean);
    this.next = 0;
    this.matches = words.length === 0
      ? null
      : new Set(this.views.flatMap((v, t) => (words.every((w) => v.words.includes(w)) ? [t] : [])));
    this.note.textContent = this.matches ? (this.matches.size === 0 ? 'none' : `${this.matches.size}`) : '';
    this.onChange();
  }
}
