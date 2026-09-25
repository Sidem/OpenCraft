// The sound designer's footer and settings transfer: volume, the "play when I turn a dial" option,
// Copy / Paste settings (with a paste panel that doubles as a manual-copy fallback) and Reset
// everything. Calls `onReplaced` whenever the whole design changed, so the designer re-renders.

import './sound-lab-footer.css';
import type { SoundSystem } from '../audio/sound';
import { defaultDesign, exportDesign, mergeDesign } from '../audio/settings';
import { button, h } from './dom';
import { VolumeControl } from './volume-control';

export class LabFooter {
  /** The footer bar. */
  readonly el = h('div', 'lab-foot');
  /** The paste panel, shown above the footer when needed. */
  readonly paste = h('div', 'lab-paste hidden');
  /** Whether turning a dial replays the sound. */
  playOnChange = true;

  private readonly pasteText = h('textarea');
  private readonly pasteStatus = h('span', 'lab-paste-status');

  constructor(
    private readonly sound: SoundSystem,
    private readonly onReplaced: () => void,
  ) {
    this.buildFooter();
    this.buildPaste();
  }

  private buildFooter(): void {
    const check = h('label', 'lab-check');
    const box = h('input');
    box.type = 'checkbox';
    box.checked = this.playOnChange;
    box.addEventListener('change', () => (this.playOnChange = box.checked));
    check.append(box, 'Play the sound when I turn a dial');

    const copy = button('secondary-btn', 'Copy settings', (b) => void this.copy(b));
    copy.title = 'Copies every sound setting as text, to keep, share, or make the new defaults.';
    const paste = button('secondary-btn', 'Paste settings', () => this.showPaste(''));
    paste.title = 'Load settings copied earlier.';
    let armed = 0;
    const resetAll = button('secondary-btn danger', 'Reset everything', (b) => {
      if (!armed) {
        b.textContent = 'Click again to reset all';
        armed = window.setTimeout(() => {
          armed = 0;
          b.textContent = 'Reset everything';
        }, 3000);
        return;
      }
      clearTimeout(armed);
      armed = 0;
      b.textContent = 'Reset everything';
      this.sound.setDesign(defaultDesign());
      this.onReplaced();
    });
    this.el.append(new VolumeControl(this.sound).el, check, h('span', 'spacer'), copy, paste, resetAll);
  }

  private buildPaste(): void {
    this.pasteText.spellcheck = false;
    this.pasteText.setAttribute('aria-label', 'Sound settings text');
    const row = h('div', 'lab-paste-row');
    row.append(
      button('secondary-btn', 'Apply', () => this.applyPaste()),
      button('secondary-btn', 'Close', () => this.paste.classList.add('hidden')),
      this.pasteStatus,
    );
    this.paste.append(this.pasteText, row);
  }

  private showPaste(text: string, status = 'Paste settings text below, then press Apply.'): void {
    this.paste.classList.remove('hidden');
    this.pasteText.value = text;
    this.pasteStatus.textContent = status;
    this.pasteText.focus();
    if (text) this.pasteText.select();
  }

  private applyPaste(): void {
    let data: unknown;
    try {
      data = JSON.parse(this.pasteText.value);
    } catch {
      this.pasteStatus.textContent = "That doesn't look like copied sound settings.";
      return;
    }
    const next = structuredClone(this.sound.design);
    const applied = mergeDesign(next, data);
    if (!applied) {
      this.pasteStatus.textContent = 'No sound settings found in that text.';
      return;
    }
    this.sound.setDesign(next);
    this.onReplaced();
    this.pasteStatus.textContent = `Applied ${applied} settings.`;
  }

  private async copy(b: HTMLButtonElement): Promise<void> {
    const text = exportDesign(this.sound.design);
    try {
      await navigator.clipboard.writeText(text);
      b.textContent = 'Copied ✓';
      setTimeout(() => (b.textContent = 'Copy settings'), 1500);
    } catch {
      this.showPaste(text, 'Your browser blocked copying: press Ctrl+C to copy the selected text.');
    }
  }
}
