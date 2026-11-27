// Tests for F-BEAT (the factory as an instrument): music theory consonance, interlocking groove &
// stall gaps, voice & distance caps, walking crossfade, music ducking, settings persistence, and
// acoustic DSP quality across all 32 synthesised instrument buffers. Run with `node --test web/test`.

import assert from 'node:assert/strict';
import test from 'node:test';
import {
  BARS_PER_CYCLE,
  beatAttenuation,
  beatDuckFor,
  beatPan,
  BeatTracker,
  DEFAULT_BEAT_SETTINGS,
  F,
  FAMILY_COUNT,
  FAMILY_STRIDE,
  loadBeatSettings,
  MAX_ACTIVE_FAMILIES,
  saveBeatSettings,
  slotActive,
  STEPS_PER_BAR,
  VOICES,
} from '../src/audio/beat-score.ts';
import { renderBeatNote } from '../src/audio/synth-beat.ts';

const RADIUS = 26;

/** Builds a flat `factory_beat()` buffer from per-family overrides. */
function makeReadings(entries) {
  const buf = new Float32Array(FAMILY_COUNT * FAMILY_STRIDE);
  for (let i = 0; i < FAMILY_COUNT; i++) buf[i * FAMILY_STRIDE + F.dist] = RADIUS + 1;
  for (const [fam, cfg] of Object.entries(entries)) {
    const o = Number(fam) * FAMILY_STRIDE;
    buf[o + F.strength] = cfg.strength ?? 0.9;
    buf[o + F.dist] = cfg.dist ?? 4;
    buf[o + F.rx] = cfg.rx ?? 0;
    buf[o + F.rz] = cfg.rz ?? 0;
    buf[o + F.health] = cfg.health ?? 1;
    buf[o + F.pace] = cfg.pace ?? 1;
    buf[o + F.phase] = cfg.phase ?? 0;
    buf[o + F.tier] = cfg.tier ?? 0;
  }
  return buf;
}

/** Converts Hz to MIDI pitch class (0=C, 2=D, 4=E, 6=F#, 9=A, 11=B). */
function pitchClass(hz) {
  const midi = Math.round(69 + 12 * Math.log2(hz / 440));
  return ((midi % 12) + 12) % 12;
}

test('every note across all 8 families belongs to D major pentatonic with zero semitone or tritone clashes', () => {
  const dMajPent = new Set([2, 4, 6, 9, 11]); // D, E, F#, A, B — diatonic in D major, E Dorian & F#m/A major
  assert.equal(VOICES.length, FAMILY_COUNT);
  for (const v of VOICES) {
    assert.equal(v.notes.length, 4, `${v.id} has 4 register notes`);
    assert.equal(v.bars.length, BARS_PER_CYCLE, `${v.id} covers 4 bars`);
    assert.equal(v.accents.length, v.steps.length, `${v.id} accents match steps`);
    assert.equal(v.dropOrder.length, v.steps.length, `${v.id} dropOrder matches steps`);
    for (const hz of v.notes) {
      const pc = pitchClass(hz);
      assert.ok(dMajPent.has(pc), `${v.id} note ${hz} Hz (pc ${pc}) must be in D major pentatonic`);
    }
    for (const bar of v.bars) {
      assert.equal(bar.length, v.steps.length);
      for (const idx of bar) assert.ok(idx >= 0 && idx < 4);
    }
  }
  // Verify pairwise intervals across the entire pitch-class set contain no semitones (1) or tritones (6).
  const pcs = [...dMajPent];
  for (const a of pcs) {
    for (const b of pcs) {
      const interval = Math.min((a - b + 12) % 12, (b - a + 12) % 12);
      assert.notEqual(interval, 1, 'no minor 2nd clashes');
      assert.notEqual(interval, 6, 'no tritone clashes');
    }
  }
});

