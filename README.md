# OpenCraft

A browser game in the space between **Minecraft** and **Satisfactory**: a voxel world you mine and build in,
growing towards resource extraction, automation and factories.

The simulation core is **Rust compiled to WebAssembly** (with SIMD). Rendering is **raw WebGL2**. There is no
game engine and no runtime dependencies. The production bundle is about 55 KB gzipped.

## Quick start

Prerequisites:

- Rust (stable) with the `wasm32-unknown-unknown` target. `rust-toolchain.toml` installs the target automatically.
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) (`cargo install wasm-pack`)
- Node.js 20.19 or newer

```bash
npm install
npm run dev
```

Open <http://localhost:5173> and click **Click to play**. `npm run dev` builds the engine, starts Vite and
watches `crates/`. Any Rust change triggers a wasm rebuild and a page reload. TypeScript changes hot-reload
as usual.

| Command             | What it does                                                |
| ------------------- | ----------------------------------------------------------- |
| `npm run dev`       | Dev server with Rust watch + rebuild (release wasm)         |
| `npm run build`     | Production build into `dist/` (wasm + typecheck + bundle)   |
| `npm run preview`   | Serve the production build                                  |
| `npm test`          | Engine unit tests (`cargo test`)                            |
| `npm run typecheck` | TypeScript check                                            |

URL parameters: `?seed=1234` picks the world seed, and `?rd=12` sets the render distance in chunks (2 to 24,
default 8).

## Controls

| Input               | Action                                    |
| ------------------- | ----------------------------------------- |
| WASD                | Move                                      |
| Space               | Jump                                      |
| Shift               | Sprint                                    |
| C                   | Crouch (you won't walk off edges)         |
| Hold left mouse     | Mine the targeted block                   |
| Right mouse (hold)  | Place the selected block                  |
| 1–9 / mouse wheel   | Select hotbar slot                        |
| Q                   | Drop one item                             |
| F                   | Toggle fly mode (Space / C: up / down)    |
| F3                  | Debug stats                               |
| Esc                 | Release the mouse                         |

Mined blocks drop as items, which are pulled into your hotbar when you get close.

## Architecture

```
┌────────────────────── Rust → wasm (crates/engine) ───────────────────────┐
│ worldgen ─► chunk storage ─► greedy mesher ─► event queue (mesh/unload)  │
│ player physics · raycast · mining/placing · item entities · inventory    │
└───────────────▲──────────────────────────────────────┬───────────────────┘
      input, dt │                                      │ pointers into wasm memory
┌───────────────┴──── TypeScript platform (web/src) ───▼───────────────────┐
│ input.ts (pointer lock) · render/ (WebGL2) · ui/hud.ts (DOM)             │
└──────────────────────────────────────────────────────────────────────────┘
```

Rust owns all game state and every hot loop. TypeScript is a thin platform layer: it forwards input, uploads
data to the GPU, and draws the HUD.

Each frame:

1. `game.update(dt)` runs input, fixed-substep physics (120 Hz), targeting, mining and placing, and items.
2. `game.work_step()` is called in a loop until a time budget runs out (6 ms, or 28 ms while loading).
   Each step generates or meshes one chunk, nearest first.
3. Mesh and unload events are drained. Mesh data is read with a `Uint32Array` view directly on wasm memory
   and uploaded, with no copy and no serialization.
4. Chunks are frustum-culled and drawn front to back, opaque pass first, then cutout.

### Performance choices

- **32³ chunks, 256 blocks tall.** Uniform chunks (all air, all stone) take no heap memory.
- **Greedy meshing with per-vertex AO.** Faces merge only where AO is constant along the merge axis, so merging
  never changes the lighting. Quads are split along the brighter diagonal to avoid AO seams.
- **4 bytes per vertex.** Position, face, AO and texture layer are packed into one `u32`. UVs are derived in
  the shader from position, and texture wrapping tiles merged quads. One shared index buffer serves every chunk.
- **Texture array** (`TEXTURE_2D_ARRAY`) instead of an atlas. There is no bleeding, and mipmaps and
  anisotropic filtering work correctly.
- **Separate opaque and cutout passes.** Only leaves use `discard`, so opaque geometry keeps early-z.
- **Camera-relative rendering.** Chunk offsets are computed in float64 on the CPU, so precision holds at
  any distance from the origin.
- **Order-independent world generation.** Trees and ore veins crossing chunk borders are derived from
  hashes of their source cell. Chunks can therefore be generated in any order, which prepares for moving
  generation to worker threads.
- **Wasm SIMD128, fat LTO, `wasm-opt -O3`.**

Measured in Chrome with an RTX 3060: generating or meshing one chunk takes a median of about 0.1 ms, and
99% finish within about 1.1 ms. The full 8-chunk radius (about 2,150 chunks) streams in about 350 ms of CPU
time.

## Project layout

```
crates/engine/src/
  lib.rs        wasm-bindgen API (`Game`), frame update, mining/placing
  world.rs      chunk map, streaming, work queues, edits, render events
  worldgen.rs   terrain, cliffs, caves, ore veins, trees
  mesher.rs     greedy mesher + ambient occlusion
  chunk.rs      32³ block storage
  block.rs      block registry and lookup tables
  player.rs     character controller
  physics.rs    swept AABB vs voxel collision
  raycast.rs    voxel DDA for targeting
  entities.rs   dropped items (physics, magnet pickup, instancing)
  inventory.rs  hotbar
  textures.rs   procedural 16×16 block textures
  noise.rs      seeded Perlin noise + fBm
web/
  index.html, src/main.ts   bootstrap + frame loop
  src/render/               WebGL2 renderer, shaders, matrix helpers
  src/input.ts              keyboard/mouse, pointer lock
  src/ui/hud.ts             hotbar, target readout, pickup toasts, debug panel
scripts/                    wasm build + dev watcher
```

For debugging, the running game is exposed as `window.opencraft.game` in the devtools console. For example,
`opencraft.game.give(8, 64)` gives a stack of iron ore, and `opencraft.game.teleport(0, 120, 0)` moves you.

## Roadmap

Sandbox foundation:

- [ ] Save and load worlds (IndexedDB; edited chunks are already tracked)
- [ ] Worldgen in Web Workers
- [ ] Water and transparent blocks
- [ ] Full inventory screen
- [ ] Day/night cycle and sky

Factory layer (the Satisfactory half):

- [ ] Item registry separate from blocks (ingots, plates, parts) and recipes
- [ ] Resource nodes with purity, plus placeable miners
- [ ] Machines: smelter, constructor, assembler
- [ ] Conveyor belts, simulated in wasm as a lane/segment system rather than per-item entities
- [ ] Power grid: generators, poles, consumption
- [ ] Blueprint and hologram placement for multi-block buildings
