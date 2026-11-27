// Tests for the admin console's commands (web/src/console/commands.ts) against a fake host.

import assert from 'node:assert/strict';
import test from 'node:test';
import { clockText, lineKind, parseTime, runCommand } from '../src/console/commands.ts';

function fakeHost(over = {}) {
  const log = [];
  const host = {
    log,
    clock: 0.25,
    setWeather: (w) => log.push(['weather', w]),
    weatherOff: () => false,
    timeOfDay() { return this.clock; },
    dayNumber: () => 3,
    daySeconds: () => 1200,
    skip: (s) => log.push(['skip', s]),
    position: () => [10, 64, -20],
    teleport: (x, y, z) => log.push(['tp', x, y, z]),
    find: (q) => ({ iron: [{ id: 1, name: 'Iron Ingot' }, { id: 2, name: 'Iron Plate' }], 'iron ingot': [{ id: 1, name: 'Iron Ingot' }] }[q] ?? []),
    give: (id, n) => log.push(['give', id, n]),
    setupDemo: (name) => { log.push(['demo', name]); return `Site Grader demo ready (${name})`; },
    shared: () => false,
    seasonNow: 0,
    dayNow: 3,
    season() { return this.seasonNow; },
    dayOfSeason() { return this.dayNow; },
    daysPerSeason: () => 9,
    weatherHere: () => [0.9, 0.8, 0.6, 0, 4.5, 2, 94, 3.7, 997.4, 5.24, 250],
    forecast: (days) => Array.from({ length: days }, (_, d) => [d % 4, d === 3 ? 1 : 0, -20 + d, 80 + d]).flat(),
    weatherAhead: (want) => (want === 4 ? -1 : 600),
    speedNow: 1,
    setTimeSpeed(s) { this.speedNow = Math.min(120, Math.max(1, s)); log.push(['speed', s]); },
    timeSpeed() { return this.speedNow; },
    ...over,
  };
  return host;
}

test('times parse from names and clock text, and print back', () => {
  assert.equal(parseTime('noon'), 0.5);
  assert.equal(parseTime('18:30'), 18.5 / 24);
  assert.equal(parseTime('7:05'), (7 * 60 + 5) / 1440);
  for (const bad of ['25:00', '12:60', 'later', '12']) assert.equal(parseTime(bad), undefined, bad);
  assert.equal(clockText(0.5), '12:00');
  assert.equal(clockText(18.5 / 24), '18:30');
});

test('weather forces a look, or hands the sky back', () => {
  const h = fakeHost();
  assert.match(runCommand('weather rain', h)[0], /held at rain/);
  assert.equal(h.log[0][1].snow, false);
  assert.ok(h.log[0][1].precip > 0.9);
  runCommand('/Weather snow 0.5', h);
  assert.equal(h.log[1][1].snow, true);
  assert.ok(h.log[1][1].precip > 0.4 && h.log[1][1].precip < 0.6);
  runCommand('weather auto', h);
  assert.equal(h.log[2][1], undefined);
  runCommand('weather clear', h);
  assert.equal(h.log[3][1].precip, 0);
  assert.match(runCommand('weather fog', h)[0], /one of/);
  assert.match(runCommand('weather rain lots', h)[0], /number/);
  assert.match(runCommand('weather rain', fakeHost({ weatherOff: () => true })).join(' '), /turned off/);
  const cold = fakeHost({ weatherHere: () => [1, 1, 0.5, 1, -3, 2] });
  assert.match(runCommand('weather rain', cold).join(' '), /-3 °C here, so it falls as snow/);
  assert.match(runCommand('weather snow', h).join(' '), /held at -4 °C too/);
});

test('weather alone reads the sky and the station: pressure, humidity, dew point, wind', () => {
  const [sky, station] = runCommand('weather', fakeHost());
  assert.match(sky, /rain \(60%\), 4.5/);
  assert.equal(station, 'Pressure 997 hPa, humidity 94%, dew point 3.7 °C, wind 5.2 m/s from the WSW.');
});

test('time runs the world forward to the asked time and never back', () => {
  const h = fakeHost(); // 06:00
  runCommand('time noon', h);
  assert.deepEqual(h.log.pop(), ['skip', 300]); // six hours of a 24-hour, 20-minute day
  const late = fakeHost({ clock: 0.75 }); // 18:00
  runCommand('time 06:00', late);
  assert.deepEqual(late.log.pop(), ['skip', 600]); // twelve hours through the night, not backwards
  assert.deepEqual(runCommand('time', h), ['Day 3, 06:00']);
  assert.match(runCommand('time lunch', h)[0], /Give a time/);
  const shared = fakeHost({ shared: () => true });
  runCommand('time noon', shared);
  assert.equal(shared.log.length, 0, 'a shared world is left alone');
});

test('tp takes numbers and ~ for here', () => {
  const h = fakeHost();
  runCommand('tp 1 2 3', h);
  runCommand('tp ~ 100 ~', h);
  runCommand('teleport ~5 ~ ~-4', h);
  assert.deepEqual(h.log, [['tp', 1, 2, 3], ['tp', 10, 100, -20], ['tp', 15, 64, -24]]);
  assert.match(runCommand('tp 1 2', h)[0], /Use tp/);
  assert.match(runCommand('tp a b c', h)[0], /Use tp/);
  const shared = fakeHost({ shared: () => true });
  runCommand('tp 1 2 3', shared);
  assert.equal(shared.log.length, 0);
});

