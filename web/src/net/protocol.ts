// Co-op messages between a host and a client: one type byte, then the payload (little-endian numbers,
// strings as a u16 byte length plus UTF-8). The engine's own bytes (snapshot, actions, frames) pass
// through untouched. `decode` returns null for anything malformed rather than throwing.
//
//   hello    client → host   build id, player key, name
//   welcome  host → client   player id, snapshot (`game.snapshot()`)
//   refuse   host → client   a reason a player can read; the host closes next
//   actions  client → host   `game.take_outbox()`
//   frames   host → client   `game.take_frames()`
//   checksum host → client   tick, state hash (every 60 ticks: `game.take_checksums()`)
//   bye      either way      leaving on purpose
//   state    client → host   its body (`game.take_states()`, 20 a second)
//   states   host → client   every body (`game.take_states()`)
//   items    host → client   the loose items near that client (`game.take_item_view(id)`, 10 a second)
//   names    host → client   (player id, name) of everyone, sent when someone joins or leaves
//   ping     host → client   the host's clock (ms), the host's player id, everyone's round trip (ms)
//   pong     client → host   the clock from that ping, straight back
//   resync   client → host   asks for a fresh snapshot: the cores went apart, or it fell far behind
//   snapshot host → client   the answer (`game.snapshot()`, taken between frames like a welcome's)
// To add a message: a type number (append), a `Message` case, and its arm in `encode` and `decode`.

export type Message =
  | { type: 'hello'; build: string; key: bigint; name: string }
  | { type: 'welcome'; player: number; snapshot: Uint8Array }
  | { type: 'refuse'; reason: string }
  | { type: 'actions'; bytes: Uint8Array }
  | { type: 'frames'; bytes: Uint8Array }
  | { type: 'checksum'; tick: bigint; hash: bigint }
  | { type: 'bye' }
  | { type: 'state'; bytes: Uint8Array }
  | { type: 'states'; bytes: Uint8Array }
  | { type: 'items'; bytes: Uint8Array }
  | { type: 'names'; names: [number, string][] }
  | { type: 'ping'; time: number; host: number; pings: [number, number][] }
  | { type: 'pong'; time: number }
  | { type: 'resync' }
  | { type: 'snapshot'; bytes: Uint8Array };

/** Which build this page runs. Peers must match: the lockstep needs the same engine everywhere. In a
 * production build this module's URL carries the bundle's content hash, which changes with any script
 * or the wasm; in development every tab serves the same files, so the URL is the same too. */
export const BUILD_ID = new URL(import.meta.url).pathname;

const TYPES = [
  'hello', 'welcome', 'refuse', 'actions', 'frames', 'checksum', 'bye', 'state', 'states', 'items', 'names',
  'ping', 'pong', 'resync', 'snapshot',
] as const;
const utf8 = new TextEncoder();
const text = new TextDecoder('utf-8', { fatal: true });

export function encode(m: Message): Uint8Array {
  const w = new Writer();
  w.u8(TYPES.indexOf(m.type) + 1);
  switch (m.type) {
    case 'hello':
      w.str(m.build);
      w.u64(m.key);
      w.str(m.name);
      break;
    case 'welcome':
      w.u8(m.player);
      w.bytes(m.snapshot);
      break;
    case 'refuse':
      w.str(m.reason);
      break;
    case 'actions':
    case 'frames':
    case 'state':
    case 'states':
    case 'items':
    case 'snapshot':
      w.bytes(m.bytes);
      break;
    case 'names':
      w.u8(m.names.length);
      for (const [id, name] of m.names) {
        w.u8(id);
        w.str(name);
      }
      break;
    case 'checksum':
      w.u64(m.tick);
      w.u64(m.hash);
      break;
    case 'ping':
      w.f64(m.time);
      w.u8(m.host);
      w.u8(m.pings.length);
      for (const [id, ms] of m.pings) {
        w.u8(id);
        w.u16(ms);
      }
      break;
    case 'pong':
      w.f64(m.time);
      break;
    case 'bye':
    case 'resync':
      break;
  }
  return w.finish();
}

