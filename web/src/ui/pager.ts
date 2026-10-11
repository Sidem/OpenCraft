// Page switcher shared by the backpack and the build menu: previous / next arrows around one numbered button per
// page, so lists never need a scrollbar. The owner calls `set(pages)` whenever its page count may have changed (the
// current page is clamped) and shows page `page` from `onChange`. A page with `gap` starts a new group (a new build
// category): it gets extra space before it, and `title` is its tooltip. `wheel` makes the mouse wheel flip pages.

import './pager.css';
import { h } from './dom';

export interface PageInfo {
  title: string;
  /** Starts a new group: drawn with a wider space before it. */
  gap?: boolean;
}

export class Pager {
  readonly el = h('div', 'pager hidden');
  page = 0;
  onChange: () => void = () => {};
  private readonly prev = h('button', 'pager-step', '‹');
  private readonly next = h('button', 'pager-step', '›');
  private readonly numbers = h('div', 'pager-numbers');
  private count = 0;
  private key = '';
  private lastWheel = 0;

  constructor() {
    for (const [b, label, d] of [[this.prev, 'Previous page', -1], [this.next, 'Next page', 1]] as const) {
      b.type = 'button';
      b.setAttribute('aria-label', label);
      b.addEventListener('click', () => this.go(this.page + d));
    }
    this.el.append(this.prev, this.numbers, this.next);
  }

  /** Sets the pages (one entry each); hides the pager when one page is all there is. */
  set(pages: PageInfo[]): void {
    this.count = pages.length;
    this.page = Math.max(0, Math.min(this.page, this.count - 1));
    this.el.classList.toggle('hidden', this.count <= 1);
    const key = pages.map((p) => p.title + (p.gap ? '+' : '')).join('|');
    if (key !== this.key) {
      this.key = key;
      this.numbers.replaceChildren(
        ...pages.map((p, i) => {
          const b = h('button', p.gap && i > 0 ? 'pager-page gap' : 'pager-page', String(i + 1));
          b.type = 'button';
          b.title = p.title;
          b.addEventListener('click', () => this.go(i));
          return b;
        }),
      );
    }
    this.mark();
  }

  /** Shows page `n` (clamped); calls `onChange` if it moved. */
  go(n: number): void {
    const page = Math.max(0, Math.min(n, this.count - 1));
    if (page === this.page) return;
    this.page = page;
    this.mark();
    this.onChange();
  }

  /** Lets the mouse wheel over `target` flip pages (the wheel can't scroll a list without a scrollbar). */
  wheel(target: HTMLElement): void {
    target.addEventListener(
      'wheel',
      (e) => {
        if (this.count <= 1) return;
        e.preventDefault();
        const now = performance.now();
        if (now - this.lastWheel < 120) return;
        this.lastWheel = now;
        this.go(this.page + Math.sign(e.deltaY));
      },
      { passive: false },
    );
  }

  private mark(): void {
    this.prev.disabled = this.page <= 0;
    this.next.disabled = this.page >= this.count - 1;
    Array.from(this.numbers.children).forEach((b, i) => b.setAttribute('aria-pressed', String(i === this.page)));
  }
}

/** How many fixed-height tiles (`tileH` px, `gap` px between) fit in `grid` as it is laid out now: its columns by
 * its rows. 0 while it is hidden, so a caller can wait until it has a size. */
export function tilesPerPage(grid: HTMLElement, tileH: number, gap: number): number {
  const height = grid.clientHeight;
  if (height === 0) return 0;
  const cols = Math.max(1, getComputedStyle(grid).gridTemplateColumns.split(' ').length);
  return cols * Math.max(1, Math.floor((height + gap) / (tileH + gap)));
}