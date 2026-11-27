// The admin console's commands, as pure functions of a line of text and a `ConsoleHost` (no DOM, no wasm; tested in
// web/test/commands.test.mjs). A line is a command when it starts with `/` or `!` (`lineKind`); anything else is chat
// (ui/console.ts sends it). `runCommand(line, host)` answers with the lines to print. The host (main.ts) wires each
// capability to the game or renderer; a command that changes the shared world (`time`, `tp`) refuses in a co-op world,
// where it would put this player out of step with everyone else. To add a command: a function here that takes the
// arguments and the host, a row in `COMMANDS`, and a line in the test.

import { HELD_SNOW_TEMP, weatherOf, type Weather, type WeatherKind } from '../render/sky/weather.ts';

export interface ConsoleHost {
  /** Draw this weather instead of the model's; `undefined` goes back to the model. */
  setWeather(w: Weather | undefined): void;
  /** Whether the sky settings have the weather turned off (a forced weather would not show). */
  weatherOff(): boolean;
  timeOfDay(): number;
  dayNumber(): number;
  daySeconds(): number;
  /** Runs the world forward by `seconds` at once. */
  skip(seconds: number): void;
  /** 0 spring, 1 summer, 2 autumn, 3 winter; the day of the season from 1; days in a season. */
  season(): number;
  dayOfSeason(): number;
  daysPerSeason(): number;
  /** The engine's weather here (`game.weather_here`): cover, gloom, precip, snow (1/0), °C, sky kind (0–3). */
  weatherHere(): number[];
  /** `days` days from today, 4 numbers each (`game.weather_forecast`): worst sky 0–3, snow, low and high in tenths °C. */
  forecast(days: number): number[];
  /** Seconds until the sky here turns to `want` (0 clear, 1 cloudy, 2 rain, 3 storm, 4 snow); -1 if not within a year. */
  weatherAhead(want: number): number;
  /** The world runs this many times as fast (1 normal); the engine clamps it. */
  setTimeSpeed(speed: number): void;
  timeSpeed(): number;
  position(): [number, number, number];
  teleport(x: number, y: number, z: number): void;
  find(query: string): { id: number; name: string }[];
  give(id: number, count: number): void;
  /** Builds a live playable concept demo near the player (`grader`). */
  setupDemo?(name: string): string;
  /** Whether this world is shared with other players. */
  shared(): boolean;
}

interface Command {
  usage: string;
  about: string;
  run(args: string[], host: ConsoleHost): string[];
}

/** Default stack for `give` when the line names no count. */
const GIVE_DEFAULT = 64;

const KINDS: Record<string, WeatherKind | 'auto'> = {
  clear: 'clear', sun: 'clear', sunny: 'clear', cloudy: 'cloudy', clouds: 'cloudy', gloomy: 'gloomy', overcast: 'gloomy',
  rain: 'rain', storm: 'rain', snow: 'snow', auto: 'auto', natural: 'auto', off: 'auto',
};

/** Named times as hours on the clock (the day starts at midnight, noon is 12). */
const TIMES: Record<string, number> = {
  midnight: 0, night: 22, dawn: 5.5, sunrise: 6, morning: 8, day: 9, noon: 12, afternoon: 15, sunset: 18, dusk: 19,
};

const pad = (n: number) => String(n).padStart(2, '0');

export function clockText(t: number): string {
  const minutes = Math.floor((((t % 1) + 1) % 1) * 24 * 60);
  return `${pad(Math.floor(minutes / 60))}:${pad(minutes % 60)}`;
}

/** A time given as a name or as `H:MM` / `HH:MM`, as a fraction of the day; `undefined` when it is neither. */
export function parseTime(word: string): number | undefined {
  const named = TIMES[word];
  if (named !== undefined) return named / 24;
  const m = /^(\d{1,2}):(\d{2})$/.exec(word);
  if (!m || Number(m[1]) > 23 || Number(m[2]) > 59) return undefined;
  return (Number(m[1]) * 60 + Number(m[2])) / (24 * 60);
}

