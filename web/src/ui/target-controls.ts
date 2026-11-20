// Structured control chips for top-of-screen HUD readouts (#target-controls in hud.ts, and the scanner pointer in
// prospect.ts). Engine labels put control lines on `\t`-prefixed rows (`\tKEY=Label\t...`) so status text and controls
// stay visually distinct: status stays concise prose above, and controls render below as crisp keycaps and mouse-button
// glyphs matching the player's keyboard layout (`keyboard-layout.ts`). Pure parser (`parseHudText`) is tested in
// web/test/target-controls.test.mjs.

import { keyLabels } from './keyboard-layout.ts';

export interface HudControl {
  /** Key specification: e.g. `RMB`, `Hold Shift`, `Ctrl+RMB`, `Ctrl+RMB/R`, `Wheel`, `R`, `Z`, `Enter`. */
  spec: string;
  /** Short 1-3 word action label: e.g. `Place`, `Full reach`, `Sever wire`. */
  label: string;
}

export interface ParsedHud {
  /** Non-control status lines joined by `\n`. */
  detail: string;
  /** Control chips collected from every `\t`-prefixed line. */
  controls: HudControl[];
}

/** Splits a multi-line HUD text into plain status lines (`detail`) and structured `\tKEY=Label` control chips. */
export function parseHudText(text: string): ParsedHud {
  if (!text) return { detail: '', controls: [] };
  const lines: string[] = [];
  const controls: HudControl[] = [];
  for (const raw of text.split('\n')) {
    if (raw.startsWith('\t')) {
      for (const entry of raw.slice(1).split('\t')) {
        const eq = entry.indexOf('=');
        if (eq > 0) controls.push({ spec: entry.slice(0, eq).trim(), label: entry.slice(eq + 1).trim() });
      }
    } else if (raw !== '') {
      lines.push(raw);
    }
  }
  return { detail: lines.join('\n'), controls };
}

/** Resolves a single key token (`R`, `Z`, `Shift`, `Ctrl`, `Enter`, ...) to the label printed on the player's keyboard. */
export function keyCaption(token: string, labelOf: (code: string) => string = (c) => keyLabels.label(c)): string {
  if (/^[A-Z]$/.test(token)) return labelOf(`Key${token}`) || token;
  return token;
}

/** Builds one `[keys] Label` chip element. */
export function renderControlChip(control: HudControl): HTMLElement {
  const item = document.createElement('span');
  item.className = 'tc-item';
  const keys = document.createElement('span');
  keys.className = 'tc-keys';
  const alts = control.spec.split('/');
  alts.forEach((alt, ai) => {
    if (ai > 0) keys.append(sep('/'));
    const parts = alt.trim().split('+');
    parts.forEach((rawPart, pi) => {
      if (pi > 0) keys.append(sep('+'));
      const part = rawPart.trim();
      const modMatch = /^(Hold|Release)\s+(.+)$/.exec(part);
      if (modMatch) {
        const mod = document.createElement('span');
        mod.className = 'tc-mod';
        mod.textContent = modMatch[1];
        keys.append(mod, renderToken(modMatch[2].trim()));
      } else {
        keys.append(renderToken(part));
      }
    });
  });
  const label = document.createElement('span');
  label.className = 'tc-label';
  label.textContent = control.label;
  item.append(keys, label);
  return item;
}

/** Replaces `el`'s children with rendered chips for `controls`, toggling `hidden` when empty. */
export function renderHudControls(el: HTMLElement, controls: HudControl[]): void {
  el.classList.toggle('hidden', controls.length === 0);
  if (controls.length === 0) {
    el.replaceChildren();
    return;
  }
  el.replaceChildren(...controls.map(renderControlChip));
}

function sep(text: string): HTMLSpanElement {
  const s = document.createElement('span');
  s.className = 'tc-sep';
  s.textContent = text;
  return s;
}

/** One visual glyph: a mouse icon for LMB / RMB / Wheel, or a keycap for keyboard keys. */
function renderToken(token: string): HTMLElement {
  if (token === 'LMB' || token === 'RMB' || token === 'Wheel') {
    const mouse = document.createElement('span');
    const kind = token === 'LMB' ? 'tc-left' : token === 'RMB' ? 'tc-right' : 'tc-scroll';
    mouse.className = `tc-mouse ${kind}`;
    mouse.title = token === 'LMB' ? 'Left mouse button' : token === 'RMB' ? 'Right mouse button' : 'Mouse wheel';
    const left = document.createElement('span');
    left.className = 'tc-btn';
    const right = document.createElement('span');
    right.className = 'tc-btn';
    mouse.append(left, right);
    if (token === 'Wheel') {
      const wheel = document.createElement('span');
      wheel.className = 'tc-wheel';
      mouse.append(wheel);
    }
    return mouse;
  }
  const cap = document.createElement('kbd');
  cap.className = 'tc-cap';
  cap.textContent = keyCaption(token);
  if (token === 'Ctrl') cap.title = 'Ctrl or C (crouch)';
  return cap;
}
