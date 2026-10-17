// Tests for the real night sky's maths (web/src/render/sky/astro.ts) and the baked star file, run with `node --test web/test`.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { apply, equatorial, MOON_DAYS, readStars, skyFrame, skyPicture, YEAR_DAYS } from '../src/render/sky/astro.ts';

const deg = (d) => (d * Math.PI) / 180;
const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
/** What render/sky.ts `skyAt` does for the sun. */
function skyAtSun(t) {
  const a = (t - 0.25) * 2 * Math.PI;
  const v = [Math.cos(a), Math.sin(a), 0.35];
  const l = Math.hypot(...v);
  return v.map((x) => x / l);
}

test('the turn is a rotation (no mirror image of the sky)', () => {
  for (const days of [0, 0.3, 17.77, 95.5]) {
    const m = skyFrame(days).rotation;
    for (let i = 0; i < 3; i++) for (let j = 0; j < 3; j++) {
      const row = m.slice(i * 3, i * 3 + 3), other = m.slice(j * 3, j * 3 + 3);
      assert.ok(Math.abs(dot(row, other) - (i === j ? 1 : 0)) < 1e-12);
    }
    const det = m[0] * (m[4] * m[8] - m[5] * m[7]) - m[1] * (m[3] * m[8] - m[5] * m[6]) + m[2] * (m[3] * m[7] - m[4] * m[6]);
    assert.ok(Math.abs(det - 1) < 1e-12, `det ${det}`);
  }
});

test('the sun of the sky frame is the sun of skyAt on every day of the year', () => {
  for (let day = 0; day < YEAR_DAYS; day += 7) {
    for (const t of [0, 0.1, 0.25, 0.5, 0.71, 0.99]) {
      const got = skyFrame(day + t).sun, want = skyAtSun(t);
      for (let i = 0; i < 3; i++) assert.ok(Math.abs(got[i] - want[i]) < 1e-9, `day ${day} t ${t} axis ${i}`);
    }
  }
});

test('stars rise in the east, cross the south-north line overhead and set in the west', () => {
  // A star on the celestial equator at the sun's right ascension behaves like the sun: it is on the meridian at noon.
  // (The sun's right ascension creeps on through the year, so the star is placed at it afresh for each time.)
  const day = 10;
  const at = (t) => apply(skyFrame(day + t).rotation, equatorial(Math.PI + (2 * Math.PI * (day + t)) / YEAR_DAYS, 0));
  assert.ok(at(0.5)[1] > 0.999, 'overhead at noon');
  assert.ok(at(0.25)[0] > 0.999 && Math.abs(at(0.25)[1]) < 1e-9, 'rising due east at 6:00');
  assert.ok(at(0.75)[0] < -0.999, 'setting due west at 18:00');
});

test('the sky is not mirrored: Betelgeuse is north of Rigel and east of it, as in Orion', () => {
  const betelgeuse = equatorial(deg(88.79), deg(7.41)), rigel = equatorial(deg(78.63), deg(-8.2));
  // When Orion crosses the meridian (local sidereal time at their right ascension of about 83 degrees).
  for (let days = 0; days < YEAR_DAYS; days += 1.37) {
    const m = skyFrame(days).rotation, b = apply(m, betelgeuse), r = apply(m, rigel);
    if (b[1] < 0.9) continue;
    assert.ok(b[2] < r[2], 'Betelgeuse is further north (-z) than Rigel');
    assert.ok(b[0] > r[0], 'Betelgeuse is east (+x) of Rigel');
  }
});

test('moon phases follow the cycle and the full moon rises at sunset', () => {
  assert.ok(Math.abs(skyFrame(0).illumination - 0.5) < 1e-12, 'first quarter at the start');
  assert.ok(Math.abs(skyFrame(2).illumination - 1) < 1e-12, 'full two days in');
  assert.ok(skyFrame(6).illumination < 1e-12, 'new six days in');
  assert.ok(Math.abs(skyFrame(MOON_DAYS + 3.3).illumination - skyFrame(3.3).illumination) < 1e-12, 'repeats every MOON_DAYS');
  const full = skyFrame(2);
  assert.ok(dot(full.moon, full.sun) < -0.999999, 'opposite the sun');
  assert.ok(full.moon[1] > 0.9, 'so it stands high at midnight');
  assert.ok(Math.abs(skyFrame(1.75).moon[1]) < 0.25, 'and is near the horizon at the sunset before (rising)');
  assert.ok(skyFrame(1.8).moon[1] > skyFrame(1.75).moon[1], 'climbing');
  const dark = skyFrame(6);
  assert.ok(dot(dark.moon, dark.sun) > 0.999999, 'the new moon is by the sun');
});

test('the Milky Way picture puts the galactic centre in Sagittarius, right of the middle', () => {
  const [u, v] = skyPicture(equatorial(deg(266.4), deg(-28.9)));
  assert.ok(Math.abs(u - 0.76) < 0.001 && Math.abs(v - 0.661) < 0.001, `${u} ${v}`);
  const [u0, v0] = skyPicture(equatorial(0, 0));
  assert.ok(Math.abs(u0 - 0.5) < 1e-12 && Math.abs(v0 - 0.5) < 1e-12);
});

test('stars.bin holds the catalogue: Sirius first, 89 sensible lines', () => {
  const file = readFileSync(new URL('../src/render/sky/stars.bin', import.meta.url));
  const c = readStars(file.buffer.slice(file.byteOffset, file.byteOffset + file.byteLength));
  assert.equal(c.count, 8920);
  assert.equal(c.lines.length, 89 * 2);
  const sirius = equatorial(deg(101.287), deg(-16.716));
  assert.ok(dot([c.data[0], c.data[1], c.data[2]], sirius) > Math.cos(deg(0.05)), 'Sirius is the first star');
  assert.ok(Math.abs(c.data[3] - -1.44) < 0.05, `its magnitude ${c.data[3]}`);
  assert.ok(c.data[(c.count - 1) * 4 + 3] <= 6.55, 'nothing fainter than the limit');
  for (let i = 0; i < c.lines.length; i += 2) {
    const a = c.lines[i] * 4, b = c.lines[i + 1] * 4;
    const angle = (Math.acos(Math.min(1, dot(c.data.subarray(a, a + 3), c.data.subarray(b, b + 3)))) * 180) / Math.PI;
    assert.ok(angle > 0.8 && angle < 40, `line ${i / 2} spans ${angle} degrees`);
  }
});