function weather(args: string[], host: ConsoleHost): string[] {
  if (args.length === 0) return [`Now: ${skyText(host.weatherHere())}.`, stationText(host.weatherHere())];
  if (args[0] === 'wait') return weatherWait(args.slice(1), host);
  const kind = KINDS[args[0]];
  if (!kind) return [`Weather is one of: clear, cloudy, gloomy, rain, snow, auto (or weather wait <kind>).`];
  if (kind === 'auto') {
    host.setWeather(undefined);
    return ['The weather follows the world again.'];
  }
  const amount = args[1] === undefined ? 1 : Number(args[1]);
  if (!Number.isFinite(amount)) return ['The strength is a number from 0 to 1.'];
  host.setWeather(weatherOf(kind, amount));
  const out = [`The weather is held at ${kind}${kind === 'rain' || kind === 'snow' ? ` (${Math.round(Math.min(1, Math.max(0, amount)) * 100)}%)` : ''}.`];
  const temp = host.weatherHere()[4];
  if (kind === 'rain' && temp < 0) out.push(`It is ${degrees(temp)} here, so it falls as snow.`);
  if (kind === 'snow' && temp > HELD_SNOW_TEMP) out.push(`It is held at ${degrees(HELD_SNOW_TEMP)} too, so the snow lies.`);
  if (host.weatherOff()) out.push('The weather is turned off in the sky settings, so you will not see it.');
  return out;
}

function time(args: string[], host: ConsoleHost): string[] {
  const now = `Day ${host.dayNumber()}, ${clockText(host.timeOfDay())}`;
  if (args.length === 0) return [now];
  const target = parseTime(args[0]);
  if (target === undefined) return ['Give a time like noon, midnight, dusk or 18:30.'];
  if (host.shared()) return ['The time is shared with the other players: it can be changed in a world of your own.'];
  const ahead = (target - host.timeOfDay() + 1) % 1; // always forward: the world only runs ahead
  host.skip(ahead * host.daySeconds());
  return [`It is ${clockText(target)} (moved ${Math.round(ahead * 24 * 60)} minutes ahead).`];
}

/** The sky kinds the engine reports (api/weather.rs), and what `weather wait` can look for. */
const SKIES = ['clear', 'cloudy', 'rain', 'storm'];
const WAITS: Record<string, number> = { clear: 0, sun: 0, cloudy: 1, clouds: 1, rain: 2, storm: 3, snow: 4 };
const SEASONS = ['spring', 'summer', 'autumn', 'winter'];
/** Where `season` lands in the day: the morning, as a new world starts. */
const SEASON_MORNING = 7 / 24;
const UNITS: Record<string, number> = { d: 1, h: 1 / 24, m: 1 / 1440 };

const degrees = (c: number) => `${Math.round(c * 10) / 10} °C`;

/** The 16 points of the compass, clockwise from north. */
const COMPASS = ['N', 'NNE', 'NE', 'ENE', 'E', 'ESE', 'SE', 'SSE', 'S', 'SSW', 'SW', 'WSW', 'W', 'WNW', 'NW', 'NNW'];

/** What a weather station reads here (`weather_here` from index 6): pressure, humidity, dew point and wind. */
export function stationText(w: number[]): string {
  const [humidity, dew, pressure, wind, from] = w.slice(6);
  const point = COMPASS[Math.round(from / 22.5) % 16];
  return `Pressure ${Math.round(pressure)} hPa, humidity ${Math.round(humidity)}%, dew point ${degrees(dew)}, wind ${Math.round(wind * 10) / 10} m/s from the ${point}.`;
}

/** One line for the engine's weather here: the sky, how hard it falls, the temperature. */
export function skyText(w: number[]): string {
  const [, , precip, snow, temp, kind] = w;
  const sky = precip >= 0.15 && snow ? 'snow' : SKIES[kind] ?? 'clear';
  const falling = precip >= 0.15 ? ` (${Math.round(precip * 100)}%)` : '';
  return `${sky}${falling}, ${degrees(temp)}`;
}

function weatherWait(args: string[], host: ConsoleHost): string[] {
  const want = WAITS[args[0] ?? ''];
  if (want === undefined) return ['Use weather wait <clear|cloudy|rain|storm|snow>.'];
  if (host.shared()) return ['The weather is shared with the other players: it can be waited for in a world of your own.'];
  const seconds = host.weatherAhead(want);
  if (seconds < 0) return [`No ${args[0]} here in the next 60 days. Try another place or season (snow needs cold: high up, far north or winter).`];
  host.skip(seconds);
  const hours = Math.round((seconds / host.daySeconds()) * 24);
  return [`Ran ${hours} hours ahead. Now: ${skyText(host.weatherHere())}.`];
}

