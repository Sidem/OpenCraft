// The admin console: a command line along the bottom of the screen for testing (weather, time, teleport, items). The
// world stays visible and keeps running behind it. `/` or the backquote key opens it (input.ts, main.ts); Enter runs the
// line, Enter on an empty line returns to play, Escape closes it (click to play on), Up and Down walk the history.
// What the commands do is in console/commands.ts; this file is only the text box and the scrolling answers.

import './console.css';
import { runCommand, type ConsoleHost } from '../console/commands';
import { h } from './dom';

/** Lines of answers kept on screen. */
const LOG_LINES = 60;

export class ConsolePanel {
  /** Called after the console closes. `resume` is true when play should continue. */
  onClose: (resume: boolean) => void = () => {};

  private readonly root = h('div', 'console hidden');
  private readonly log = h('div', 'console-log');
  private readonly line = h('input', 'console-line');
  private readonly history: string[] = [];
  private back = -1; // how far up the history the line is (-1: a fresh line)

  constructor(private readonly host: ConsoleHost) {
    this.line.type = 'text';
    this.line.autocomplete = 'off';
    this.line.spellcheck = false;
    this.line.placeholder = 'Type a command: help';
    this.line.setAttribute('aria-label', 'Command line');
    this.root.append(this.log, this.line);
    this.line.addEventListener('keydown', (e) => this.onKey(e));
    this.root.addEventListener('contextmenu', (e) => e.stopPropagation());
    document.body.append(this.root);
  }

  get isOpen(): boolean {
    return !this.root.classList.contains('hidden');
  }

  open(): void {
    this.root.classList.remove('hidden');
    if (this.log.childElementCount === 0) this.print('Type help for the commands.');
    this.line.value = '';
    this.back = -1;
    this.line.focus();
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.root.classList.add('hidden');
    this.line.blur();
    this.onClose(resume);
  }

  private onKey(e: KeyboardEvent): void {
    e.stopPropagation(); // the game's own keys (and a browser's) must not see what is typed
    if (e.key === 'Escape') {
      e.preventDefault();
      this.close(false);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const text = this.line.value.trim();
      if (text === '') {
        this.close(true);
        return;
      }
      this.print(`> ${text}`, 'echo');
      for (const out of runCommand(text, this.host)) this.print(out);
      if (this.history[0] !== text) this.history.unshift(text);
      this.line.value = '';
      this.back = -1;
    } else if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
      e.preventDefault();
      this.back = Math.max(-1, Math.min(this.history.length - 1, this.back + (e.key === 'ArrowUp' ? 1 : -1)));
      this.line.value = this.back < 0 ? '' : this.history[this.back];
    }
  }

  private print(text: string, kind = ''): void {
    this.log.append(h('div', `console-out ${kind}`.trim(), text));
    while (this.log.childElementCount > LOG_LINES) this.log.firstElementChild?.remove();
    this.log.scrollTop = this.log.scrollHeight;
  }
}
