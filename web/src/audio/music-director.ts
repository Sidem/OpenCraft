// The music director: decides what plays when, with no Web Audio in it (music.ts plays what it says). It holds the
// player's place (a `Context`) steady for `HOLD` seconds before music follows it, picks a piece that fits the place and
// the chosen mode (never the one that just played when there is another), and lets silence fall between pieces: minutes
// in "context" mode, a breath in the "always" modes. Tests: web/test/music-director.test.mjs (node --test), which runs
// this file as it is, so keep it to syntax Node can strip types from (no enums, no constructor parameter properties).
// To add a piece, add a row in music-pieces.ts. To add a mode: a `Mode`, a `GAP` row and music.ts's `MODES`.

import type { Context } from './surroundings';

export type Mood = 'calm' | 'lively';
export type Mode = 'context' | 'calm' | 'lively';

/** What the director needs to know of a piece. A piece tagged `'any'` is a fallback for places with no piece of their own. */
export interface PieceInfo {
  id: string;
  mood: Mood;
  contexts: readonly (Context | 'any')[];
}

/** A place must hold this many seconds before the music follows it (a coast line or a cave mouth never flips it). */
export const HOLD = 20;
/** Seconds to wait before the first piece of a session, and between pieces, by mode: [shortest, longest]. */
export const FIRST_WAIT: readonly [number, number] = [10, 30];
export const GAP: Record<Mode, readonly [number, number]> = { context: [10, 30], calm: [4, 12], lively: [4, 12] };
/** Seconds to wait to try again when no piece fits, and after a piece that would not load. */
const RETRY = 10;
const AFTER_FAILURE = 2;

/** How much working machinery lowers the music, 1 (nothing) down to 0.7. */
export const duckFor = (din: number) => 1 - 0.3 * Math.min(1, Math.max(0, din));

export interface Play {
  piece: PieceInfo;
  /** Another piece is playing and is to be faded into this one. */
  crossfade: boolean;
}

export class MusicDirector {
  mode: Mode;
  /** The piece the director believes is playing. */
  playing: PieceInfo | null = null;
  /** The place the music follows (the latest place held for `HOLD` seconds). */
  committed: Context | null = null;

  private raw: Context | null = null;
  private held = 0;
  private wait: number;
  private last: string | null = null;
  private reselect = false;
  private readonly bad = new Set<string>();
  private readonly pieces: readonly PieceInfo[];
  private readonly rand: () => number;

  /** `rand` returns 0 to 1 (`Math.random`, or a seeded one in tests). */
  constructor(pieces: readonly PieceInfo[], rand: () => number, mode: Mode = 'context') {
    this.pieces = pieces;
    this.rand = rand;
    this.mode = mode;
    this.wait = this.between(mode === 'context' ? FIRST_WAIT : [0, 3]);
  }

  /** Advances `dt` seconds with the place the player is in now; returns the piece to start, if any. */
  update(dt: number, now: Context): Play | null {
    if (now !== this.raw) {
      this.raw = now;
      this.held = 0;
    } else {
      this.held += dt;
    }
    let moved = false;
    if (this.committed === null) {
      this.committed = now;
    } else if (now !== this.committed && this.held >= HOLD) {
      this.committed = now;
      moved = true;
    }
    if (this.playing) {
      if (!moved && !this.reselect) return null;
      this.reselect = false;
      const pool = this.pool(this.committed);
      if (pool.length === 0 || pool.includes(this.playing)) return null;
      return this.start(this.choose(pool), true);
    }
    this.wait -= dt;
    if (this.wait > 0) return null;
    const pool = this.pool(this.committed);
    if (pool.length === 0) {
      this.wait = RETRY;
      return null;
    }
    return this.start(this.choose(pool), false);
  }

  /** The piece finished playing: silence until the next. */
  ended(): void {
    this.playing = null;
    this.wait = this.between(GAP[this.mode]);
  }

  /** The piece could not be played: never choose it again, and try another soon. */
  failed(id: string): void {
    this.bad.add(id);
    this.playing = null;
    this.wait = AFTER_FAILURE;
  }

  /** The player picked another mode: a playing piece that no longer fits is faded out, silence is cut short. */
  setMode(mode: Mode): void {
    if (mode === this.mode) return;
    this.mode = mode;
    if (this.playing) this.reselect = true;
    else this.wait = Math.min(this.wait, this.between(mode === 'context' ? FIRST_WAIT : [0, 3]));
  }

  /** The pieces that suit `place` in the current mode: the ones made for it, else the fallbacks, else (in an "always" mode) any of the mood. */
  private pool(place: Context): PieceInfo[] {
    const ok = this.pieces.filter((p) => !this.bad.has(p.id) && (this.mode === 'context' || p.mood === this.mode));
    const own = ok.filter((p) => p.contexts.includes(place));
    if (own.length > 0) return own;
    const any = ok.filter((p) => p.contexts.includes('any'));
    if (any.length > 0) return any;
    return this.mode === 'context' ? [] : ok;
  }

  private choose(pool: PieceInfo[]): PieceInfo {
    const fresh = pool.filter((p) => p.id !== this.last && p !== this.playing);
    const from = fresh.length > 0 ? fresh : pool;
    return from[Math.min(from.length - 1, Math.floor(this.rand() * from.length))];
  }

  private start(piece: PieceInfo, crossfade: boolean): Play {
    this.playing = piece;
    this.last = piece.id;
    return { piece, crossfade };
  }

  private between(range: readonly [number, number]): number {
    return range[0] + (range[1] - range[0]) * this.rand();
  }
}
