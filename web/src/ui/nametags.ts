// Name tags over other players: one label per anchor the engine reports this frame (`label_ptr`,
// `LABEL_FLOATS` each: player id, then the camera-relative point above the head), placed with the
// renderer's camera. Names come from the co-op session; anyone without one is "Player N".

import './nametags.css';
import type { Renderer } from '../render/renderer';
import { h } from './dom';

const LABEL_FLOATS = 4;

export class NameTags {
  private readonly el = h('div', 'nametags');
  private readonly tags = new Map<number, HTMLDivElement>();

  constructor() {
    document.getElementById('hud')!.append(this.el);
  }

  /** Call after rendering, with this frame's anchors (a view straight into wasm memory). */
  update(anchors: Float32Array, count: number, renderer: Renderer, name: (id: number) => string | undefined): void {
    const seen = new Set<number>();
    for (let i = 0; i < count; i++) {
      const o = i * LABEL_FLOATS;
      const id = anchors[o];
      const at = renderer.project(anchors[o + 1], anchors[o + 2], anchors[o + 3]);
      if (!at) continue;
      seen.add(id);
      let tag = this.tags.get(id);
      if (!tag) {
        tag = h('div', 'nametag');
        this.tags.set(id, tag);
        this.el.append(tag);
      }
      const text = name(id) ?? `Player ${id + 1}`;
      if (tag.textContent !== text) tag.textContent = text;
      tag.style.transform = `translate(${Math.round(at[0])}px, ${Math.round(at[1])}px) translate(-50%, -100%)`;
    }
    for (const [id, tag] of this.tags) {
      if (seen.has(id)) continue;
      tag.remove();
      this.tags.delete(id);
    }
  }
}
