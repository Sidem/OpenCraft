// The admin console's commands, as pure functions of a line of text and a `ConsoleHost` (no DOM, no wasm; tested in
// web/test/commands.test.mjs). `runCommand(line, host)` answers with the lines to print. The host (main.ts) wires each
// capability to the game or renderer; a command that changes the shared world (`time`, `tp`) refuses in a co-op world,
// where it would put this player out of step with everyone else. To add a command: a function here that takes the
// arguments and the host, a row in `COMMANDS`, and a line in the test.

import { weatherOf, type Weather, type WeatherKind } from '../render/sky/weather.ts';

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
  position(): [number, number, number];
  teleport(x: number, y: number, z: number): void;
  find(query: string): { id: number; name: string }[];
  give(id: number, count: number): void;
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
  const kind = KINDS[args[0] ?? ''];
  if (!kind) return [`Weather is one of: clear, cloudy, gloomy, rain, snow, auto.`];
  if (kind === 'auto') {
    host.setWeather(undefined);
    return ['The weather follows the world again.'];
  }
  const amount = args[1] === undefined ? 1 : Number(args[1]);
  if (!Number.isFinite(amount)) return ['The strength is a number from 0 to 1.'];
  host.setWeather(weatherOf(kind, amount));
  const out = [`The weather is held at ${kind}${kind === 'rain' || kind === 'snow' ? ` (${Math.round(Math.min(1, Math.max(0, amount)) * 100)}%)` : ''}.`];
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

function help(_args: string[], _host: ConsoleHost): string[] {
  return Object.values(COMMANDS).map((c) => `${c.usage}: ${c.about}`);
}

const COMMANDS: Record<string, Command> = {
  help: { usage: 'help', about: 'this list', run: help },
  weather: {
    usage: 'weather <clear|cloudy|gloomy|rain|snow|auto> [strength]',
    about: 'hold the weather (auto: let the world decide)',
    run: weather,
  },
  time: {
    usage: 'time [noon|midnight|dawn|dusk|night|18:30]',
    about: 'show the time, or run the world ahead to it',
    run: time,
  },
  tp: { usage: 'tp <x> <y> <z>', about: 'move there (~ means where you are)', run: teleport },
  pos: { usage: 'pos', about: 'where you are', run: position },
  give: { usage: `give <item> [count]`, about: `put items in your pack (${GIVE_DEFAULT} if no count)`, run: give },
};

/** Runs one line; the answer is the lines to print (empty for an empty line). A leading `/` is allowed. */
export function runCommand(line: string, host: ConsoleHost): string[] {
  const words = line.trim().replace(/^\//, '').toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return [];
  const command = COMMANDS[words[0]] ?? (words[0] === 'teleport' ? COMMANDS.tp : undefined);
  if (!command) return [`There is no command "${words[0]}". Type help for the list.`];
  return command.run(words.slice(1), host);
}