test('give finds an item by name and takes an optional count', () => {
  const h = fakeHost();
  assert.match(runCommand('give iron ingot 20', h)[0], /Gave 20 Iron Ingot/);
  runCommand('give iron ingot', h);
  assert.deepEqual(h.log, [['give', 1, 20], ['give', 1, 64]]);
  assert.match(runCommand('give iron', h)[0], /Which one\? Iron Ingot, Iron Plate/);
  assert.match(runCommand('give gold', h)[0], /No item/);
  assert.match(runCommand('give', h)[0], /Use give/);
  assert.equal(h.log.length, 2, 'unclear or unknown names give nothing');
});

test('weather and forecast read the engine, and weather wait runs ahead to the real thing', () => {
  const h = fakeHost();
  assert.equal(runCommand('weather', h)[0], 'Now: rain (60%), 4.5 °C.');
  const f = runCommand('forecast', h);
  assert.equal(f.length, 8);
  assert.equal(f[1], 'Day 3 (today): clear, -2 °C to 8 °C');
  assert.match(f[4], /Day 6: snow/);
  assert.match(runCommand('weather wait rain', h)[0], /Ran 12 hours ahead\. Now: rain/);
  assert.deepEqual(h.log.pop(), ['skip', 600]);
  assert.match(runCommand('weather wait snow', h)[0], /No snow here in the next 60 days/);
  assert.match(runCommand('weather wait fog', h)[0], /Use weather wait/);
  const shared = fakeHost({ shared: () => true });
  runCommand('weather wait rain', shared);
  assert.equal(shared.log.length, 0);
});

test('season runs ahead to the first morning of a season, skip by days, hours or minutes', () => {
  const h = fakeHost(); // spring, day 3 of 9, 06:00
  assert.deepEqual(runCommand('season', h), ['Spring, day 3 of 9 (day 3).']);
  runCommand('season summer', h);
  // Day 3 to summer's first day (day 10) is 7 days, and 06:00 to 07:00 an hour.
  assert.ok(Math.abs(h.log.pop()[1] - (7 + 1 / 24) * 1200) < 1e-6);
  runCommand('season fall', h);
  assert.ok(Math.abs(h.log.pop()[1] - (16 + 1 / 24) * 1200) < 1e-6);
  assert.match(runCommand('season spring', h)[0], /spring already/);
  assert.match(runCommand('season monsoon', h)[0], /Use season/);
  runCommand('skip 2d', h);
  runCommand('skip 6h', h);
  runCommand('skip 30m', h);
  runCommand('skip 1', h);
  assert.deepEqual(h.log.splice(0).map((e) => Math.round(e[1] * 1000) / 1000), [2400, 300, 25, 1200]);
  assert.match(runCommand('skip soon', h)[0], /Use skip/);
  assert.match(runCommand('skip 500d', h)[0], /At most/);
  const shared = fakeHost({ shared: () => true });
  runCommand('season winter', shared);
  runCommand('skip 1d', shared);
  assert.equal(shared.log.length, 0);
});

test('speed sets how fast the world runs and says how long a day lasts', () => {
  const h = fakeHost();
  assert.match(runCommand('speed', h)[0], /1×/);
  assert.match(runCommand('speed 60', h)[0], /60×: a day lasts 20 seconds/);
  assert.match(runCommand('speed 2', h)[0], /2×: a day lasts 10 minutes/);
  assert.match(runCommand('speed 999', h)[0], /120×/);
  assert.match(runCommand('speed 0', h)[0], /Use speed/);
  const shared = fakeHost({ shared: () => true });
  runCommand('speed 60', shared);
  assert.equal(shared.log.length, 0);
});

test('help lists every command, unknown words say so, and an empty line is quiet', () => {
  const h = fakeHost();
  const help = runCommand('help', h).join('\n');
  for (const name of ['weather', 'forecast', 'time', 'season', 'skip', 'speed', 'tp', 'pos', 'give']) {
    assert.ok(help.includes(name), name);
  }
  assert.match(runCommand('fly', h)[0], /no command "fly"/);
  assert.deepEqual(runCommand('   ', h), []);
  assert.match(runCommand('pos', h)[0], /10 64 -20/);
});

test('a line starting with / or ! is a command, any other is chat, and both prefixes run the same command', () => {
  assert.equal(lineKind('/help'), 'command');
  assert.equal(lineKind('  !tp 0 0 0'), 'command');
  assert.equal(lineKind('hello there'), 'chat');
  assert.equal(lineKind('help'), 'chat');
  assert.equal(lineKind('   '), 'empty');
  const h = fakeHost();
  assert.deepEqual(runCommand('!pos', h), runCommand('/pos', h));
  assert.match(runCommand('/fly', h)[0], /Type \/help/);
  assert.match(runCommand('/demo grader', h)[0], /Site Grader demo ready/);
  assert.deepEqual(h.log.pop(), ['demo', 'grader']);
});