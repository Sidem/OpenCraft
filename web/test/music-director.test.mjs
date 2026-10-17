// Tests for the music director and the place reading (web/src/audio/music-director.ts, surroundings.ts), run with
// `node --test web/test` (Node 24 runs the TypeScript directly). They replay scripted walks through a fake clock:
// no audio, no browser. Each helper `walk` plays the part of music.ts: it starts a piece when told to and reports it
// ended after its length.

import assert from 'node:assert/strict';
import test from 'node:test';
import { duckFor, FIRST_WAIT, GAP, HOLD, MusicDirector } from '../src/audio/music-director.ts';
import { PIECES } from '../src/audio/music-pieces.ts';
import { contextFor, S } from '../src/audio/surroundings.ts';

const DT = 0.5;
const LENGTH = 150;

function seeded(seed) {
  let a = seed;
  return () => {
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Runs `seconds` with the place `placeAt(t)`; returns each start {t, id, crossfade} and the times pieces ended. */
function walk(director, seconds, placeAt, onTick) {
  const starts = [];
  const ends = [];
  let endsAt = Infinity;
  for (let t = 0; t < seconds; t += DT) {
    onTick?.(t);
    if (t >= endsAt) {
      director.ended();
      ends.push(t);
      endsAt = Infinity;
    }
    const play = director.update(DT, placeAt(t));
    if (play) {
      starts.push({ t, id: play.piece.id, crossfade: play.crossfade, mood: play.piece.mood });
      endsAt = t + LENGTH;
    }
  }
  return { starts, ends };
}

const make = (seed = 1, mode = 'context') => new MusicDirector(PIECES, seeded(seed), mode);

test('the library covers every place and both moods', () => {
  for (const place of ['day', 'night', 'underground', 'desert', 'highlands', 'sea']) {
    assert.ok(PIECES.some((p) => p.contexts.includes(place) || p.contexts.includes('any')), `a piece for ${place}`);
  }
  for (const mood of ['calm', 'lively']) assert.ok(PIECES.some((p) => p.mood === mood), `a ${mood} piece`);
  assert.equal(new Set(PIECES.map((p) => p.id)).size, PIECES.length, 'ids are unique');
});

test('a reading falls in one place: caves, then night, then the biome, then the sea', () => {
  const reading = (o) => {
    const s = new Array(9).fill(0);
    s[S.sky] = 1;
    for (const [k, v] of Object.entries(o)) s[S[k]] = v;
    return s;
  };
  const NOON = 0.5;
  const MIDNIGHT = 0;
  assert.equal(contextFor(reading({}), NOON), 'day');
  assert.equal(contextFor(reading({}), MIDNIGHT), 'night');
  assert.equal(contextFor(reading({ biome: 1 }), NOON), 'desert');
  assert.equal(contextFor(reading({ biome: 2 }), NOON), 'highlands');
  assert.equal(contextFor(reading({ biome: 1 }), MIDNIGHT), 'night');
  assert.equal(contextFor(reading({ sea: 1 }), NOON), 'sea');
  assert.equal(contextFor(reading({ sky: 0, depth: 20, biome: 1 }), MIDNIGHT), 'underground');
  assert.equal(contextFor(reading({ sky: 0, depth: 2 }), NOON), 'day', 'a shallow overhang is not a cave');
});

test('the first piece comes after a short wait, then silence of two to five minutes falls between pieces', () => {
  const d = make(3);
  const { starts, ends } = walk(d, 4 * 3600, () => 'day');
  assert.ok(starts[0].t >= FIRST_WAIT[0] - DT && starts[0].t <= FIRST_WAIT[1] + DT, `first at ${starts[0].t}`);
  assert.ok(starts.length > 30, `${starts.length} pieces in four hours`);
  for (let i = 1; i < starts.length; i++) {
    const gap = starts[i].t - ends[i - 1];
    assert.ok(gap >= GAP.context[0] - DT && gap <= GAP.context[1] + DT, `gap ${gap} before piece ${i}`);
  }
});

test('the same piece never plays twice in a row while another fits', () => {
  const { starts } = walk(make(7), 20 * 3600, () => 'day');
  assert.ok(starts.length > 100);
  for (let i = 1; i < starts.length; i++) assert.notEqual(starts[i].id, starts[i - 1].id, `piece ${i}`);
});

test('music follows a new place only after it has held for HOLD seconds, and fades into its piece', () => {
  const { starts } = walk(make(2), 400, (t) => (t < 100 ? 'day' : 'underground'));
  const follow = starts.find((s) => s.id === 'underground');
  assert.ok(follow, 'the cave piece starts');
  assert.ok(follow.crossfade, 'it fades in over the day piece');
  assert.ok(follow.t >= 100 + HOLD - DT && follow.t <= 100 + HOLD + 2 * DT, `at ${follow.t}`);
});

test('a place that flickers every two seconds never moves the music', () => {
  const { starts } = walk(make(5), 900, (t) => (Math.floor(t / 2) % 2 ? 'underground' : 'day'));
  assert.ok(starts.length >= 1);
  assert.ok(starts.every((s) => !s.crossfade), 'no crossfade');
  assert.ok(starts.every((s) => s.id !== 'underground'), 'never the cave piece');
});

test('a place with no piece of its own keeps the piece that is playing', () => {
  const { starts } = walk(make(4), 300, (t) => (t < 100 ? 'day' : 'sea'));
  assert.ok(starts.every((s) => !s.crossfade), 'the day piece plays on at the coast');
});

test('a place with no piece of its own is served by a fallback piece', () => {
  const { starts } = walk(make(4), 200, () => 'sea');
  assert.ok(starts.length >= 1);
  assert.ok(starts.every((s) => PIECES.find((p) => p.id === s.id).contexts.includes('any')));
});

test('an always mode plays only its mood, with a short breath between pieces', () => {
  for (const mood of ['calm', 'lively']) {
    const { starts, ends } = walk(make(9, mood), 2 * 3600, (t) => (t % 600 < 300 ? 'day' : 'night'));
    assert.ok(starts.length > 20, `${starts.length} ${mood} pieces`);
    assert.ok(starts.every((s) => s.mood === mood), `only ${mood} pieces`);
    // A start that fades over a playing piece follows no silence; every other start follows the last end.
    for (const s of starts.filter((x) => !x.crossfade && x.t > 0)) {
      const before = ends.filter((e) => e <= s.t).pop();
      if (before !== undefined) assert.ok(s.t - before <= GAP[mood][1] + DT, `breath of ${s.t - before} s`);
    }
  }
});

test('an always mode still prefers the piece made for the place', () => {
  const { starts } = walk(make(11, 'calm'), 600, () => 'night');
  assert.ok(starts.length > 0 && starts.every((s) => s.id === 'night'));
});

test('changing mode while a piece plays fades to one that fits; while silent it cuts the wait short', () => {
  const d = make(6, 'lively');
  let switched = null;
  walk(d, 200, () => 'day', (t) => {
    if (t === 60) d.setMode('calm');
    if (t > 60 && !switched && d.playing?.mood === 'calm') switched = t;
  });
  assert.ok(switched !== null && switched <= 61, `switched at ${switched}`);
  const quiet = make(8);
  quiet.update(DT, 'day');
  quiet.setMode('lively');
  let started = null;
  for (let t = 0; t < 10 && !started; t += DT) started = quiet.update(DT, 'day');
  assert.ok(started, 'a piece within 10 seconds');
});

test('a piece that cannot be played is never chosen again', () => {
  const d = make(1);
  const pool = PIECES.filter((p) => p.contexts.includes('day'));
  for (const p of pool) d.failed(p.id);
  const { starts } = walk(d, 3600, () => 'day');
  assert.equal(starts.length, 0, 'nothing else fits the day, so silence');
});

test('machines lower the music by at most 30%', () => {
  assert.equal(duckFor(0), 1);
  assert.ok(Math.abs(duckFor(1) - 0.7) < 1e-9);
  assert.ok(Math.abs(duckFor(5) - 0.7) < 1e-9, 'clamped');
});
