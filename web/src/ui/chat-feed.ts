// Co-op chat lines while the command line is closed (console.ts owns it): the newest few at the foot of the HUD's left
// column (#hud-left), above the onboarding tip, each fading out a while after it came. Nothing here is kept; the
// console's log holds the history. To change how many or how long: `FEED_LINES`, `SHOW_MS`.

import './chat-feed.css';
import { h } from './dom';

const FEED_LINES = 6;
const SHOW_MS = 12_000;

export class ChatFeed {
  private readonly el = h('div', 'chat-feed');

  constructor() {
    (document.getElementById('hud-left') ?? document.body).prepend(this.el); // first: before the tip among equals
  }

  /** Shows `text` from `from`; the oldest line goes past `FEED_LINES`. */
  add(from: string, text: string): void {
    const row = h('div', 'chat-line');
    row.append(h('span', 'chat-from', `${from}:`), ` ${text}`);
    this.el.append(row);
    while (this.el.childElementCount > FEED_LINES) this.el.firstElementChild?.remove();
    setTimeout(() => row.classList.add('gone'), SHOW_MS);
    setTimeout(() => row.remove(), SHOW_MS + 1000);
  }

  /** Hidden while the console is open (its log shows the same lines). */
  show(on: boolean): void {
    this.el.classList.toggle('hidden', !on);
  }
}