test('a balanced line produces a steady interlocking groove and starved machines leave audible gaps', () => {
  const healthy = makeReadings({ 0: { dist: 5 }, 2: { dist: 4 }, 4: { dist: 3 }, 5: { dist: 6 } });
  const starved = makeReadings({
    0: { dist: 5 },
    2: { dist: 4, health: 0.5 },
    4: { dist: 3, health: 0.5 },
    5: { dist: 6 },
  });

  const trackerFull = new BeatTracker();
  const trackerGap = new BeatTracker();
  for (let i = 0; i < 20; i++) {
    trackerFull.update(healthy, RADIUS, 0.25);
    trackerGap.update(starved, RADIUS, 0.25);
  }

  let fullShaping = 0;
  let gapShaping = 0;
  let fullSmelting = 0;
  let gapSmelting = 0;
  for (let s = 0; s < STEPS_PER_BAR; s++) {
    const a = trackerFull.planStep(s, healthy, RADIUS, 0);
    const b = trackerGap.planStep(s, starved, RADIUS, 0);
    fullShaping += a.filter((n) => n.family === 4).length;
    gapShaping += b.filter((n) => n.family === 4).length;
    fullSmelting += a.filter((n) => n.family === 2).length;
    gapSmelting += b.filter((n) => n.family === 2).length;
  }

  assert.equal(fullShaping, VOICES[4].steps.length, 'healthy shaping plays every beat of its groove');
  assert.ok(gapShaping > 0 && gapShaping < fullShaping, `half-starved shaping leaves gaps (${gapShaping} < ${fullShaping})`);
  assert.equal(fullSmelting, VOICES[2].steps.length);
  assert.ok(gapSmelting > 0 && gapSmelting < fullSmelting, `half-starved smelting leaves gaps (${gapSmelting} < ${fullSmelting})`);

  // Completely stalled family drops out completely.
  const down = makeReadings({ 0: { dist: 5 } });
  for (let i = 0; i < 40; i++) trackerFull.update(down, RADIUS, 0.25);
  const notes = Array.from({ length: STEPS_PER_BAR }, (_, s) => trackerFull.planStep(s, down, RADIUS, 0)).flat();
  assert.ok(notes.every((n) => n.family === 0), 'stalled families fall silent');
});

test('hard cap: at most MAX_ACTIVE_FAMILIES sound even when all 8 families run right beside the player', () => {
  const crowded = makeReadings({
    0: { dist: 2, strength: 0.95 },
    1: { dist: 3, strength: 0.9 },
    2: { dist: 4, strength: 0.85 },
    3: { dist: 5, strength: 0.8 },
    4: { dist: 8, strength: 0.7 },
    5: { dist: 10, strength: 0.6 },
    6: { dist: 12, strength: 0.5 },
    7: { dist: 14, strength: 0.4 },
  });
  const tracker = new BeatTracker();
  for (let i = 0; i < 30; i++) tracker.update(crowded, RADIUS, 0.25);

  const heard = new Set();
  for (let s = 0; s < STEPS_PER_BAR * BARS_PER_CYCLE; s++) {
    for (const n of tracker.planStep(s, crowded, RADIUS, 0)) heard.add(n.family);
  }
  assert.equal(heard.size, MAX_ACTIVE_FAMILIES, `only the ${MAX_ACTIVE_FAMILIES} nearest families sound`);
  assert.deepEqual([...heard].sort((a, b) => a - b), [0, 1, 2, 3]);
});

test('hard distance cap silences machines at or beyond radius and panning stays gentle', () => {
  assert.equal(beatAttenuation(0, RADIUS), 1);
  assert.equal(beatAttenuation(RADIUS, RADIUS), 0);
  assert.equal(beatAttenuation(RADIUS + 10, RADIUS), 0);
  assert.ok(beatAttenuation(10, RADIUS) > beatAttenuation(20, RADIUS));

  // Panning never slams hard L/R (capped at ±0.55 to avoid headphone fatigue).
  const right = beatPan(10, 0, 10, 0);
  const left = beatPan(-10, 0, 10, 0);
  assert.ok(right > 0.5 && right <= 0.551, `right pan ${right}`);
  assert.ok(left < -0.5 && left >= -0.551, `left pan ${left}`);

  const far = makeReadings({ 4: { dist: RADIUS, strength: 0.9 } });
  const tracker = new BeatTracker();
  for (let i = 0; i < 10; i++) tracker.update(far, RADIUS, 0.25);
  assert.equal(tracker.activity, 0);
});

