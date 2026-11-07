// Hands the engine's finished chunk meshes to the renderer (`game.next_event()`: 1 a mesh, 2 an unloaded chunk). Each
// mesh is two views into wasm memory, its cubes and its shaped list (shaped.ts), uploaded at once and dropped.

import type { Game } from '../wasm/engine.js';
import type { Renderer } from './renderer';

/** Uploads every mesh the engine has finished and drops every chunk it unloaded. */
export function drainMeshes(game: Game, memory: WebAssembly.Memory, renderer: Renderer): void {
  for (;;) {
    const kind = game.next_event();
    if (kind === 0) return;
    const x = game.event_x(), y = game.event_y(), z = game.event_z();
    if (kind !== 1) {
      renderer.removeChunk(x, y, z);
      continue;
    }
    const verts = new Uint32Array(memory.buffer, game.mesh_ptr(), game.mesh_len());
    const shaped = new Uint32Array(memory.buffer, game.mesh_shaped_ptr(), game.mesh_shaped_len());
    renderer.upsertChunk(x, y, z, verts, game.mesh_opaque_quads(), game.mesh_cutout_quads(), game.mesh_liquid_quads(),
      shaped, game.mesh_shaped_opaque(), game.mesh_shaped_cutout());
  }
}
