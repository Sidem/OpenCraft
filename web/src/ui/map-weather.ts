// The weather on the world map (worldmap.ts): a row of layer buttons over the view (no weather, temperature, rain and
// clouds, air pressure, wind) and, for the chosen layer, the weather measured inside each working Weather Station's
// circle (`game.weather_map`, api/weather_map.rs) as coloured squares, with wind arrows on the wind layer. Every
// station's reach is ringed (dashed while it has no power). The row shows only once a station stands. Samples are read
// again only when the view, the grid or the air (`weather_version`) changes. Colours and parsing: weather-scale.ts;
// to add a layer, a row there.

import './map-weather.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import type { Info } from './map-filter';
import { icon } from './seasons';
import { gridStep, LAYERS, type Layer, parseSamples, type Sample, sampleFill, speed, STRONG_WIND, windColour, windFrom } from './weather-scale';

/** About how far apart samples are on screen (px), and where wind arrows go (every n-th sample each way). */
const SAMPLE_PX = 14;
const ARROW_EVERY = 2;
const STATION_COLOUR = '#7fd4ff';

type View = { w: number; h: number; bpp: number; left: number; top: number };

export class MapWeather {
  /** The layer buttons, for the map to place over its view. */
  readonly el = h('div', 'mw-bar hidden');
  private layer: Layer = 'off';
  private readonly buttons: HTMLButtonElement[] = [];
  private key = '';
  private samples: Sample[] = [];
  private step = 8;

  constructor(private readonly game: Game) {
    LAYERS.forEach((l) => {
      const b = button('mw-btn', l.icon ? '' : '∅', () => this.pick(l.key));
      if (l.icon) b.append(icon(l.icon));
      b.title = l.name;
      b.setAttribute('aria-label', l.name);
      this.buttons.push(b);
      this.el.append(b);
    });
    this.pick('off');
  }

  private pick(layer: Layer): void {
    this.layer = layer;
    this.key = '';
    LAYERS.forEach((l, i) => this.buttons[i].classList.toggle('on', l.key === layer));
  }

  /** Draws the chosen layer and the stations' circles; `at` turns a world column into overlay pixels. */
  draw(c: CanvasRenderingContext2D, at: (x: number, z: number) => [number, number], dpr: number, v: View): void {
    const stations = this.game.weather_stations();
    this.el.classList.toggle('hidden', stations.length === 0);
    if (this.layer === 'off' || stations.length === 0) return;
    this.read(v);
    const size = (this.step / v.bpp) * dpr;
    c.save();
    c.beginPath(); // the squares stop at the working stations' circles
    for (let i = 0; i < stations.length; i += 4) {
      if (!stations[i + 3]) continue;
      const [x, y] = at(stations[i] + 0.5, stations[i + 1] + 0.5);
      c.moveTo(x + (stations[i + 2] / v.bpp) * dpr, y);
      c.arc(x, y, (stations[i + 2] / v.bpp) * dpr, 0, Math.PI * 2);
    }
    c.clip();
    for (const s of this.samples) {
      const fill = sampleFill(this.layer, s);
      if (!fill) continue;
      const [x, y] = at(s.x, s.z);
      c.globalAlpha = fill.alpha;
      c.fillStyle = fill.colour;
      c.fillRect(x - size / 2, y - size / 2, size + 0.5, size + 0.5);
    }
    c.globalAlpha = 1;
    if (this.layer === 'wind') this.arrows(c, at, size, dpr);
    c.restore();
    c.save();
    for (let i = 0; i < stations.length; i += 4) {
      const [x, y] = at(stations[i] + 0.5, stations[i + 1] + 0.5);
      c.beginPath();
      c.arc(x, y, (stations[i + 2] / v.bpp) * dpr, 0, Math.PI * 2);
      c.lineWidth = 1.5 * dpr;
      c.strokeStyle = STATION_COLOUR;
      c.setLineDash(stations[i + 3] ? [] : [6 * dpr, 5 * dpr]);
      c.stroke();
    }
    c.restore();
  }

  /** The hover line for the sample nearest column (x, z), when the layer is on and one is within a grid step. */
  near(x: number, z: number): Info[] {
    if (this.layer === 'off') return [];
    let best: Sample | null = null, d = this.step * 0.75;
    for (const s of this.samples) {
      const e = Math.hypot(s.x - x, s.z - z);
      if (e < d) [best, d] = [s, e];
    }
    if (!best) return [];
    const from = windFrom(best.east, best.south);
    const wind = from ? `wind ${Math.round(speed(best))} m/s from the ${from}` : 'calm';
    const sky = best.precip > 0.02 ? (best.snow ? 'snowing' : 'raining') : `${Math.round(best.cover * 100)} % cloud`;
    return [{
      colour: STATION_COLOUR,
      title: `${Math.round(best.temp)} °C, ${sky}`,
      detail: `Humidity ${Math.round(best.humidity)} %, pressure ${Math.round(best.pressure)} hPa, ${wind}. Measured by a Weather Station.`,
    }];
  }

  /** Reads the samples for this view again when it, the grid step or the air changed. */
  private read(v: View): void {
    this.step = gridStep(v.bpp, SAMPLE_PX);
    const s = this.step;
    const x0 = Math.floor(v.left / s) * s, z0 = Math.floor(v.top / s) * s;
    const x1 = Math.ceil((v.left + v.w * v.bpp) / s) * s, z1 = Math.ceil((v.top + v.h * v.bpp) / s) * s;
    const key = `${x0},${z0},${x1},${z1},${s},${this.game.weather_version()},${this.game.weather_stations().join()}`;
    if (key === this.key) return;
    this.key = key;
    this.samples = parseSamples(this.game.weather_map(x0, z0, x1, z1, s), this.game.weather_map_fields());
  }

  /** An arrow along the wind on every `ARROW_EVERY`-th sample, longer and redder as it blows harder. */
  private arrows(c: CanvasRenderingContext2D, at: (x: number, z: number) => [number, number], size: number, dpr: number): void {
    const every = this.step * ARROW_EVERY;
    c.lineCap = 'round';
    for (const s of this.samples) {
      if (s.x % every !== 0 || s.z % every !== 0) continue;
      const ms = speed(s);
      if (ms < 0.3) continue;
      const [x, y] = at(s.x, s.z);
      const len = size * ARROW_EVERY * 0.45 * Math.min(1, 0.35 + ms / STRONG_WIND);
      const ux = s.east / ms, uy = s.south / ms;
      const tx = x + ux * len, ty = y + uy * len, hx = x - ux * len, hy = y - uy * len;
      const head = Math.max(3 * dpr, len * 0.4);
      c.beginPath();
      c.moveTo(hx, hy);
      c.lineTo(tx, ty);
      c.moveTo(tx - ux * head - uy * head * 0.6, ty - uy * head + ux * head * 0.6);
      c.lineTo(tx, ty);
      c.lineTo(tx - ux * head + uy * head * 0.6, ty - uy * head - ux * head * 0.6);
      c.lineWidth = 3 * dpr;
      c.strokeStyle = 'rgba(0, 0, 0, 0.55)';
      c.stroke();
      c.lineWidth = 1.5 * dpr;
      c.strokeStyle = windColour(ms);
      c.stroke();
    }
  }
}
