// Starting co-op sessions: from the URL at page load (`startCoop`), or hosting the open world from the
// menu (`hostWorld`, ui/coop.ts). A room code goes over WebRTC between machines (`webrtc.ts`); any
// other room name stays between tabs of this browser (`broadcast.ts`, the dev loop). Also keeps this
// browser's player key and chosen name. The session itself runs in `host.ts` / `client.ts`.
//
//   ?host          host the latest world over WebRTC (the menu shows the code)
//   ?host=<room>   host it for tabs of this browser
//   ?join=<code>   join over WebRTC; ?join=<room> joins tabs of this browser
//   ?relay         force the TURN relay (testing)

import type { Game } from '../wasm/engine.js';
import { type Opened, message } from '../save/session';
import { newWorld } from '../save/store';
import { connectBroadcast, listenBroadcast } from './broadcast';
import { CoopClient } from './client';
import type { Coop } from './coop';
import { CoopHost } from './host';
import { isRoomCode } from './signal';
import { hostWebRtc, joinWebRtc } from './webrtc';

const KEY_STORAGE = 'opencraft.playerKey';
const NAME_STORAGE = 'opencraft.playerName';

/** Opens the world to play: the host's, for `?join=`, else `openSolo()`'s, which `?host` then hosts.
 * `coop` is null for solo play. Throws a message a player can read if joining fails. */
export async function startCoop(
  params: URLSearchParams,
  viewRadius: number,
  openSolo: () => Promise<Opened>,
): Promise<{ opened: Opened; coop: Coop | null }> {
  const relay = params.has('relay');
  const join = params.get('join');
  if (join) {
    const t = isRoomCode(join) ? await joinWebRtc(join, relay) : connectBroadcast(join);
    const client = await CoopClient.join(t, playerKey(), playerName(), viewRadius);
    const meta = newWorld(`Co-op game ${join.toUpperCase()}`, client.game.seed());
    return { opened: { game: client.game, meta, restored: false, notice: '' }, coop: leaveOnClose(client) };
  }
  const opened = await openSolo();
  const room = params.get('host');
  if (room === null) return { opened, coop: null };
  try {
    return { opened, coop: await hostWorld(opened.game, room, relay) };
  } catch (err) {
    return { opened: { ...opened, notice: `Couldn't open this world to others: ${message(err)}` }, coop: null };
  }
}

/** Opens `game`'s world to others: over WebRTC with a fresh room code when `room` is '', else for
 * tabs of this browser in `room`. Throws a message a player can read. */
export async function hostWorld(game: Game, room: string, relay = false): Promise<CoopHost> {
  if (room) {
    const host = new CoopHost(game, playerName());
    host.code = room;
    host.stopListening = listenBroadcast(room, (t) => host.addPeer(t));
    return leaveOnClose(host);
  }
  // Ask for the room first, so a failure leaves the game solo. No one can join before this returns:
  // no one has the code yet.
  let host: CoopHost | null = null;
  const { code, stop } = await hostWebRtc((t) => host?.addPeer(t), relay);
  host = new CoopHost(game, playerName());
  host.code = code;
  host.stopListening = stop;
  console.info(`co-op: hosting, invite link ${inviteLink(code)}`);
  return leaveOnClose(host);
}

/** The link that joins room `code`. */
export function inviteLink(code: string): string {
  return `${location.origin}${location.pathname}?join=${code}`;
}

/** The room in a pasted invite link or code, or null if it holds none. */
export function roomOf(text: string): string | null {
  const t = text.trim();
  const fromLink = /[?&]join=([^&#\s]+)/.exec(t);
  const room = fromLink ? decodeURIComponent(fromLink[1]) : t;
  return isRoomCode(room) ? room.toUpperCase() : null;
}

/** The name this player chose, or '' (the host then names them by player number). */
export function playerName(): string {
  return readStorage(NAME_STORAGE) ?? '';
}

export function setPlayerName(name: string): void {
  writeStorage(NAME_STORAGE, name.trim().slice(0, 24));
}

/** Says bye when the page closes, so the others don't wait for a timeout. */
function leaveOnClose<T extends Coop>(coop: T): T {
  window.addEventListener('pagehide', () => coop.close());
  return coop;
}

/** This browser's player key: random, kept, so the host gives back what this player had. */
function playerKey(): bigint {
  const kept = readStorage(KEY_STORAGE);
  if (kept && /^\d+$/.test(kept)) return BigInt(kept);
  const key = crypto.getRandomValues(new BigUint64Array(1))[0] || 1n;
  writeStorage(KEY_STORAGE, key.toString());
  return key;
}

function readStorage(name: string): string | null {
  try {
    return localStorage.getItem(name);
  } catch {
    return null;
  }
}

function writeStorage(name: string, value: string): void {
  try {
    localStorage.setItem(name, value);
  } catch {
    // Without storage the key lasts for this page only.
  }
}
