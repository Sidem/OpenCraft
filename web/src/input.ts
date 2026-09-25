// Keyboard and mouse input. Held state (movement, mining, using) is read every frame; one-shot
// keys become `Action`s that main.ts drains with `takeActions()`. Gameplay input is captured only
// while the pointer is locked to the canvas. To add a key: an `Action` variant, a line in the
// keydown handler, and its handling in main.ts's action loop.

export type Action =
  | { kind: 'slot'; slot: number }
  | { kind: 'scroll'; delta: number }
  | { kind: 'fly' }
  | { kind: 'drop' }
  | { kind: 'mute' }
  | { kind: 'sound-lab' }
  | { kind: 'inventory' }
  | { kind: 'research' }
  | { kind: 'hint' }
  | { kind: 'debug' };

export interface Movement {
  forward: number;
  strafe: number;
  jump: boolean;
  sprint: boolean;
  crouch: boolean;
}

/** Keyboard/mouse state. Gameplay input is only captured while the pointer is locked to the canvas. */
export class Input {
  locked = false;
  mining = false;
  using = false;
  onLockChange: (locked: boolean) => void = () => {};

  private readonly keys = new Set<string>();
  private lookX = 0;
  private lookY = 0;
  private actions: Action[] = [];

  constructor(private readonly canvas: HTMLCanvasElement) {
    document.addEventListener('pointerlockchange', () => {
      this.locked = document.pointerLockElement === canvas;
      if (!this.locked) this.reset();
      this.onLockChange(this.locked);
    });
    window.addEventListener('keydown', (e) => this.onKey(e, true));
    window.addEventListener('keyup', (e) => this.onKey(e, false));
    window.addEventListener('blur', () => this.reset());
    document.addEventListener('mousemove', (e) => {
      if (!this.locked) return;
      this.lookX += e.movementX;
      this.lookY += e.movementY;
    });
    canvas.addEventListener('mousedown', (e) => {
      if (!this.locked) return;
      if (e.button === 0) this.mining = true;
      else if (e.button === 2) this.using = true;
      e.preventDefault();
    });
    window.addEventListener('mouseup', (e) => {
      if (e.button === 0) this.mining = false;
      else if (e.button === 2) this.using = false;
    });
    canvas.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener(
      'wheel',
      (e) => {
        if (!this.locked) return;
        e.preventDefault();
        if (e.deltaY !== 0) this.actions.push({ kind: 'scroll', delta: Math.sign(e.deltaY) });
      },
      { passive: false },
    );
  }

  /** Requests raw (unaccelerated) mouse input where supported, falling back to plain pointer lock. */
  async lock(): Promise<void> {
    const request = this.canvas.requestPointerLock as (opts?: { unadjustedMovement?: boolean }) => Promise<void> | void;
    try {
      await request.call(this.canvas, { unadjustedMovement: true });
    } catch {
      try {
        await request.call(this.canvas);
      } catch {
        // Browsers refuse re-locking for ~1s after Escape; the user can simply click again.
      }
    }
  }

  movement(): Movement {
    const k = (code: string) => (this.keys.has(code) ? 1 : 0);
    return {
      forward: Math.max(k('KeyW'), k('ArrowUp')) - Math.max(k('KeyS'), k('ArrowDown')),
      strafe: Math.max(k('KeyD'), k('ArrowRight')) - Math.max(k('KeyA'), k('ArrowLeft')),
      jump: this.keys.has('Space'),
      sprint: this.keys.has('ShiftLeft') || this.keys.has('ShiftRight'),
      crouch: this.keys.has('KeyC'),
    };
  }

  takeLook(): [number, number] {
    const d: [number, number] = [this.lookX, this.lookY];
    this.lookX = this.lookY = 0;
    return d;
  }

  takeActions(): Action[] {
    const a = this.actions;
    this.actions = [];
    return a;
  }

  private onKey(e: KeyboardEvent, down: boolean): void {
    if (e.code === 'F3') {
      e.preventDefault();
      if (down && !e.repeat) this.actions.push({ kind: 'debug' });
      return;
    }
    if (!this.locked) return;
    if (down) this.keys.add(e.code);
    else this.keys.delete(e.code);
    if (down && !e.repeat) {
      const digit = /^Digit([1-9])$/.exec(e.code);
      if (digit) this.actions.push({ kind: 'slot', slot: Number(digit[1]) - 1 });
      else if (e.code === 'KeyF') this.actions.push({ kind: 'fly' });
      else if (e.code === 'KeyQ') this.actions.push({ kind: 'drop' });
      else if (e.code === 'KeyM') this.actions.push({ kind: 'mute' });
      else if (e.code === 'KeyO') this.actions.push({ kind: 'sound-lab' });
      else if (e.code === 'KeyE') this.actions.push({ kind: 'inventory' });
      else if (e.code === 'KeyR') this.actions.push({ kind: 'research' });
      else if (e.code === 'KeyH') this.actions.push({ kind: 'hint' });
    }
    if (e.code === 'Space' || e.code === 'Tab' || e.code.startsWith('Arrow')) e.preventDefault();
  }

  private reset(): void {
    this.keys.clear();
    this.mining = false;
    this.using = false;
    this.lookX = this.lookY = 0;
  }
}