test('music ducks gently when the factory groove plays and returns to full when silent', () => {
  assert.equal(beatDuckFor(0), 1);
  assert.ok(beatDuckFor(1) >= 0.5 && beatDuckFor(1) <= 0.6, `full activity duck ${beatDuckFor(1)}`);
  assert.equal(beatDuckFor(3), beatDuckFor(1), 'clamped above 1');
  assert.equal(beatDuckFor(-1), 1, 'clamped below 0');

  // Brownout slows/thins syncopated slots cleanly.
  assert.equal(slotActive(VOICES[4], 4, 1, 1), true);
  assert.equal(slotActive(VOICES[4], 4, 1, 0.2), false, 'severe brownout drops ornamental beat');
});

test('all 32 synthesised instrument buffers are warm, click-free, DC-clean and non-clipping', () => {
  const sr = 44100;
  for (let fam = 0; fam < FAMILY_COUNT; fam++) {
    for (let ni = 0; ni < 4; ni++) {
      const freq = VOICES[fam].notes[ni];
      const buf = renderBeatNote(sr, fam, freq);
      assert.ok(buf.length > sr * 0.25, `family ${fam} note ${ni} has musical body`);

      let peak = 0;
      let sum = 0;
      let energy = 0;
      let diffEnergy = 0;
      for (let i = 0; i < buf.length; i++) {
        const v = buf[i];
        assert.ok(Number.isFinite(v), `family ${fam} note ${ni} sample ${i} finite`);
        peak = Math.max(peak, Math.abs(v));
        sum += v;
        energy += v * v;
        if (i > 0) {
          const d = v - buf[i - 1];
          diffEnergy += d * d;
        }
      }
      const mean = Math.abs(sum / buf.length);
      const rms = Math.sqrt(energy / buf.length);
      // Spectral brightness proxy: sqrt(diffEnergy / energy) / (2*pi) * sr ~ centroid Hz.
      const estHz = (Math.sqrt(diffEnergy / (energy || 1)) / (2 * Math.PI)) * sr;

      assert.ok(peak > 0.1 && peak <= 0.9, `family ${fam} note ${ni} peak ${peak.toFixed(3)} in headroom`);
      assert.ok(rms > 0.03, `family ${fam} note ${ni} audible RMS ${rms.toFixed(4)}`);
      assert.ok(mean < 0.005, `family ${fam} note ${ni} DC offset ${mean.toFixed(5)}`);
      assert.ok(Math.abs(buf[0]) < 1e-4, `family ${fam} note ${ni} starts at zero (no click)`);
      assert.ok(Math.abs(buf[buf.length - 1]) < 1e-4, `family ${fam} note ${ni} ends at zero (no click)`);
      assert.ok(estHz < 2200, `family ${fam} note ${ni} stays warm & non-fatiguing (est centroid ${estHz.toFixed(0)} Hz)`);
    }
  }
});

test('beat settings persist volume and mute cleanly and recover from corrupt storage', () => {
  const store = new Map();
  globalThis.localStorage = {
    getItem: (k) => store.get(k) ?? null,
    setItem: (k, v) => void store.set(k, String(v)),
  };
  assert.deepEqual(loadBeatSettings(), DEFAULT_BEAT_SETTINGS);
  saveBeatSettings({ volume: 65, muted: true });
  assert.deepEqual(loadBeatSettings(), { volume: 65, muted: true });
  store.set('opencraft.beat', '{bad json');
  assert.deepEqual(loadBeatSettings(), DEFAULT_BEAT_SETTINGS);
});
