// The host side of a co-op session, over any `Transport` per peer. Answers each joiner's hello with a
// welcome (player id and snapshot) or a refusal, stamps clients' actions and takes their body states,
// and after every `game.update` sends the new frames, checksums and body states to everyone welcomed,
// and each one the items near it. Everyone's names go out whenever someone joins or leaves; a ping
// every second measures each round trip and tells everyone the results. A client that asks to resync
// gets a fresh snapshot. A peer that says bye, closes, or stays silent for `SILENT_MS` leaves:
// `host_leave` keeps their things under their key.
// Invariant: `pump` runs right after every `update`, so a message (handled between frames) always
// sees every frame taken, and a snapshot's joiner gets frames from exactly its tick.

import type { Game } from '../wasm/engine.js';
import { message } from '../save/session';
import { BUILD_ID, decode, encode, type Message } from './protocol';
import { type Coop, defaultName, type PlayerInfo, PING_MS, SILENT_MS } from './coop';
import type { Transport } from './transport';

export class CoopHost implements Coop {
  /** How others join: a room code (WebRTC) or a room name (tabs of this browser), once open. */
  code = '';
  onNotice = (_text: string) => {};
  onEnd = (_reason: string) => {};
  /** Set by whoever feeds `addPeer`, to stop accepting joiners on `close`. */
  stopListening = () => {};
  private readonly peers = new Set<HostPeer>();
  private readonly names = new Map<number, string>();
  private readonly local: number;
  private lastPing = 0;

  constructor(
    private readonly game: Game,
    name: string,
  ) {
    game.start_host();
    this.local = game.local_player();
    this.names.set(this.local, name || defaultName(this.local));
  }

  name(id: number): string | undefined {
    return this.names.get(id);
  }

  players(): PlayerInfo[] {
    const list: PlayerInfo[] = [{ id: this.local, name: this.names.get(this.local)!, ping: null, you: true, host: true }];
    for (const p of this.peers) {
      if (p.id !== null) list.push({ id: p.id, name: p.name, ping: p.ping, you: false, host: false });
    }
    return list;
  }

  /** A new connection; it becomes a player once its hello is accepted. */
  addPeer(t: Transport): void {
    const peer: HostPeer = { t, id: null, name: '', ping: null, heard: performance.now() };
    this.peers.add(peer);
    t.onMessage = (bytes) => this.receive(peer, decode(bytes));
    t.onClose = () => this.drop(peer);
  }

  pump(): void {
    const frames = this.game.take_frames();
    const checksums = this.game.take_checksums();
    const states = this.game.take_states();
    const out: Uint8Array[] = [];
    if (frames.length > 0) out.push(encode({ type: 'frames', bytes: frames }));
    if (states.length > 0) out.push(encode({ type: 'states', bytes: states }));
    for (let i = 0; i + 1 < checksums.length; i += 2) {
      out.push(encode({ type: 'checksum', tick: checksums[i], hash: checksums[i + 1] }));
    }
    const now = performance.now();
    if (now - this.lastPing >= PING_MS) {
      this.lastPing = now;
      out.push(encode({ type: 'ping', time: now, host: this.local, pings: this.pings() }));
    }
    for (const peer of [...this.peers]) {
      if (now - peer.heard > SILENT_MS) {
        console.info(`co-op: ${peer.name || 'a joiner'} went silent`);
        peer.t.close();
        this.drop(peer);
        continue;
      }
      if (peer.id === null) continue;
      for (const m of out) peer.t.send(m);
      const items = this.game.take_item_view(peer.id);
      if (items.length > 0) peer.t.send(encode({ type: 'items', bytes: items }));
    }
  }

  close(): void {
    this.stopListening();
    for (const peer of [...this.peers]) {
      peer.t.send(encode({ type: 'bye' }));
      peer.t.close();
      this.drop(peer);
    }
  }

  private receive(peer: HostPeer, m: Message | null): void {
    peer.heard = performance.now();
    if (m?.type === 'hello' && peer.id === null) this.welcome(peer, m);
    else if (m?.type === 'actions' && peer.id !== null) {
      if (!this.game.host_stamp(peer.id, m.bytes)) console.warn(`co-op: refused actions from ${peer.name}`);
    } else if (m?.type === 'state' && peer.id !== null) {
      this.game.host_state(peer.id, m.bytes);
    } else if (m?.type === 'pong') {
      peer.ping = Math.round(peer.heard - m.time);
    } else if (m?.type === 'resync' && peer.id !== null) {
      console.info(`co-op: sending ${peer.name} a fresh copy of the world`);
      peer.t.send(encode({ type: 'snapshot', bytes: this.game.snapshot() }));
    } else if (m?.type === 'bye') {
      peer.t.close();
      this.drop(peer);
    }
  }

  private welcome(peer: HostPeer, hello: Extract<Message, { type: 'hello' }>): void {
    if (hello.build !== BUILD_ID) {
      return this.refuse(peer, 'The host runs a different version of OpenCraft. Both of you: reload the page to update.');
    }
    try {
      peer.id = this.game.host_join(hello.key);
    } catch (err) {
      return this.refuse(peer, message(err));
    }
    peer.name = hello.name || defaultName(peer.id);
    peer.t.send(encode({ type: 'welcome', player: peer.id, snapshot: this.game.snapshot() }));
    this.names.set(peer.id, peer.name);
    this.sendNames();
    this.onNotice(`${peer.name} joined`);
    console.info(`co-op: ${peer.name} joined as player ${peer.id}`);
  }

  private pings(): [number, number][] {
    const list: [number, number][] = [];
    for (const p of this.peers) if (p.id !== null && p.ping !== null) list.push([p.id, p.ping]);
    return list;
  }

  private sendNames(): void {
    const m = encode({ type: 'names', names: [...this.names] });
    for (const peer of this.peers) if (peer.id !== null) peer.t.send(m);
  }

  private refuse(peer: HostPeer, reason: string): void {
    peer.t.send(encode({ type: 'refuse', reason }));
    peer.t.close();
    this.peers.delete(peer);
  }

  private drop(peer: HostPeer): void {
    if (!this.peers.delete(peer) || peer.id === null) return;
    this.game.host_leave(peer.id);
    this.names.delete(peer.id);
    this.sendNames();
    this.onNotice(`${peer.name} left`);
    console.info(`co-op: ${peer.name} left`);
  }
}

interface HostPeer {
  t: Transport;
  /** The player id, once welcomed. */
  id: number | null;
  name: string;
  /** The latest round trip in ms, once a ping came back. */
  ping: number | null;
  /** When anything last arrived from them (`performance.now()`). */
  heard: number;
}
