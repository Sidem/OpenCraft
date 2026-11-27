// The master volume on the menu's Sound page: a dial over everything (sound effects, ambience and music) and a button that
// mutes it all. Muting here is what the K key does; each channel keeps its own level and mute, so unmuting brings your mix back.
// The dial plays a short sample once it rests, so the new level can be judged by ear. Kept in sync with the sound system.

import './master-volume.css';
import type { SoundSystem } from '../audio/sound';
import { DEFAULT_LEVELS } from '../audio/settings';
import { button, h } from './dom';
import { Knob } from './knob';

export const MASTER_HELP = 'Turns everything up or down at once. Sound effects, ambience, factory beat and music keep their own balance.';
/** Milliseconds the dial rests before its sample plays. */
const SAMPLE_AFTER = 200;

export class MasterVolume {
  readonly el = h('div', 'master-volume');
  private readonly knob: Knob;
  private readonly mute: HTMLButtonElement;
  private sampleTimer = 0;

  constructor(private readonly sound: SoundSystem) {
    this.knob = new Knob({
      label: 'Master',
      low: 'silent',
      high: 'full',
      help: MASTER_HELP,
      value: sound.settings.master,
      defaultValue: DEFAULT_LEVELS.master,
      onInput: (value) => {
        sound.unlock();
        if (sound.settings.masterMuted) sound.setMasterMuted(false);
        sound.setMasterVolume(value);
        clearTimeout(this.sampleTimer);
        this.sampleTimer = window.setTimeout(() => sound.preview('place', 0), SAMPLE_AFTER);
      },
      onHelp: () => undefined, // the page shows the same words under the mixer
    });
    this.mute = button('secondary-btn master-mute', '', () => {
      sound.unlock();
      sound.setMasterMuted(!sound.settings.masterMuted);
    });
    this.el.append(this.knob.el, this.mute);
    sound.subscribe(() => this.sync());
    this.sync();
  }

  private sync(): void {
    const { master, masterMuted } = this.sound.settings;
    this.knob.set(master);
    this.mute.textContent = masterMuted ? 'Unmute all' : 'Mute all';
    this.mute.title = masterMuted ? 'Unmute everything (K)' : 'Mute everything (K)';
    this.mute.classList.toggle('active', masterMuted);
    this.el.classList.toggle('muted', masterMuted);
  }
}
