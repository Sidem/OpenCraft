// The weather in the calendar (calendar.ts): now, as a big icon and the temperature, then the days ahead, each an icon of
// its worst sky with its high and low. Read from the engine's `weather_here` and `weather_forecast` (api/weather.rs) each
// time the calendar draws; temperatures are coloured from cold blue to hot red; words only in tooltips. To show more or
// fewer days: `DAYS`; to restyle a sky: its row in `SKIES`.

import './forecast.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { icon } from './seasons';

/** Days in the forecast, today first. */
const DAYS = 6;
/** The engine's kinds of sky in its order (weather/mod.rs `Kind`): a name for tooltips, an icon and its colour. */
const SKIES = [
  { name: 'Clear', icon: 'sun', colour: '#ffd75e' },
  { name: 'Cloudy', icon: 'cloud', colour: '#c9d2e0' },
  { name: 'Rain', icon: 'rain', colour: '#7fb2f0' },
  { name: 'Storm', icon: 'storm', colour: '#b49cf2' },
] as const;
const SNOW = { name: 'Snow', colour: '#ffffff' };

/** A temperature's colour: blue at -10 °C and below, through green, to red at 32 °C and above. */
export function tempColour(c: number): string {
  const t = Math.min(1, Math.max(0, (c + 10) / 42));
  return `hsl(${Math.round(210 - 210 * t)} 75% 68%)`;
}

/** Draws the weather row into `into`. */
export function drawForecast(game: Game, into: HTMLElement): void {
  const here = game.weather_here();
  const snowing = here[3] > 0.5, temp = here[4], kind = Math.round(here[5]);
  const now = h('div', 'fc-now');
  now.append(skyIcon(kind, snowing), degrees(temp));
  now.title = `Now: ${skyName(kind, snowing)}, ${Math.round(temp)} °C`;
  const per = game.forecast_fields(), days = game.weather_forecast(DAYS);
  const cells = [];
  for (let d = 0; d < DAYS; d++) {
    const [k, snow, low, high] = [0, 1, 2, 3].map((i) => days[d * per + i]);
    const cell = h('div', d === 0 ? 'fc-day today' : 'fc-day');
    const range = h('span', 'fc-range');
    range.append(degrees(high / 10), degrees(low / 10));
    cell.append(skyIcon(k, snow === 1), range);
    const when = d === 0 ? 'Today' : d === 1 ? 'Tomorrow' : `In ${d} days`;
    cell.title = `${when}: ${skyName(k, snow === 1)}, ${Math.round(low / 10)} to ${Math.round(high / 10)} °C`;
    cells.push(cell);
  }
  const ahead = h('div', 'fc-ahead');
  ahead.append(...cells);
  into.replaceChildren(now, ahead);
}

function skyIcon(kind: number, snow: boolean): SVGSVGElement {
  const sky = SKIES[kind] ?? SKIES[0];
  const wet = kind >= 2;
  const i = icon(wet && snow ? 'winter' : sky.icon);
  i.style.color = wet && snow ? SNOW.colour : sky.colour;
  return i;
}

function skyName(kind: number, snow: boolean): string {
  const sky = SKIES[kind] ?? SKIES[0];
  return kind >= 2 && snow ? (kind === 3 ? 'Blizzard' : SNOW.name) : sky.name;
}

function degrees(c: number): HTMLElement {
  const span = h('span', 'fc-temp', `${Math.round(c)}°`);
  span.style.color = tempColour(c);
  return span;
}
