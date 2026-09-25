// World storage in IndexedDB. The `worlds` store holds a small record per world (name, seed, play time,
// which save slot is newest), so listing worlds never reads save data. The `saves` store holds the
// engine's save bytes keyed `[world id, slot]`. Each world alternates between two slots, so the
// previous save stays as a backup; bytes and record are written in one transaction, so a crash
// mid-write leaves the old pair intact. Bytes are gzipped when there's time (`pack`); saves made while
// the page closes go in raw, and `unpack` accepts both. To store more per world: a field on `WorldMeta`.

const DB_NAME = 'opencraft';
const DB_VERSION = 1;

export interface WorldMeta {
  id: string;
  name: string;
  seed: number;
  /** When the world was last saved or opened (ms since 1970); the latest one opens at startup. */
  updated: number;
  /** Seconds of game time played. */
  playTime: number;
  /** Slot (0 or 1) of the newest save, or null for a world never saved (it starts from its seed). */
  slot: number | null;
}

/** A new, never-saved world. */
export function newWorld(name: string, seed: number): WorldMeta {
  return { id: crypto.randomUUID(), name, seed, updated: Date.now(), playTime: 0, slot: null };
}

export class WorldStore {
  private constructor(private readonly db: IDBDatabase) {}

  static async open(): Promise<WorldStore> {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      req.result.createObjectStore('worlds', { keyPath: 'id' });
      req.result.createObjectStore('saves');
    };
    return new WorldStore(await done(req));
  }

  /** Every world, most recently played first. */
  async list(): Promise<WorldMeta[]> {
    const all: WorldMeta[] = await done(this.db.transaction('worlds').objectStore('worlds').getAll());
    return all.sort((a, b) => b.updated - a.updated);
  }

  async get(id: string): Promise<WorldMeta | undefined> {
    return done(this.db.transaction('worlds').objectStore('worlds').get(id));
  }

  /** The save bytes in one slot, unpacked, or null if the slot is empty. */
  async read(id: string, slot: number): Promise<Uint8Array | null> {
    const data: Uint8Array | undefined = await done(this.db.transaction('saves').objectStore('saves').get([id, slot]));
    return data ? unpack(data) : null;
  }

  /** Stores a world's record (new, or opened) without touching its saves. */
  put(meta: WorldMeta): Promise<void> {
    const tx = this.db.transaction('worlds', 'readwrite');
    tx.objectStore('worlds').put(meta);
    return finished(tx);
  }

  /** Stores `bytes` in slot `meta.slot` together with the record. Starts synchronously and commits at
   * once, so it has the best chance of finishing while the page closes. */
  write(meta: WorldMeta, bytes: Uint8Array): Promise<void> {
    const tx = this.db.transaction(['worlds', 'saves'], 'readwrite');
    tx.objectStore('saves').put(bytes, [meta.id, meta.slot ?? 0]);
    tx.objectStore('worlds').put(meta);
    tx.commit?.();
    return finished(tx);
  }

  delete(id: string): Promise<void> {
    const tx = this.db.transaction(['worlds', 'saves'], 'readwrite');
    tx.objectStore('worlds').delete(id);
    tx.objectStore('saves').delete(IDBKeyRange.bound([id, 0], [id, 1]));
    return finished(tx);
  }
}

/** Gzips save bytes with the browser's built-in compressor (no wasm or library cost). */
export async function pack(bytes: Uint8Array): Promise<Uint8Array> {
  return pipe(bytes, new CompressionStream('gzip'));
}

/** Save bytes from stored or imported data, gzipped or not. Throws if the gzip data is broken. */
export async function unpack(data: Uint8Array): Promise<Uint8Array> {
  return data[0] === 0x1f && data[1] === 0x8b ? pipe(data, new DecompressionStream('gzip')) : data;
}

async function pipe(bytes: Uint8Array, stream: CompressionStream | DecompressionStream): Promise<Uint8Array> {
  const out = new Blob([bytes as Uint8Array<ArrayBuffer>]).stream().pipeThrough(stream);
  return new Uint8Array(await new Response(out).arrayBuffer());
}

function done<T>(req: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

function finished(tx: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onerror = tx.onabort = () => reject(tx.error ?? new Error('The browser refused to save.'));
  });
}
