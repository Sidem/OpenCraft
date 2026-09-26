// What the rest of the page sees of a co-op session, host or client side (`host.ts`, `client.ts`),
// and the timings both sides share. `session.ts` starts one.

/** How often the host measures round trips. */
export const PING_MS = 1000;
/** Nothing heard from the other side for this long: the connection is taken as lost. */
export const SILENT_MS = 10_000;

export interface PlayerInfo {
  id: number;
  name: string;
  /** Round trip to the host in ms; null for the host itself, or before the first ping. */
  ping: number | null;
  you: boolean;
  host: boolean;
}

export interface Coop {
  /** Every frame, right after `game.update`. */
  pump(): void;
  /** Leaves on purpose: says bye and closes every connection. */
  close(): void;
  /** The name player `id` chose, if they are in this session. */
  name(id: number): string | undefined;
  /** Everyone in the session, this player included. */
  players(): PlayerInfo[];
  /** Called with a line for the player when someone joins or leaves. */
  onNotice: (text: string) => void;
  /** Called once when the session ends from the other side (a client only), with a readable reason. */
  onEnd: (reason: string) => void;
}

/** What players who didn't choose a name are called. */
export function defaultName(id: number): string {
  return `Player ${id + 1}`;
}