export function decode(b: Uint8Array): Message | null {
  const r = new Reader(b);
  try {
    const type = TYPES[r.u8() - 1];
    let m: Message;
    switch (type) {
      case 'hello':
        m = { type, build: r.str(), key: r.u64(), name: r.str() };
        break;
      case 'welcome':
        m = { type, player: r.u8(), snapshot: r.rest() };
        break;
      case 'refuse':
        m = { type, reason: r.str() };
        break;
      case 'actions':
      case 'frames':
      case 'state':
      case 'states':
      case 'items':
      case 'snapshot':
        m = { type, bytes: r.rest() };
        break;
      case 'names': {
        const names: [number, string][] = [];
        for (let n = r.u8(); n > 0; n--) names.push([r.u8(), r.str()]);
        m = { type, names };
        break;
      }
      case 'checksum':
        m = { type, tick: r.u64(), hash: r.u64() };
        break;
      case 'ping': {
        const time = r.f64(), host = r.u8();
        const pings: [number, number][] = [];
        for (let n = r.u8(); n > 0; n--) pings.push([r.u8(), r.u16()]);
        m = { type, time, host, pings };
        break;
      }
      case 'pong':
        m = { type, time: r.f64() };
        break;
      case 'bye':
      case 'resync':
        m = { type };
        break;
      default:
        return null;
    }
    return r.done() ? m : null;
  } catch {
    return null; // Past the end, or text that isn't UTF-8.
  }
}

class Writer {
  private buf = new Uint8Array(64);
  private len = 0;

  u8(v: number): void {
    this.room(1)[0] = v;
  }

  u16(v: number): void {
    const r = this.room(2);
    new DataView(r.buffer, r.byteOffset, 2).setUint16(0, Math.min(v, 0xffff), true);
  }

  f64(v: number): void {
    const r = this.room(8);
    new DataView(r.buffer, r.byteOffset, 8).setFloat64(0, v, true);
  }

  u64(v: bigint): void {
    const r = this.room(8);
    new DataView(r.buffer, r.byteOffset, 8).setBigUint64(0, v, true);
  }

  str(s: string): void {
    const b = utf8.encode(s.slice(0, 1000)); // names and reasons are short; this keeps the length in a u16
    const r = this.room(2);
    new DataView(r.buffer, r.byteOffset, 2).setUint16(0, b.length, true);
    this.bytes(b);
  }

  bytes(b: Uint8Array): void {
    this.room(b.length).set(b);
  }

  finish(): Uint8Array {
    return this.buf.slice(0, this.len);
  }

  /** The next `n` bytes to fill, growing the buffer as needed. */
  private room(n: number): Uint8Array {
    if (this.len + n > this.buf.length) {
      const bigger = new Uint8Array(Math.max(this.buf.length * 2, this.len + n));
      bigger.set(this.buf.subarray(0, this.len));
      this.buf = bigger;
    }
    this.len += n;
    return this.buf.subarray(this.len - n, this.len);
  }
}

class Reader {
  private pos = 0;
  private readonly view: DataView;

  constructor(private readonly b: Uint8Array) {
    this.view = new DataView(b.buffer, b.byteOffset, b.byteLength);
  }

  u8(): number {
    return this.view.getUint8(this.take(1));
  }

  u16(): number {
    return this.view.getUint16(this.take(2), true);
  }

  f64(): number {
    return this.view.getFloat64(this.take(8), true);
  }

  u64(): bigint {
    return this.view.getBigUint64(this.take(8), true);
  }

  str(): string {
    const n = this.view.getUint16(this.take(2), true);
    return text.decode(this.b.subarray(this.take(n), this.pos));
  }

  rest(): Uint8Array {
    return this.b.slice(this.take(this.b.length - this.pos));
  }

  done(): boolean {
    return this.pos === this.b.length;
  }

  /** Moves past `n` bytes and returns where they start; throws past the end. */
  private take(n: number): number {
    if (this.pos + n > this.b.length) throw new RangeError('short message');
    this.pos += n;
    return this.pos - n;
  }
}
