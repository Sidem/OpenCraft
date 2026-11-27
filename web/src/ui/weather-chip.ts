// The weather chip under the season chip (season-chip.ts): the sky's icon, the temperature and an arrow the way the
// wind blows with its speed; under them, where a Weather Station measures (`weather_measured_here`), the relative
// humidity and the air pressure, else dim dashes. Read from `weather_here` (api/weather.rs) twice a second. Owned and
// updated by the Minimap (it shows and hides with it); the layout height is mirrored as WEATHER_CHIP_PX there. Words
// only in the tooltip.

import './weather-chip.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { skyIcon, skyName } from './forecast';
import { icon } from './seasons';
import { compassPoint, tempColour, windColour } from './weather-scale';

export const WEATHER_CHIP_PX = 46; // must match .weather-chip height in weather-chip.css
const READ_MS = 500;

export class WeatherChip {
  readonly el = h('div', 'weather-chip');
  private readonly sky = h('span', 'wc-sky');
  private readonly temp = h('span', 'wc-temp');
  private readonly arrow = h('span', 'wc-arrow', '↑');
  private readonly wind = h('span', 'wc-wind');
  private readonly humidity = h('span', 'wc-num');
  private readonly pressure = h('span', 'wc-num');
  private readonly measured = h('div', 'wc-row wc-measured');
  private last = -Infinity;
  private key = '';

  constructor(private readonly game: Game) {
    const top = h('div', 'wc-row');
    top.append(this.sky, this.temp, h('span', 'wc-gap'), this.arrow, this.wind, h('span', 'wc-unit', 'm/s'));
    const drop = icon('drop'), gauge = icon('gauge');
    drop.classList.add('wc-drop');
    this.measured.append(drop, this.humidity, h('span', 'wc-gap'), gauge, this.pressure, h('span', 'wc-unit', 'hPa'));
    this.el.append(top, this.measured);
  }

  /** Call every frame while shown; reads the weather twice a second and touches the DOM only when it changed. */
  update(now: number): void {
    if (now - this.last < READ_MS) return;
    this.last = now;
    const w = this.game.weather_here();
    const snow = w[3] > 0.5, temp = Math.round(w[4]), kind = Math.round(w[5]);
    const hum = Math.round(w[6]), hpa = Math.round(w[8]), ms = Math.round(w[9]), from = Math.round(w[10]);
    const known = this.game.weather_measured_here();
    const key = `${snow}|${temp}|${kind}|${ms}|${from}|${known}|${hum}|${hpa}`;
    if (key === this.key) return;
    const was = this.key.split('|');
    this.key = key;
    if (was[0] !== String(snow) || was[2] !== String(kind)) this.sky.replaceChildren(skyIcon(kind, snow));
    this.temp.textContent = `${temp}°`;
    this.temp.style.color = tempColour(temp);
    this.arrow.style.visibility = ms > 0 ? 'visible' : 'hidden';
    this.arrow.style.transform = `rotate(${from + 180}deg)`;
    this.arrow.style.color = windColour(w[9]);
    this.wind.textContent = `${ms}`;
    this.measured.classList.toggle('dim', !known);
    this.humidity.textContent = known ? `${hum}%` : '–';
    this.pressure.textContent = known ? `${hpa}` : '–';
    const wind = ms > 0 ? `wind ${ms} m/s from the ${compassPoint(from)}` : 'calm';
    const more = known
      ? `Humidity ${hum} %, pressure ${hpa} hPa (measured by a Weather Station).`
      : 'Build a Weather Station nearby to measure humidity and air pressure.';
    this.el.title = `${skyName(kind, snow)}, ${temp} °C, ${wind}. ${more} Shift+N: calendar`;
  }
}
