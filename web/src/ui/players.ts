// In-game co-op overlays on the HUD: the player list (names, host, ping) while Tab is held, and short
// notices at the top left when someone joins or leaves, each fading after `NOTICE_MS`. main.ts feeds
// it the session and the Tab key.

import './players.css';
import type { Coop } from '../net/coop';
import { playerRow } from './coop';
import { h } from './dom';

const NOTICE_MS = 5000;
const REFRESH_MS = 500;

export class PlayerList {
  private readonly list = h('div', 'player-list hidden');
  private readonly rows = h('ul', 'coop-players');
  private readonly notices = h('div', 'coop-notices');
  private lastRender = 0;

  constructor() {
    this.list.append(h('div', 'player-list-title', 'Players'), this.rows);
    document.getElementById('hud')!.append(this.list, this.notices);
  }

  /** Every frame: shows the list while `show` (Tab held in a session), refreshed twice a second. */
  update(coop: Coop | null, show: boolean, now: number): void {
    const visible = show && coop !== null;
    this.list.classList.toggle('hidden', !visible);
    if (!visible || now - this.lastRender < REFRESH_MS) return;
    this.lastRender = now;
    this.rows.replaceChildren(...coop.players().map((p) => playerRow(p)));
  }

  notice(text: string): void {
    const el = h('div', 'coop-notice', text);
    this.notices.append(el);
    setTimeout(() => el.classList.add('out'), NOTICE_MS);
    setTimeout(() => el.remove(), NOTICE_MS + 400);
  }
}
