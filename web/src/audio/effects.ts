// Sound effect files: audio/effects/*.ogg (64 kbit/s mono Opus), made outside the game from recordings and rendered
// sounds (sources and licences in effects/CREDITS.md). Files are found by name: <action>_<material>_<variant>.ogg for
// sounds that depend on the block's material, <action>_<variant>.ogg for the rest, variants counted from 0. Each sound
// is levelled by its row in effect-levels.ts. A sound with no files is silent; one still loading is skipped.
// To add or replace a sound: drop its files in effects/ under that name, add a row to effect-levels.ts and credit
// the source in effects/CREDITS.md.

import { EFFECT_GAIN } from './effect-levels';

const URLS = import.meta.glob('./effects/*.ogg', { eager: true, query: '?url', import: 'default' }) as Record<string, string>;

/** A sound's file stem: `dig_stone`, or `pickup` for a sound without a material. */
export const effectName = (action: string, material?: string): string => (material ? `${action}_${material}` : action);

/** Playback gain that levels a sound to the others. */
export const effectGain = (name: string): number => EFFECT_GAIN[name] ?? 1;

const url = (name: string, variant: number): string | undefined => URLS[`./effects/${name}_${variant}.ogg`];

/** Decoded effect files for one AudioContext, all fetched once when it starts. */
export class EffectBank {
  /** Variants per sound, in variant order; `undefined` until decoded. */
  private readonly sounds = new Map<string, (AudioBuffer | undefined)[]>();

  constructor(private readonly ctx: AudioContext) {
    for (const path of Object.keys(URLS)) {
      const [, name, variant] = /^\.\/effects\/(.+)_(\d+)\.ogg$/.exec(path) ?? [];
      if (!name) continue;
      const list = this.sounds.get(name) ?? [];
      list.length = Math.max(list.length, Number(variant) + 1);
      this.sounds.set(name, list);
    }
    for (const [name, list] of this.sounds) for (let v = 0; v < list.length; v++) void this.load(name, v);
  }

  /** A random variant of the sound, or undefined if it has no files or none has loaded yet. */
  pick(name: string): AudioBuffer | undefined {
    const list = this.sounds.get(name);
    if (!list?.length) return undefined;
    return list[Math.floor(Math.random() * list.length)] ?? list.find((b) => b);
  }

  private async load(name: string, variant: number): Promise<void> {
    const src = url(name, variant);
    if (!src) return;
    try {
      const data = await (await fetch(src)).arrayBuffer();
      this.sounds.get(name)![variant] = await this.ctx.decodeAudioData(data);
    } catch {
      // Missing or undecodable: that variant stays silent.
    }
  }
}
