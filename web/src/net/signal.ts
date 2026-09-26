// Where co-op peers find each other: the signalling Worker (`signal/`, Cloudflare), deployed by the user.
// It hands out room codes and ICE servers and relays WebRTC offers, answers and candidates over one
// WebSocket per player (protocol in `signal/src/room.ts`); game data never passes through it. Deploying
// and its routes: docs/WORKFLOW.md section 4. `webrtc.ts` is the only user.

export const SIGNAL_URL = 'https://opencraft-signal.opencraft.workers.dev';

/** Room codes: six characters without look-alikes (0/O, 1/I/L), as the Worker makes them. */
const ROOM_CODE = /^[ABCDEFGHJKMNPQRSTUVWXYZ23456789]{6}$/;
const UNREACHABLE = "Couldn't reach the co-op server. Check your connection and try again.";

/** What the Worker's close codes mean, for the player. */
const CLOSE_REASONS: Record<number, string> = {
  4000: 'Someone is already hosting with this code.',
  4001: 'The host left.',
  4004: 'No game is hosted with this code.',
  4008: 'This game is full.',
};

/** Whether `text` (any case) is a room code rather than a room name for tabs on one machine. */
export function isRoomCode(text: string): boolean {
  return ROOM_CODE.test(text.toUpperCase());
}

/** A fresh room code for a host. */
export async function newRoom(): Promise<string> {
  const { code } = (await request('/room', 'POST')) as { code: string };
  return code;
}

/** STUN, plus TURN relays when the Worker has them set up. */
export async function fetchIce(): Promise<RTCIceServer[]> {
  const { iceServers } = (await request('/ice', 'GET')) as { iceServers: RTCIceServer[] };
  return iceServers;
}

/** The WebSocket into room `code`, as its host or a joiner. */
export function openRoom(code: string, role: 'host' | 'join'): WebSocket {
  return new WebSocket(`${SIGNAL_URL.replace(/^http/, 'ws')}/room/${code.toUpperCase()}?role=${role}`);
}

/** Why the room's socket closed, in words for the player. */
export function closeReason(code: number): string {
  return CLOSE_REASONS[code] ?? UNREACHABLE;
}

async function request(path: string, method: string): Promise<unknown> {
  try {
    const r = await fetch(SIGNAL_URL + path, { method });
    if (r.ok) return await r.json();
  } catch {
    // Offline, blocked or the Worker is down: the same message either way.
  }
  throw new Error(UNREACHABLE);
}