function forecast(_args: string[], host: ConsoleHost): string[] {
  const days = 7;
  const f = host.forecast(days);
  const out = [`Now: ${skyText(host.weatherHere())}.`];
  for (let d = 0; d * 4 < f.length; d++) {
    const [sky, snow, low, high] = f.slice(d * 4, d * 4 + 4);
    const name = snow && sky >= 2 ? 'snow' : SKIES[sky] ?? '?';
    out.push(`Day ${host.dayNumber() + d}${d === 0 ? ' (today)' : ''}: ${name}, ${degrees(low / 10)} to ${degrees(high / 10)}`);
  }
  return out;
}

function season(args: string[], host: ConsoleHost): string[] {
  const now = `${cap(SEASONS[host.season()])}, day ${host.dayOfSeason()} of ${host.daysPerSeason()} (day ${host.dayNumber()}).`;
  if (args.length === 0) return [now];
  const target = SEASONS.indexOf(args[0] === 'fall' ? 'autumn' : args[0]);
  if (target < 0) return ['Use season <spring|summer|autumn|winter>.'];
  if (host.shared()) return ['The season is shared with the other players: it can be changed in a world of your own.'];
  if (host.season() === target) return [`It is ${SEASONS[target]} already: ${now} skip <days> moves on within it.`];
  const per = host.daysPerSeason();
  const intoYear = host.season() * per + host.dayOfSeason() - 1;
  const ahead = (target * per - intoYear + 4 * per) % (4 * per); // whole days to that season's first day
  const days = ahead + SEASON_MORNING - host.timeOfDay();
  host.skip(days * host.daySeconds());
  return [`Ran ${Math.round(days * 10) / 10} days ahead. ${cap(SEASONS[host.season()])}, day ${host.dayOfSeason()}.`];
}

function skip(args: string[], host: ConsoleHost): string[] {
  const m = /^(\d+(?:\.\d+)?)([dhm]?)$/.exec(args[0] ?? '');
  if (!m) return ['Use skip <n>[d|h|m]: skip 3d (days), skip 6h (hours), skip 30m (minutes). A bare number is days.'];
  const days = Number(m[1]) * UNITS[m[2] || 'd'];
  if (days > 4 * 36) return ['At most 144 days at once (four years).'];
  if (host.shared()) return ['The time is shared with the other players: it can be changed in a world of your own.'];
  host.skip(days * host.daySeconds());
  return [`Ran ${args[0]} ahead. Day ${host.dayNumber()}, ${clockText(host.timeOfDay())}, ${SEASONS[host.season()]}.`];
}

function speed(args: string[], host: ConsoleHost): string[] {
  if (args.length === 0) return [`The world runs at ${host.timeSpeed()}× (speed 1 is normal).`];
  const n = Number(args[0]);
  if (!Number.isFinite(n) || n < 1) return ['Use speed <1 to 120>: speed 60 makes a day last 20 seconds. speed 1 is normal.'];
  if (host.shared()) return ['The time is shared with the other players: it can be sped up in a world of your own.'];
  host.setTimeSpeed(Math.round(n));
  const s = host.timeSpeed();
  const day = host.daySeconds() / s;
  return [`The world runs at ${s}×: a day lasts ${day >= 60 ? `${Math.round(day / 6) / 10} minutes` : `${Math.round(day)} seconds`}.`];
}

const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

function teleport(args: string[], host: ConsoleHost): string[] {
  const here = host.position();
  const coords = args.map((a, i) => {
    if (a === '~') return here[i];
    if (a.startsWith('~')) return here[i] + Number(a.slice(1));
    return a === '' ? NaN : Number(a);
  });
  if (args.length !== 3 || coords.some((c) => !Number.isFinite(c))) {
    return ['Use tp x y z. A ~ means where you are: tp ~ 100 ~ goes up to height 100, tp ~10 ~ ~ goes 10 east.'];
  }
  if (host.shared()) return ['Moving yourself by command would put you out of step with the other players.'];
  host.teleport(coords[0], coords[1], coords[2]);
  return [`Moved to ${coords.map(Math.round).join(' ')}.`];
}

