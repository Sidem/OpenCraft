// Tests for the admin console's commands (web/src/console/commands.ts) against a fake host.

import assert from 'node:assert/strict';
import test from 'node:test';
import { clockText, parseTime, runCommand } from '../src/console/commands.ts';

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
    shared: () => false,
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

test('help lists every command, unknown words say so, and an empty line is quiet', () => {
  const h = fakeHost();
  const help = runCommand('help', h).join('\n');
  for (const name of ['weather', 'time', 'tp', 'pos', 'give']) assert.ok(help.includes(name), name);
  assert.match(runCommand('fly', h)[0], /no command "fly"/);
  assert.deepEqual(runCommand('   ', h), []);
  assert.match(runCommand('pos', h)[0], /10 64 -20/);
});
