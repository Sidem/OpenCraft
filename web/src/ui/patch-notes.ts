// "What's new" in the menu: the player-facing patch notes from `web/src/patch-notes.md` (bundled as text),
// a teaser card of the latest update for the home page, and a dot on the menu button while the latest
// update is unread (the last one read is kept in localStorage). To add an update, write a
// `## date · vX.Y.Z · title` section at the top of that file (older ones have no version); nothing here changes.
// The game's version is `package.json`'s, put in by Vite (`vite.config.ts`); `web/test/version.test.mjs` checks that
// it, the engine crate's and the newest note's agree.

import './patch-notes.css';
import source from '../patch-notes.md?raw';
import { button, h } from './dom';
import { inlinePieces } from './inline-text';

declare const __APP_VERSION__: string;
/** The game's version, `package.json`'s (rule: `docs/DECISIONS.md`, "Version numbers"). */
export const VERSION = __APP_VERSION__;

const STORAGE_KEY = 'opencraft.notes.seen';
const REPO = 'https://github.com/Sidem/OpenCraft';

export interface Update {
  date: string;
  /** `v0.8.0`, or empty for updates from before version numbers. */
  version: string;
  title: string;
  items: string[];
}

export class PatchNotes {
  readonly updates = parse(source);
  /** The full list, for the "What's new" page. */
  readonly el = h('div', 'notes');
  /** The latest update as a card that opens the page (empty when there are none). */
  readonly teaser = h('div', 'notes-teaser');

  /** `badge` gets a `has-new` class while the latest update is unread (`checkUnread`); `open` shows the page. */
  constructor(private readonly badge: HTMLElement, open: () => void) {
    for (const u of this.updates) {
      const entry = h('article', 'notes-entry');
      const head = h('header', 'notes-head');
      head.append(h('h3', '', u.title), h('time', 'notes-date', stamp(u)));
      const list = h('ul', 'notes-items');
      for (const item of u.items) list.append(noteItem(item));
      entry.append(head, list);
      this.el.append(entry);
    }
    const more = h('a', 'notes-more', 'Every change, in full, on GitHub');
    more.href = `${REPO}/commits/main`;
    more.target = '_blank';
    more.rel = 'noopener';
    this.el.append(more);

    const latest = this.updates[0];
    if (!latest) return;
    const card = button('notes-card', '', open);
    card.append(h('span', 'notes-card-label', `What's new · ${stamp(latest)}`), h('strong', '', latest.title));
    this.teaser.append(card);
  }

  /** Shows the dot if the latest update is unread. A new player has nothing to catch up on. */
  checkUnread(firstVisit: boolean): void {
    const latest = this.updates[0];
    if (!latest) return;
    if (firstVisit && load() === null) this.markRead();
    this.badge.classList.toggle('has-new', load() !== key(latest));
  }

  /** The page was opened: the latest update is read. */
  markRead(): void {
    const latest = this.updates[0];
    if (!latest) return;
    save(key(latest));
    this.badge.classList.remove('has-new');
  }
}

/** Splits the notes file into updates: `## date · vX.Y.Z · title` (the version optional) starts one, `- ` lines
 *  are its items. */
export function parse(text: string): Update[] {
  const updates: Update[] = [];
  for (const raw of text.split('\n')) {
    const line = raw.trim();
    if (line.startsWith('## ')) {
      const [date, ...title] = line.slice(3).split(' · ');
      const version = /^v\d+\.\d+\.\d+$/.test(title[0]?.trim() ?? '') ? title.shift()!.trim() : '';
      updates.push({ date: date.trim(), version, title: title.join(' · ').trim() || date.trim(), items: [] });
    } else if (line.startsWith('- ') && updates.length) {
      updates[updates.length - 1].items.push(line.slice(2));
    } else if (line && !line.startsWith('#') && updates.length) {
      // A wrapped item continues the one above it.
      const items = updates[updates.length - 1].items;
      if (items.length) items[items.length - 1] += ` ${line}`;
    }
  }
  return updates;
}

/** One list item: its **bold** and `code` spans become elements (never innerHTML: the text is data). */
function noteItem(text: string): HTMLLIElement {
  const li = h('li');
  for (const piece of inlinePieces(text)) {
    if (piece.kind === 'plain') li.append(document.createTextNode(piece.text));
    else li.append(h(piece.kind === 'bold' ? 'strong' : 'code', '', piece.text));
  }
  return li;
}

const key = (u: Update) => `${u.date} ${u.title}`;

/** `v0.8.0 · 15 Nov 2026`, or just the date for updates from before version numbers. */
const stamp = (u: Update) => (u.version ? `${u.version} · ${formatDate(u.date)}` : formatDate(u.date));

function formatDate(iso: string): string {
  const d = new Date(`${iso}T12:00:00`);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleDateString(undefined, { dateStyle: 'medium' });
}

// Storage can be unavailable (private windows, blocked site data); the dot then shows every time.
function load(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

function save(value: string): void {
  try {
    localStorage.setItem(STORAGE_KEY, value);
  } catch {
    // Not remembered; see load.
  }
}
