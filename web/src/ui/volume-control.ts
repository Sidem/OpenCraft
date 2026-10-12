// A volume control for one sound channel (sound effects or music): mute button, slider and level
// readout, kept in sync with the sound system. Used on the menu's sound page and in the sound
// designer's footer (effects).

import './volume-control.css';
import type { SoundSystem } from '../audio/sound';
import { button, h } from './dom';

const SPEAKER_ON =
  '<svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M3 9v6h4l5 4V5L7 9z"/><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" d="M15.5 8.5a5 5 0 0 1 0 7M18.5 5.5a9 9 0 0 1 0 13"/></svg>';
const SPEAKER_OFF =
  '<svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M3 9v6h4l5 4V5L7 9z"/><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" d="M16 9.5l5 5M21 9.5l-5 5"/></svg>';

export type Channel = 'effects' | 'music';
const NAMES: Record<Channel, string> = { effects: 'Sound effects', music: 'Music' };

export class VolumeControl {
  readonly el = h('div', 'volume-control');
  private readonly toggle: HTMLButtonElement;
  private readonly range = h('input');
  private readonly readout = h('span', 'volume-value');

  constructor(private readonly sound: SoundSystem, private readonly channel: Channel = 'effects') {
    this.toggle = button('mute-toggle', '', () => {
      sound.unlock();
      this.setMuted(!this.state.muted);
    });
    this.range.type = 'range';
    this.range.min = '0';
    this.range.max = '100';
    this.range.setAttribute('aria-label', `${NAMES[channel]} volume`);
    this.range.addEventListener('input', () => {
      sound.unlock();
      if (this.state.muted) this.setMuted(false);
      if (channel === 'music') sound.music.setVolume(Number(this.range.value));
      else sound.setVolume(Number(this.range.value));
    });
    // A sample on release, so the new level can be judged (music is already playing).
    if (channel === 'effects') this.range.addEventListener('change', () => sound.preview('place', 0));
    this.el.append(this.toggle, this.range, this.readout);
    sound.subscribe(() => this.sync());
    this.sync();
  }

  private get state(): { volume: number; muted: boolean } {
    return this.channel === 'music' ? this.sound.music.settings : this.sound.settings;
  }

  private setMuted(muted: boolean): void {
    if (this.channel === 'music') this.sound.music.setMuted(muted);
    else this.sound.setMuted(muted);
  }

  private sync(): void {
    const { volume, muted } = this.state;
    const name = NAMES[this.channel].toLowerCase();
    this.range.value = String(volume);
    this.readout.textContent = muted ? 'Muted' : `${volume}%`;
    this.toggle.innerHTML = muted || volume === 0 ? SPEAKER_OFF : SPEAKER_ON;
    this.toggle.title = muted ? `Unmute ${name}` : `Mute ${name}`;
    this.toggle.setAttribute('aria-label', this.toggle.title);
    this.el.classList.toggle('muted', muted);
  }
}
