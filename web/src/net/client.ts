// The client side of a co-op session, over one `Transport` to the host. Sends its outbox and body
// state, feeds the host's frames, states and items to the engine, answers pings, and compares the
// host's checksums with its own. When they differ (a bug: logged with the tick and both hashes), or
// it falls more than `LAG_TICKS` behind, it asks for a fresh snapshot and resyncs in place
// (`game.resync`). A bye, a closed transport or `SILENT_MS` of silence ends the session (`onEnd`).

import { Game } from '../wasm/engine.js';
import { message } from '../save/session';
import { type Coop, type PlayerInfo, SILENT_MS } from './coop';
import { BUILD_ID, decode, encode, type Message } from './protocol';
import type { Transport } from './transport';

/** How long a joiner waits for the host to answer. */
const JOIN_TIMEOUT_MS = 5000;
/** Ticks the host has confirmed that this game hasn't run yet, beyond which it resyncs (5 seconds). */
const LAG_TICKS = 300;
const HOST_LEFT = 'The host left, so the session ended.';
const LOST = 'Lost the connection to the host.';

export class CoopClient implements Coop {
  /** Checksums compared, how many differed, and resyncs done (for the console and tests). */
  checked = 0;
  mismatches = 0;
  resyncs = 0;
  onNotice = (_text: string) => {};
  onEnd = (_reason: string) => {};
  private readonly hostHashes = new Map<bigint, bigint>();
  private readonly ownHashes = new Map<bigint, bigint>();
  private names = new Map<number, string>();
  private named = false;
  private pings = new Map<number, number>();
  private hostId = 0;
  private heard = performance.now();
  private resyncing = false;
  private ended = false;

  /** Says hello and waits for the host's welcome. Throws a message a player can read. */
  static join(t: Transport, key: bigint, name: string, viewRadius: number): Promise<CoopClient> {
    return new Promise((resolve, reject) => {
      const fail = (reason: string) => {
        clearTimeout(timer);
        t.close();
        reject(new Error(reason));
      };
      const timer = setTimeout(() => fail('No game is hosted here.'), JOIN_TIMEOUT_MS);
      t.onClose = () => fail('The host closed the connection.');
      t.onMessage = (bytes) => {
        const m = decode(bytes);
        if (m?.type === 'refuse') fail(m.reason);
        if (m?.type !== 'welcome') return;
        clearTimeout(timer);
        try {
          resolve(new CoopClient(t, Game.from_snapshot(m.snapshot, m.player, viewRadius)));
        } catch (err) {
          fail(message(err));
        }
      };
      t.send(encode({ type: 'hello', build: BUILD_ID, key, name }));
    });
  }

  private constructor(
    private readonly t: Transport,
    readonly game: Game,
  ) {
    t.onMessage = (bytes) => this.receive(decode(bytes));
    t.onClose = () => this.end(LOST);
  }

  name(id: number): string | undefined {
    return this.names.get(id);
  }

  /** The host's name, once the names arrived. */
  hostName(): string | undefined {
    return this.names.get(this.hostId);
  }

  players(): PlayerInfo[] {
    const local = this.game.local_player();
    return [...this.names].map(([id, name]) => ({
      id,
      name,
      ping: id === this.hostId ? null : (this.pings.get(id) ?? null),
      you: id === local,
      host: id === this.hostId,
    }));
  }

  pump(): void {
    if (this.ended) return;
    if (performance.now() - this.heard > SILENT_MS) {
      this.t.close();
      this.end(LOST);
      return;
    }
    const outbox = this.game.take_outbox();
    if (outbox.length > 0) this.t.send(encode({ type: 'actions', bytes: outbox }));
    const state = this.game.take_states();
    if (state.length > 0) this.t.send(encode({ type: 'state', bytes: state }));
    const lag = this.game.confirmed_tick() - this.game.core_tick();
    if (lag > LAG_TICKS) this.requestResync(`${lag} ticks behind the host`);
    const own = this.game.take_checksums();
    for (let i = 0; i + 1 < own.length; i += 2) this.ownHashes.set(own[i], own[i + 1]);
    for (const [tick, hash] of this.ownHashes) {
      const theirs = this.hostHashes.get(tick);
      if (theirs === undefined) continue;
      this.checked++;
      if (theirs !== hash) {
        this.mismatches++;
        this.requestResync(`the cores differ at tick ${tick} (host ${theirs.toString(16)}, here ${hash.toString(16)})`);
      }
      this.ownHashes.delete(tick);
      this.hostHashes.delete(tick);
    }
  }

  close(): void {
    this.ended = true;
    this.t.send(encode({ type: 'bye' }));
    this.t.close();
  }

  private receive(m: Message | null): void {
    this.heard = performance.now();
    if (m?.type === 'frames') {
      if (!this.game.push_frames(m.bytes)) this.requestResync("the host's frames don't follow on");
    } else if (m?.type === 'checksum') {
      this.hostHashes.set(m.tick, m.hash);
    } else if (m?.type === 'states') {
      this.game.push_states(m.bytes);
    } else if (m?.type === 'items') {
      this.game.push_items(m.bytes);
    } else if (m?.type === 'names') {
      this.rename(new Map(m.names));
    } else if (m?.type === 'ping') {
      this.t.send(encode({ type: 'pong', time: m.time }));
      this.hostId = m.host;
      this.pings = new Map(m.pings);
    } else if (m?.type === 'snapshot') {
      this.resync(m.bytes);
    } else if (m?.type === 'bye') {
      this.t.close();
      this.end(HOST_LEFT);
    }
  }

  /** New names from the host: says who joined or left since the last list. */
  private rename(next: Map<number, string>): void {
    const local = this.game.local_player();
    if (this.named) {
      for (const [id, name] of next) if (id !== local && !this.names.has(id)) this.onNotice(`${name} joined`);
      for (const [id, name] of this.names) if (!next.has(id)) this.onNotice(`${name} left`);
    }
    this.names = next;
    this.named = true;
  }

  private requestResync(why: string): void {
    if (this.resyncing || this.ended) return;
    this.resyncing = true;
    console.warn(`co-op: ${why}; asking the host for a fresh copy of the world`);
    this.t.send(encode({ type: 'resync' }));
  }

  private resync(snapshot: Uint8Array): void {
    try {
      this.game.resync(snapshot);
    } catch (err) {
      this.t.close();
      return this.end(`The world couldn't be brought back in step with the host: ${message(err)}`);
    }
    this.hostHashes.clear();
    this.ownHashes.clear();
    this.resyncing = false;
    this.resyncs++;
    console.info(`co-op: back in step with the host at tick ${this.game.core_tick()}`);
  }

  private end(reason: string): void {
    if (this.ended) return;
    this.ended = true;
    console.info(`co-op: ${reason}`);
    this.onEnd(reason);
  }
}
