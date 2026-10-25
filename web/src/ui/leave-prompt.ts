// "Ask before closing" (settings page, off by default): a checkbox, and the prompt it enables. A page cannot show its
// own question while a tab closes, only the browser's generic "Leave site?" box, so with the setting on and the world
// changed since the last export, `beforeunload` asks the browser to hold the closing and this file opens a dialog
// behind it: whoever chooses to stay finds "Export this world / Export all worlds" waiting. The rule is in
// `save/leave-guard.ts`; exporting is `save/export.ts`. Never built for a co-op client (the world is the host's).

import './leave-prompt.css';
import type { Game } from '../wasm/engine.js';
import { exportAll, exportWorld } from '../save/export';
import { holdsClosing, loadAskBeforeClosing, saveAskBeforeClosing } from '../save/leave-guard';
import { message, type Session } from '../save/session';
import type { WorldStore } from '../save/store';
import { button, h } from './dom';

export class LeavePrompt {
  /** The settings row: a checkbox with its explanation. */
  readonly row = h('div', 'leave-row');
  private enabled = loadAskBeforeClosing();
  /** Game time up to which the world is covered: loaded, exported, or "not now". */
  private covered: number;
  private readonly dialog = h('div', 'leave-backdrop hidden');
  private readonly status = h('p', 'leave-status');

  constructor(
    private readonly game: Game,
    private readonly store: WorldStore,
    private readonly session: Session,
  ) {
    this.covered = game.play_seconds();

    const box = h('input');
    box.type = 'checkbox';
    box.checked = this.enabled;
    box.addEventListener('change', () => {
      this.enabled = box.checked;
      saveAskBeforeClosing(this.enabled);
    });
    const label = h('label', 'leave-check');
    label.append(box, h('span', '', 'Ask me to export my worlds before the tab closes'));
    this.row.append(
      label,
      h(
        'p',
        'comfort-help',
        'For browsers set to delete all site data when they close, which would erase your worlds. The browser shows its own ' +
          '"Leave site?" box when you close the tab; choose Stay and the game offers an export. Off by default. A browser that ' +
          'clears its data also clears this choice, so turn it on again each visit.',
      ),
    );

    const card = h('div', 'leave-card');
    card.setAttribute('role', 'dialog');
    card.setAttribute('aria-label', 'Export before closing');
    card.append(
      h('h2', '', 'Export a copy before you go?'),
      h('p', '', "Closing this tab may delete your saved worlds if your browser clears site data. Exporting downloads them as .ocworld files you can import later."),
      this.buttons(),
      this.status,
    );
    this.dialog.append(card);
    document.body.append(this.dialog);

    window.addEventListener('beforeunload', (e) => {
      if (!holdsClosing(this.enabled, this.game.play_seconds(), this.covered)) return;
      e.preventDefault();
      e.returnValue = '';
      this.status.textContent = '';
      this.dialog.classList.remove('hidden');
    });
  }

  private buttons(): HTMLElement {
    const row = h('div', 'leave-buttons');
    row.append(
      button('secondary-btn active', 'Export this world', () => this.run(async () => {
        await exportWorld(this.store, this.session, this.session.meta);
        return `Exported "${this.session.meta.name}".`;
      })),
      button('secondary-btn', 'Export all worlds', () => this.run(async () => {
        const n = await exportAll(this.store, this.session);
        return `Exported ${n} world${n === 1 ? '' : 's'}.`;
      })),
      button('secondary-btn', 'Not now', () => {
        this.covered = this.game.play_seconds();
        this.dialog.classList.add('hidden');
      }),
    );
    return row;
  }

  /** Runs an export; afterwards the world counts as covered and the player is told they can close the tab. */
  private run(task: () => Promise<string>): void {
    this.status.textContent = 'Exporting...';
    task()
      .then((done) => {
        this.covered = this.game.play_seconds();
        this.status.textContent = `${done} You can close the tab now.`;
      })
      .catch((err: unknown) => (this.status.textContent = message(err)));
  }
}