function position(_args: string[], host: ConsoleHost): string[] {
  return [`You are at ${host.position().map(Math.round).join(' ')}.`];
}

function give(args: string[], host: ConsoleHost): string[] {
  const last = args[args.length - 1];
  const counted = args.length > 1 && /^\d+$/.test(last);
  const query = (counted ? args.slice(0, -1) : args).join(' ');
  if (!query) return [`Use give <item> [count], for example: give iron ingot 20.`];
  const count = counted ? Math.min(Number(last), 9999) : GIVE_DEFAULT;
  const found = host.find(query);
  if (found.length === 0) return [`No item is called "${query}".`];
  if (found.length > 1) return [`Which one? ${found.map((f) => f.name).join(', ')}.`];
  host.give(found[0].id, count);
  return [`Gave ${count} ${found[0].name}.`];
}

function demo(args: string[], host: ConsoleHost): string[] {
  const which = args[0] ?? 'grader';
  if (which !== 'grader') return ['Available demos: demo grader'];
  if (host.shared()) return ['Demos can only be spawned in a solo world of your own.'];
  if (!host.setupDemo) return ['Demos are not available in this build.'];
  return [host.setupDemo(which)];
}

function help(_args: string[], _host: ConsoleHost): string[] {
  return Object.values(COMMANDS).map((c) => `${c.usage}: ${c.about}`);
}

const COMMANDS: Record<string, Command> = {
  help: { usage: 'help', about: 'this list', run: help },
  demo: { usage: 'demo [grader]', about: 'spawn a live concept demo (Telescoping Bucket-Wheel Site Grader)', run: demo },
  weather: {
    usage: 'weather [clear|cloudy|gloomy|rain|snow|auto] [strength]',
    about: 'show the weather, or hold how the sky looks (auto: back to the world\'s)',
    run: weather,
  },
  'weather wait': {
    usage: 'weather wait <clear|cloudy|rain|storm|snow>',
    about: 'run the world ahead until that weather really comes here',
    run: weatherWait,
  },
  forecast: { usage: 'forecast', about: 'the weather now and for the next 6 days here', run: forecast },
  time: {
    usage: 'time [noon|midnight|dawn|dusk|night|18:30]',
    about: 'show the time, or run the world ahead to it',
    run: time,
  },
  season: {
    usage: 'season [spring|summer|autumn|winter]',
    about: 'show the season, or run ahead to the first morning of that one',
    run: season,
  },
  skip: { usage: 'skip <n>[d|h|m]', about: 'run the world ahead that many days, hours or minutes', run: skip },
  speed: { usage: 'speed [1-120]', about: 'run the world faster (60: a day in 20 seconds); speed 1 is normal', run: speed },
  tp: { usage: 'tp <x> <y> <z>', about: 'move there (~ means where you are)', run: teleport },
  pos: { usage: 'pos', about: 'where you are', run: position },
  give: { usage: `give <item> [count]`, about: `put items in your pack (${GIVE_DEFAULT} if no count)`, run: give },
};

/** The characters that start a command; any other line is words to the other players. */
export const COMMAND_PREFIXES = ['/', '!'];

/** What a typed line is: a command (it starts with `/` or `!`), chat (anything else), or nothing (blank). */
export function lineKind(line: string): 'command' | 'chat' | 'empty' {
  const t = line.trim();
  return t === '' ? 'empty' : COMMAND_PREFIXES.includes(t[0]) ? 'command' : 'chat';
}

/** Runs one line; the answer is the lines to print (empty for an empty line). A leading `/` or `!` is allowed. */
export function runCommand(line: string, host: ConsoleHost): string[] {
  const words = line.trim().replace(/^[/!]/, '').toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return [];
  const command = COMMANDS[words[0]] ?? (words[0] === 'teleport' ? COMMANDS.tp : undefined);
  if (!command) return [`There is no command "${words[0]}". Type /help for the list.`];
  return command.run(words.slice(1), host);
}
