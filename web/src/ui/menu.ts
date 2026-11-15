// The launcher and pause menu (`#menu` in index.html). A side column holds what matters most: the Play
// button with the loading bar, the world it opens, the page buttons and the GitHub links. Beside it one
// page shows at a time: home (a welcome and the first steps for a new player, the latest
// update for a returning one), worlds, play together, settings, sound, controls and what's new. Pages' contents
// come from their own panels, which main.ts builds into each page's container; this file switches pages,
// shows loading and the play state, greets (the returning player's overview is home.ts, drawn when `onHome` fires), shows the game's version beside the title, and starts the key labels
// (keyboard-layout.ts) that the Controls page (controls.ts) and every `kbd[data-code]` print. While the settings page
// shows, the dim and blur go so changes show on the game behind the panel (`previewing`); the panel never moves.
// To add a page: a nav button with `data-tab` and a `.menu-page` with the same `data-page` in index.html.

import './menu.css';
import './menu-pages.css';
import { ControlsPage } from './controls';
import { type Greeting, greeting } from './greetings';
import { keyLabels } from './keyboard-layout';
import { PatchNotes, VERSION } from './patch-notes';
import { wireSettingsTabs } from './settings-tabs';

export type Page = 'home' | 'worlds' | 'together' | 'settings' | 'sound' | 'controls' | 'notes';
const WELCOMED_KEY = 'opencraft.welcomed';

export class Menu {
  readonly el = document.getElementById('menu')!;
  readonly play = document.getElementById('play') as HTMLButtonElement;
  private readonly fill = document.getElementById('loading-fill')!;
  private readonly status = document.getElementById('loading-text')!;
  private readonly worldName = document.getElementById('world-name')!;
  private readonly worldDetail = document.getElementById('world-detail')!;
  private readonly tabs = [...this.el.querySelectorAll<HTMLButtonElement>('[data-tab]')];
  private readonly pages = [...this.el.querySelectorAll<HTMLElement>('[data-page]')];
  private readonly main = this.el.querySelector<HTMLElement>('.menu-main')!;
  private readonly notes: PatchNotes;
  private page: Page = 'home';
  private greeting: Greeting = 'first';
  /** Called when the home page shows and when loading ends: redraws the overview (home.ts), which main.ts sets. */
  onHome: () => void = () => {};

  constructor() {
    for (const tab of this.tabs) tab.addEventListener('click', () => this.show(tab.dataset.tab as Page));
    const notesTab = this.tabs.find((t) => t.dataset.tab === 'notes')!;
    this.notes = new PatchNotes(notesTab, () => this.show('notes'));
    document.getElementById('notes')!.append(this.notes.el);
    document.getElementById('menu-latest')!.append(this.notes.teaser);
    document.getElementById('app-version')!.textContent = `v${VERSION}`;
    wireSettingsTabs(this.el.querySelector<HTMLElement>('[data-page="settings"]')!);
    keyLabels.start();
    document.getElementById('controls-page')!.append(new ControlsPage().el);
  }

  /** A world was opened (`restored`: from a save). A new world for someone who never pressed Play here
   *  means a new player (players from before this menu have saves, so they count as returning). */
  opened(restored: boolean): void {
    const firstVisit = !restored && !welcomed();
    this.greet(firstVisit ? 'first' : restored ? 'back' : 'fresh');
    this.notes.checkUnread(firstVisit);
    this.play.addEventListener('click', welcome, { once: true });
  }

  get isOpen(): boolean {
    return !this.el.classList.contains('hidden');
  }

  /** True while the pause screen shows (the menu opened by Escape during play, on any page): Escape resumes. */
  get isPaused(): boolean {
    return this.isOpen && this.greeting === 'paused';
  }

  /** True while the settings page shows: the vignette and other settings preview on the game (any settings tab). */
  get previewing(): boolean {
    return this.isOpen && this.page === 'settings';
  }

  show(page: Page): void {
    this.page = page;
    for (const tab of this.tabs) {
      const on = tab.dataset.tab === page;
      tab.classList.toggle('active', on);
      tab.toggleAttribute('aria-current', on);
    }
    for (const p of this.pages) p.classList.toggle('hidden', p.dataset.page !== page);
    this.el.classList.toggle('previewing', page === 'settings');
    this.main.scrollTop = 0;
    if (page === 'home') this.onHome();
    if (page === 'notes') this.notes.markRead();
  }

  /** Shows the menu on its home page: the pause screen. */
  pause(): void {
    this.el.classList.remove('hidden');
    this.greet('paused');
    this.play.textContent = 'Resume';
    this.show('home');
  }

  greet(kind: Greeting): void {
    this.greeting = kind;
    const [title, line] = greeting(kind);
    document.getElementById('home-title')!.textContent = title;
    document.getElementById('home-lead')!.textContent = line;
    // A new player sees the first steps; anyone else the latest update.
    this.el.classList.toggle('first-visit', kind === 'first');
  }

  /** Loading progress, 0 to 100; at 100 the bar goes and Play can be clicked. */
  loading(pct: number): void {
    this.fill.style.transform = `scaleX(${pct / 100})`;
    const done = pct >= 100;
    this.say(done ? '' : `Generating terrain... ${pct}%`);
    this.play.disabled = !done;
    this.el.classList.toggle('ready', done);
    if (done) this.onHome();
  }

  /** A line under the Play button: loading steps, or what went wrong (empty hides it). */
  say(text: string): void {
    this.status.textContent = text;
  }

  /** The world Play opens: its name and a line of detail. */
  world(name: string, detail: string): void {
    this.worldName.textContent = name;
    this.worldDetail.textContent = detail;
  }
}

// Storage can be unavailable (private windows, blocked site data); the first steps then show each visit.
function welcomed(): boolean {
  try {
    return localStorage.getItem(WELCOMED_KEY) !== null;
  } catch {
    return false;
  }
}

function welcome(): void {
  try {
    localStorage.setItem(WELCOMED_KEY, '1');
  } catch {
    // Not remembered; see welcomed.
  }
}
