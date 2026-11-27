// The menu's Sound page (`#sound-panel` in index.html): a mixer with the master dial (ui/master-volume.ts) beside a level and
// mute for sound effects, ambience (wind, water, birds) and music; the music's modes; whether sound is muted while the tab is
// not in focus; and a reset for every level. The sound designer opens with the O key, not from here. To add a control: a row below.

import './sound-panel.css';
import { MODES } from '../audio/music';
import type { SoundSystem } from '../audio/sound';
import { button, h } from './dom';
import { MASTER_HELP, MasterVolume } from './master-volume';
import { VolumeControl } from './volume-control';

export class SoundPanel {
  readonly el = h('div', 'sound-panel');

  constructor(sound: SoundSystem) {
    const channel = (label: string, about: string, control: VolumeControl) => {
      const row = h('div', 'sound-panel-row');
      const name = h('span', 'sound-panel-label', label);
      name.append(h('small', '', about));
      row.append(name, control.el);
      return row;
    };
    const mixer = h('div', 'sound-mixer');
    const channels = h('div', 'sound-channels');
    channels.append(
      channel('Sound effects', 'Mining, building, steps', new VolumeControl(sound, 'effects')),
      channel('Ambience', 'Wind, water, birds', new VolumeControl(sound, 'ambience')),
      channel('Factory beat', 'Machines play a groove', new VolumeControl(sound, 'beat')),
      channel('Music', 'Pieces that follow you', new VolumeControl(sound, 'music')),
    );
    mixer.append(new MasterVolume(sound).el, channels);

    const tracks = h('div', 'sound-tracks');
    const buttons = MODES.map((m) => button('secondary-btn', m.label, () => {
      sound.unlock();
      sound.music.setMode(m.id);
    }));
    tracks.append(...buttons);
    const help = h('p', 'sound-panel-help');

    const syncs: (() => void)[] = [];
    const check = (label: string, get: () => boolean, set: (on: boolean) => void) => {
      const row = h('label', 'sound-panel-check');
      const box = h('input');
      box.type = 'checkbox';
      box.addEventListener('change', () => set(box.checked));
      syncs.push(() => (box.checked = get()));
      row.append(box, label);
      return row;
    };
    const checks = [
      check('Mute music', () => sound.music.settings.muteInBackground, (on) => sound.music.setMuteInBackground(on)),
      check('Mute sound effects and ambience', () => sound.settings.muteInBackground, (on) => sound.setMuteInBackground(on)),
    ];

    const draw = () => {
      const current = sound.music.settings.mode;
      buttons.forEach((b, i) => b.classList.toggle('active', MODES[i].id === current));
      help.textContent = MODES.find((m) => m.id === current)?.help ?? '';
      syncs.forEach((fn) => fn());
    };
    sound.subscribe(draw);
    draw();

    this.el.append(
      mixer,
      h('p', 'sound-panel-help', MASTER_HELP),
      h('h3', '', 'Music'),
      tracks,
      help,
      h('h3', '', 'When the tab is not in focus'),
      ...checks,
      button('secondary-btn sound-reset', 'Reset levels', () => sound.resetLevels()),
    );
  }
}
