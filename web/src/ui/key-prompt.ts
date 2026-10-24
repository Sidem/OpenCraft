// A floating input prompt in the world (the way games point at "press this here"): a key cap or a mouse with
// the button to use lit, and one word under it, hanging over the thing to act on and bobbing gently. The engine
// says where and which (`opening_marker`: a camera-relative point and a `PROMPT_*` kind from `opening.rs`;
// `lesson_marker` the same for a stalled machine, kind `PROMPT_CHECK` from `stall_lesson.rs`);
// this places it with the renderer's camera each frame. To add a prompt: a row in `KINDS` and its engine kind.

import './key-prompt.css';
import type { Renderer } from '../render/renderer';
import { h } from './dom';

type Icon = { mouse: 'left' | 'right' } | { key: string };

/** By the engine's `PROMPT_*` number. */
const KINDS: Record<number, { icon: Icon; label: string }> = {
  1: { icon: { mouse: 'left' }, label: 'Hold' },
  2: { icon: { mouse: 'right' }, label: 'Place' },
  3: { icon: { key: 'R' }, label: 'Turn' },
  4: { icon: { mouse: 'right' }, label: 'Open' },
  5: { icon: { mouse: 'right' }, label: 'Check' },
};

export class KeyPrompt {
  private readonly el = h('div', 'key-prompt hidden');
  private kind = 0;

  constructor() {
    document.getElementById('hud')!.append(this.el);
  }

  /** Call after rendering with the engine's marker (empty for none). */
  update(marker: Float32Array | number[], renderer: Renderer): void {
    const at = marker.length >= 4 ? renderer.project(marker[0], marker[1], marker[2]) : null;
    const kind = at ? marker[3] : 0;
    this.el.classList.toggle('hidden', !at || !KINDS[kind]);
    if (!at || !KINDS[kind]) return;
    if (kind !== this.kind) this.draw(kind);
    this.el.style.transform = `translate(${Math.round(at[0])}px, ${Math.round(at[1])}px) translate(-50%, -100%)`;
  }

  private draw(kind: number): void {
    this.kind = kind;
    const { icon, label } = KINDS[kind];
    const face = h('div', 'key-prompt-bob');
    if ('key' in icon) {
      face.append(h('div', 'key-prompt-cap', icon.key));
    } else {
      const mouse = h('div', `key-prompt-mouse lit-${icon.mouse}`);
      mouse.append(h('span', 'key-prompt-btn left'), h('span', 'key-prompt-btn right'));
      face.append(mouse);
    }
    face.append(h('div', 'key-prompt-label', label));
    this.el.replaceChildren(face);
  }
}
