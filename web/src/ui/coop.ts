// "Play together" in the pause menu: the player's name, then by state:
//   solo    Host this world (opens it over WebRTC and shows the code and link), or join from a pasted
//           invite link or code (saves this world, then reloads into the host's)
//   host    the code and link to share, and who is here with their ping
//   client  whose world this is, who is here, and Leave
//   ended   why the session ended, and the way back to this player's own worlds
// Sessions start and run in `net/`; main.ts wires the callbacks. Re-renders the player rows once a
// second while the menu shows.

import './coop.css';
import { CoopClient } from '../net/client';
import type { Coop, PlayerInfo } from '../net/coop';
import { CoopHost } from '../net/host';
import { inviteLink, playerName, roomOf, setPlayerName } from '../net/session';
import { message, soloUrl } from '../save/session';
import { button, h } from './dom';

const REFRESH_MS = 1000;

export interface CoopActions {
  /** Opens this world to others and starts the session. */
  host(): Promise<Coop>;
  /** Saves and stops the solo world before the page leaves for a host's. */
  leave(): Promise<void>;
}

export class CoopPanel {
  readonly el = h('section', 'coop');
  private readonly body = h('div', 'coop-body');
  private readonly status = h('p', 'coop-status');
  private ended = '';
  private lastRender = 0;

  constructor(
    private coop: Coop | null,
    private readonly actions: CoopActions,
  ) {
    const name = h('input', 'coop-name');
    name.value = playerName();
    name.placeholder = 'Your name';
    name.maxLength = 24;
    name.addEventListener('change', () => {
      setPlayerName(name.value);
      if (this.coop) this.say('Others see your new name the next time you host or join.');
    });
    const head = h('div', 'coop-head');
    head.append(h('h2', '', 'Play together'), name);
    this.el.append(head, this.body, this.status);
    this.render();
  }

  /** A session started elsewhere (hosting from the menu goes through `actions.host`). */
  started(coop: Coop): void {
    this.coop = coop;
    this.render();
  }

  /** The session ended from the other side: show why, and the way back. */
  end(reason: string): void {
    this.ended = reason;
    this.render();
  }

  /** Every frame; refreshes the player rows (ping) now and then while the menu shows. */
  update(now: number): void {
    if (!this.coop || this.ended || now - this.lastRender < REFRESH_MS || !this.el.offsetParent) return;
    this.render();
  }

  private render(): void {
    this.lastRender = performance.now();
    const coop = this.coop;
    if (this.ended) {
      this.body.replaceChildren(
        h('p', 'coop-ended', this.ended),
        button('secondary-btn active', 'Back to my worlds', () => location.assign(soloUrl())),
      );
    } else if (coop instanceof CoopHost) {
      this.body.replaceChildren(this.invite(coop.code), this.list(coop.players()));
    } else if (coop instanceof CoopClient) {
      const leave = button('secondary-btn', 'Leave', () => {
        coop.close();
        location.assign(soloUrl());
      });
      const line = h('div', 'coop-row');
      const host = coop.hostName();
      line.append(h('span', 'coop-line', host ? `In ${host}'s world` : "In the host's world"), leave);
      this.body.replaceChildren(line, this.list(coop.players()));
    } else {
      this.body.replaceChildren(this.solo());
    }
  }

  private solo(): HTMLElement {
    const wrap = h('div', 'coop-solo');
    const host = button('secondary-btn active', 'Host this world', (b) => {
      b.disabled = true;
      this.say('Opening your world to others...');
      this.actions
        .host()
        .then((coop) => {
          this.say('');
          this.started(coop);
        })
        .catch((err: unknown) => {
          b.disabled = false;
          this.say(message(err));
        });
    });
    const form = h('form', 'coop-join');
    const input = h('input');
    input.placeholder = 'Invite link or code';
    form.append(input, h('button', 'secondary-btn', 'Join'));
    form.addEventListener('submit', (e) => {
      e.preventDefault();
      const room = roomOf(input.value);
      if (!room) return this.say("That isn't an invite link or a 6-character code.");
      this.say('Saving your world...');
      const url = new URL(soloUrl());
      url.searchParams.set('join', room);
      this.actions
        .leave()
        .then(() => location.assign(url))
        .catch((err: unknown) => this.say(message(err)));
    });
    wrap.append(host, form);
    return wrap;
  }

  private invite(code: string): HTMLElement {
    const wrap = h('div', 'coop-invite');
    const link = h('input', 'coop-link');
    link.readOnly = true;
    link.value = inviteLink(code);
    link.addEventListener('focus', () => link.select());
    const copy = button('secondary-btn', 'Copy link', () => {
      navigator.clipboard.writeText(link.value).then(
        () => this.say('Link copied. Send it to a friend.'),
        () => this.say('Copy the link above by hand.'),
      );
    });
    const row = h('div', 'coop-row');
    row.append(link, copy);
    const line = h('div', 'coop-line', 'Friends join with the code ');
    line.append(h('strong', 'coop-code', code));
    wrap.append(line, row);
    return wrap;
  }

  private list(players: PlayerInfo[]): HTMLElement {
    const ul = h('ul', 'coop-players');
    for (const p of players) ul.append(playerRow(p));
    return ul;
  }

  private say(text: string): void {
    this.status.textContent = text;
  }
}

/** One player: name, a tag for you or the host, and the round trip. Shared with the Tab list. */
export function playerRow(p: PlayerInfo): HTMLLIElement {
  const li = h('li', p.you ? 'coop-player you' : 'coop-player');
  li.append(h('span', 'coop-player-name', p.name));
  if (p.host || p.you) li.append(h('span', 'coop-tag', p.host && p.you ? 'you, host' : p.host ? 'host' : 'you'));
  li.append(h('span', 'coop-ping', p.host ? '' : p.ping === null ? '…' : `${p.ping} ms`));
  return li;
}
