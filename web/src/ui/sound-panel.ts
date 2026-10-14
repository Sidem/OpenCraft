// The menu's Sound page (`#sound-panel` in index.html): a volume slider with a mute button for sound
// effects and another for music, whether each is muted while the tab is not in focus, and the choice
// between the music's tracks. The sound designer button
// stays in the page's markup (main.ts wires it). To add a control: a row below.

import './sound-panel.css';
import { TRACKS } from '../audio/music';
import type { SoundSystem } from '../audio/sound';
import { button, h } from './dom';
import { VolumeControl } from './volume-control';

export class SoundPanel {
  readonly el = h('div', 'sound-panel');

  constructor(sound: SoundSystem) {
    const volume = (label: string, control: VolumeControl) => {
      const row = h('div', 'sound-panel-row');
      row.append(h('span', 'sound-panel-label', label), control.el);
      return row;
    };

    const tracks = h('div', 'sound-tracks');
    const buttons = TRACKS.map((t) => button('secondary-btn', t.label, () => {
      sound.unlock();
      sound.music.setTrack(t.id);
    }));
    tracks.append(...buttons);
    const help = h('p', 'sound-panel-help');
    const draw = () => {
      const current = sound.music.settings.track;
      buttons.forEach((b, i) => b.classList.toggle('active', TRACKS[i].id === current));
      help.textContent = TRACKS.find((t) => t.id === current)?.help ?? '';
    };
    sound.subscribe(draw);
    draw();

    const check = (label: string, get: () => boolean, set: (on: boolean) => void) => {
      const row = h('label', 'sound-panel-check');
      const box = h('input');
      box.type = 'checkbox';
      box.checked = get();
      box.addEventListener('change', () => set(box.checked));
      row.append(box, label);
      return row;
    };

    this.el.append(
      h('h3', '', 'Volume'),
      volume('Sound effects', new VolumeControl(sound, 'effects')),
      volume('Music', new VolumeControl(sound, 'music')),
      h('h3', '', 'When the tab is not in focus'),
      check('Mute music', () => sound.music.settings.muteInBackground, (on) => sound.music.setMuteInBackground(on)),
      check('Mute sound effects', () => sound.settings.muteInBackground, (on) => sound.setMuteInBackground(on)),
      h('h3', '', 'Music'),
      tracks,
      help,
    );
  }
}
