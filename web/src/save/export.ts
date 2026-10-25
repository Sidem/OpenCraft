// Exporting worlds as gzipped `.ocworld` files: the Worlds panel's Export button (`ui/worlds.ts`) and the
// "save before closing" prompt (`ui/leave-prompt.ts`) share it. The open world is saved first, so the file is its
// newest state; other worlds export the save they were left in.

import { pack, type WorldMeta, type WorldStore } from './store';
import type { Session } from './session';

/** Downloads one world's newest save. Throws a message a player can read when there is nothing to export. */
export async function exportWorld(store: WorldStore, session: Session | null, w: WorldMeta): Promise<void> {
  if (w.id === session?.meta.id) await session.save();
  const meta = await store.get(w.id);
  const bytes = meta?.slot != null ? await store.read(meta.id, meta.slot) : null;
  if (!bytes) throw new Error(`"${w.name}" hasn't been saved yet, so there's nothing to export.`);
  const url = URL.createObjectURL(new Blob([(await pack(bytes)) as Uint8Array<ArrayBuffer>]));
  const a = document.createElement('a');
  a.href = url;
  a.download = `${w.name.replace(/[\\/:*?"<>|]/g, '_')}.ocworld`;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
}

/** Downloads every world that has a save, one file each (a browser may ask first to allow several downloads).
 * Returns how many were exported. */
export async function exportAll(store: WorldStore, session: Session | null): Promise<number> {
  let n = 0;
  for (const w of await store.list()) {
    if (w.slot === null && w.id !== session?.meta.id) continue;
    await exportWorld(store, session, w).catch(() => undefined);
    n++;
  }
  return n;
}
