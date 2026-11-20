// The music library: one row per piece (composed in the MusicComposer project, see docs/WORKFLOW.md). Files are
// Ogg Opus passes (64 kbit/s) that play once; music.ts streams them from a media element. `gain` trims a piece to the others
// by ear (1 is as rendered, all at about -18 LUFS, checked by scripts/check-music.mjs).
// To add a piece: drop the .ogg in audio/music/ and add a row. `contexts` are the places and eras it is made for
// (day, night, underground, desert, highlands, lowlands, basalt, sea, town, factory_early, factory_mid, factory_high,
// space), plus 'any' if it may play where no piece is made for the place; `mood` picks it for the "always calm" and
// "always adventurous" modes.

import type { PieceInfo } from './music-director';

export interface Piece extends PieceInfo {
  label: string;
  url: string;
  gain: number;
}

const file = (name: string) => new URL(`./music/${name}.ogg`, import.meta.url).href;

export const PIECES: readonly Piece[] = [
  { id: 'day_calm', label: 'Open Ground (calm)', mood: 'calm', contexts: ['day', 'any'], gain: 1, url: file('day_calm') },
  { id: 'day_lively', label: 'Open Ground', mood: 'lively', contexts: ['day', 'any'], gain: 0.9, url: file('day_lively') }, // peaks at -0.02 dBFS
  { id: 'night', label: 'Lantern Hours', mood: 'calm', contexts: ['night'], gain: 1, url: file('night') },
  { id: 'underground', label: 'Deep Seam', mood: 'calm', contexts: ['underground'], gain: 1, url: file('underground') },
  { id: 'desert', label: 'Salt Road', mood: 'lively', contexts: ['desert'], gain: 1, url: file('desert') },
  { id: 'highlands', label: 'High Pasture', mood: 'lively', contexts: ['highlands'], gain: 1, url: file('highlands') },
  { id: 'forest', label: 'Canopy & Moss', mood: 'calm', contexts: ['lowlands'], gain: 1, url: file('forest') },
  { id: 'basalt', label: 'Obsidian Forge', mood: 'lively', contexts: ['basalt'], gain: 1, url: file('basalt') },
  { id: 'sea', label: 'Tide & Timber', mood: 'lively', contexts: ['sea'], gain: 1, url: file('sea') },
  { id: 'town', label: 'Lantern Market', mood: 'calm', contexts: ['town'], gain: 1, url: file('town') },
  { id: 'industrial_early', label: 'Copper & Cogs', mood: 'lively', contexts: ['factory_early'], gain: 1, url: file('industrial_early') },
  { id: 'industrial', label: 'Iron Cadence', mood: 'lively', contexts: ['factory_mid'], gain: 1, url: file('industrial') },
  { id: 'industrial_hightech', label: 'Silicon Pulse', mood: 'lively', contexts: ['factory_high'], gain: 1, url: file('industrial_hightech') },
  { id: 'space', label: 'Orbital Drift', mood: 'calm', contexts: ['space'], gain: 1, url: file('space') },
];
