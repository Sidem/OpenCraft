// The command line and co-op chat: a text line along the bottom of the screen with the answers and chat above it. The
// world stays visible and keeps running behind it. Enter opens it empty, `/` or the backquote key with a `/` typed
// (input.ts, main.ts). A line starting with `/` or `!` is a command (console/commands.ts runs it, for testing: weather,
// time, teleport, items); any other line is said to everyone in a co-op world (`say`, wired in main.ts) and closes the
// line. Enter on an empty line returns to play, Escape closes it (click to play on), Up and Down walk the history.
// Lines others say arrive through `hear`: here while open, in the chat feed (chat-feed.ts) while closed.

import './console.css';
import { lineKind, runCommand, type ConsoleHost } from '../console/commands';
import { ChatFeed } from './chat-feed';
import { h } from './dom';

/** Lines of answers kept on screen. */
const LOG_LINES = 60;

export class ConsolePanel {
  /** Called after the console closes. `resume` is true when play should continue. */
  onClose: (resume: boolean) => void = () => {};
  /** Sends a chat line to everyone; null outside a co-op world. */
  say: ((text: string) => void) | null = null;

  private readonly root = h('div', 'console hidden');
  private readonly log = h('div', 'console-log');
  private readonly line = h('input', 'console-line');
  private readonly feed = new ChatFeed();
  private readonly history: string[] = [];
  private back = -1; // how far up the history the line is (-1: a fresh line)

  constructor(private readonly host: ConsoleHost) {
    this.line.type = 'text';
    this.line.autocomplete = 'off';
    this.line.spellcheck = false;
    this.line.maxLength = 300;
    this.line.setAttribute('aria-label', 'Chat and command line');
    this.root.append(this.log, this.line);
    this.line.addEventListener('keydown', (e) => this.onKey(e));
    this.root.addEventListener('contextmenu', (e) => e.stopPropagation());
    document.body.append(this.root);
  }

  get isOpen(): boolean {
    return !this.root.classList.contains('hidden');
  }

  /** Opens the line with `typed` already in it (`/` when a command key opened it). */
  open(typed = ''): void {
    this.root.classList.remove('hidden');
    this.feed.show(false);
    this.line.placeholder = this.say ? 'Say something, or /help for commands' : 'Type /help for the commands';
    this.line.value = typed;
    this.back = -1;
    this.line.focus();
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.root.classList.add('hidden');
    this.feed.show(true);
    this.line.blur();
    this.onClose(resume);
  }

  /** A chat line from `from` (this player's own lines come back here too). */
  hear(from: string, text: string): void {
    const row = h('div', 'console-out chat');
    row.append(h('span', 'chat-from', `${from}:`), ` ${text}`);
    this.append(row);
    if (!this.isOpen) this.feed.add(from, text);
  }

  private onKey(e: KeyboardEvent): void {
    e.stopPropagation(); // the game's own keys (and a browser's) must not see what is typed
    if (e.key === 'Escape') {
      e.preventDefault();
      this.close(false);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      this.enter(this.line.value.trim());
    } else if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
      e.preventDefault();
      this.back = Math.max(-1, Math.min(this.history.length - 1, this.back + (e.key === 'ArrowUp' ? 1 : -1)));
      this.line.value = this.back < 0 ? '' : this.history[this.back];
    }
  }

  private enter(text: string): void {
    const kind = lineKind(text);
    if (kind === 'empty') return this.close(true);
    if (this.history[0] !== text) this.history.unshift(text);
    this.line.value = '';
    this.back = -1;
    if (kind === 'command') {
      this.print(`> ${text}`, 'echo');
      for (const out of runCommand(text, this.host)) this.print(out);
    } else if (this.say) {
      this.say(text);
      this.close(true);
    } else {
      this.print(`> ${text}`, 'echo');
      this.print('Commands start with / or ! (try /help). Other lines are chat, for a world shared with friends.');
    }
  }

  private print(text: string, kind = ''): void {
    this.append(h('div', `console-out ${kind}`.trim(), text));
  }

  private append(row: HTMLElement): void {
    this.log.append(row);
    while (this.log.childElementCount > LOG_LINES) this.log.firstElementChild?.remove();
    this.log.scrollTop = this.log.scrollHeight;
  }
}
