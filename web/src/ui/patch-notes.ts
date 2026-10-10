// "What's new" in the menu: the player-facing patch notes from `web/src/patch-notes.md` (bundled as text),
// a teaser card of the latest update for the home page, and a dot on the menu button while the latest
// update is unread (the last one read is kept in localStorage). To add an update, write a `## date · title`
// section at the top of that file; nothing here changes.

import './patch-notes.css';
import source from '../patch-notes.md?raw';
import { button, h } from './dom';

const STORAGE_KEY = 'opencraft.notes.seen';
const REPO = 'https://github.com/Sidem/OpenCraft';

export interface Update {
  date: string;
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
      head.append(h('h3', '', u.title), h('time', 'notes-date', formatDate(u.date)));
      const list = h('ul', 'notes-items');
      for (const item of u.items) list.append(h('li', '', item));
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
    card.append(h('span', 'notes-card-label', `What's new · ${formatDate(latest.date)}`), h('strong', '', latest.title));
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

/** Splits the notes file into updates: `## date · title` starts one, `- ` lines are its items. */
export function parse(text: string): Update[] {
  const updates: Update[] = [];
  for (const raw of text.split('\n')) {
    const line = raw.trim();
    if (line.startsWith('## ')) {
      const [date, ...title] = line.slice(3).split(' · ');
      updates.push({ date: date.trim(), title: title.join(' · ').trim() || date.trim(), items: [] });
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

const key = (u: Update) => `${u.date} ${u.title}`;

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
