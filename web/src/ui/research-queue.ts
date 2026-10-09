// The research queue strip on the research screen: the techs waiting behind the current one, next first, each
// with a × that takes it out (and any queued tech that needs it). Read from the engine (`research_queue_*`);
// the strip is hidden while the queue is empty. Choosing and queueing techs is done on the tree (research.ts).

import './research-queue.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

export class ResearchQueue {
  readonly el = h('div', 'rq hidden');
  private drawn = '';

  constructor(private readonly game: Game) {}

  /** The queued techs, next first. */
  list(): number[] {
    const g = this.game;
    return Array.from({ length: g.research_queue_len() }, (_, i) => g.research_queue_at(i));
  }

  /** Redraws when the queue changed. */
  update(): void {
    const g = this.game;
    const queue = this.list();
    const key = queue.join(',');
    if (key === this.drawn) return;
    this.drawn = key;
    this.el.classList.toggle('hidden', queue.length === 0);
    const label = h('span', 'rq-label', `Up next (${queue.length} of ${g.research_queue_max()})`);
    const chips = queue.map((t, i) => {
      const chip = h('span', 'rq-chip');
      const remove = h('button', 'rq-x', '×');
      remove.type = 'button';
      remove.title = 'Take out of the queue';
      remove.setAttribute('aria-label', `Remove ${g.tech_name(t)} from the queue`);
      remove.addEventListener('click', () => g.unqueue_research(t));
      chip.append(h('span', 'rq-n', String(i + 1)), g.tech_name(t), remove);
      return chip;
    });
    this.el.replaceChildren(label, ...chips);
  }
}
