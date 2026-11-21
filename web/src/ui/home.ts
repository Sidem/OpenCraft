// The Home page's overview for a returning player (`#home-dash` in index.html), three blocks: where you left off (the tip or
// lesson the game would show next, and the research running), shortcut buttons for the screens the keys E P T M open (printed
// with the player's key labels, keyboard-layout.ts), and a few numbers about the world. Everything is read from `game` when
// the page is shown (`refresh`, called by the menu): the world is not running while the menu is open. To add a shortcut: a
// row in the `wireHome` call (main.ts); a number: a row in `stats()`.

import './home.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { playTimeText, researchProgress, waitingText } from './home-text';
import { keyLabels } from './keyboard-layout';
import type { Menu } from './menu';
import { richText, tipParts } from './tip-text';

export interface HomeParts {
  game: Game;
  /** A screen a key opens: its key place (`KeyE`), its name, and what opens it. */
  shortcuts: { code: string; name: string; open: () => void }[];
  /** False while the world is still loading: the shortcuts wait. */
  ready: () => boolean;
  /** Optional one-time deposits upgrade notice (`ui/world-upgrade.ts`). */
  upgrade?: { notice: HTMLElement; refresh(): void };
}

export class HomePanel {
  readonly el = h('div', 'home');
  private readonly next = h('div', 'home-card');
  private readonly stat = h('div', 'home-stats');
  private readonly buttons: HTMLButtonElement[] = [];

  constructor(private readonly parts: HomeParts) {
    const shortcuts = h('div', 'home-shortcuts');
    for (const s of parts.shortcuts) {
      const cap = h('kbd', '', keyLabels.label(s.code));
      cap.dataset.code = s.code;
      const b = button('home-shortcut', '', () => {
        if (parts.ready()) s.open();
      });
      b.append(cap, s.name);
      this.buttons.push(b);
      shortcuts.append(b);
    }
    this.el.append(
      ...(parts.upgrade ? [parts.upgrade.notice] : []),
      section('Where you left off', this.next),
      section('Jump to', shortcuts),
      section('This world', this.stat),
    );
    this.refresh();
  }

  /** Redraws everything from the game; call when the page shows. */
  refresh(): void {
    this.parts.upgrade?.refresh();
    this.buttons.forEach((b) => b.setAttribute('aria-disabled', String(!this.parts.ready())));
    this.next.replaceChildren(...this.where(), this.research());
    this.stat.replaceChildren(...this.stats().map(([value, label]) => stat(value, label)));
  }

  /** The lesson on show, else the first tip not done yet, else a line saying there are none. */
  private where(): HTMLElement[] {
    const g = this.parts.game;
    const lesson = g.lesson_now() !== 0;
    const progress = g.hint_progress();
    const text = lesson ? g.lesson_text() : progress < g.hint_count() ? g.hint_text(progress) : '';
    if (!text) return [h('p', 'home-body home-muted', 'You have seen every tip. The rest is up to you.')];
    const { title, body } = tipParts(text);
    const p = h('p', 'home-body');
    richText(p, body);
    return [h('div', 'home-title', lesson ? g.lesson_head() : title || 'Next tip'), p];
  }

  private research(): HTMLElement {
    const g = this.parts.game;
    const box = h('div', 'home-research');
    const t = g.current_research();
    if (t < 0) {
      box.append(h('span', 'home-muted', 'Nothing is being researched. Press T to choose.'));
      return box;
    }
    const done = g.tech_progress(t);
    const units = g.tech_units(t);
    const endless = g.tech_endless(t);
    const line = h('div');
    line.append(h('b', '', g.tech_name(t)), h('span', 'home-muted', ` · ${researchProgress(done, units, endless)}`));
    const waiting = waitingText(g.research_queue_len());
    if (waiting) line.append(h('span', 'home-muted', ` · ${waiting}`));
    box.append(h('div', 'home-muted', 'Researching'), line);
    if (!endless && units > 0) {
      const fill = h('i');
      fill.style.width = `${Math.round((100 * Math.min(done, units)) / units)}%`;
      const bar = h('div', 'home-bar');
      bar.append(fill);
      box.append(bar);
    }
    return box;
  }

  private stats(): [string, string][] {
    const g = this.parts.game;
    let done = 0;
    let total = 0;
    for (let t = 0; t < g.tech_count(); t++) {
      if (g.tech_endless(t)) continue;
      total++;
      if (g.tech_done(t)) done++;
    }
    return [
      [playTimeText(g.play_seconds()), 'played'],
      [`${done} of ${total}`, 'techs researched'],
      [String(g.analytics_machines()[0] ?? 0), 'machines working'],
    ];
  }
}

/** A screen the menu's shortcuts can open: the same panels the keys open (main.ts), each of which frees the mouse. */
export interface Screen {
  open(): void;
}

/** Puts the overview on the menu's Home page: a shortcut hides the menu and opens its screen; closing that screen plays on. */
export function wireHome(
  menu: Menu,
  game: Game,
  sound: { unlock(): void },
  screens: { code: string; name: string; screen: Screen }[],
  upgrade?: { notice: HTMLElement; refresh(): void },
): void {
  const home = new HomePanel({
    game,
    ready: () => !menu.play.disabled,
    upgrade,
    shortcuts: screens.map((s) => ({
      code: s.code,
      name: s.name,
      open: () => {
        sound.unlock();
        menu.el.classList.add('hidden');
        s.screen.open();
      },
    })),
  });
  document.getElementById('home-dash')!.append(home.el);
  menu.onHome = () => home.refresh();
}

function stat(value: string, label: string): HTMLElement {
  const box = h('div', 'home-stat');
  box.append(h('b', '', value), h('span', '', label));
  return box;
}

function section(title: string, body: HTMLElement): HTMLElement {
  const s = h('section');
  s.append(h('h3', '', title), body);
  return s;
}
