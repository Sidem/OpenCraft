# OpenCraft

A browser game in the space between **Minecraft** and **Satisfactory**: a voxel world you mine and build in,
growing towards resource extraction, automation and factories.

**Play it: <https://sidem.github.io/OpenCraft/>** (desktop browser with keyboard and mouse).

The simulation core is **Rust compiled to WebAssembly** (with SIMD). Rendering is **raw WebGL2**. There is no
game engine and no runtime dependencies. Every texture and sound effect is generated procedurally at startup,
so there are no asset files, and the whole production build is about 100 KB gzipped.

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
| Right mouse (hold)  | Place the selected block. Belts run the way you face |
| Right mouse on a box or miner | Take everything it holds. Hold C to place against it instead |
| E                   | Inventory and build menu                  |
| 1–9 / mouse wheel   | Select hotbar slot                        |
| Q                   | Drop one item                             |
| F                   | Toggle fly mode (Space / C: up / down)    |
| M                   | Mute / unmute sound                       |
| O                   | Open the sound designer                   |
| F3                  | Debug stats                               |
| Esc                 | Release the mouse                         |

Mined blocks drop as items, which are pulled into your inventory when you get close. In the inventory screen
(E), click a slot to pick up a stack and click again to put it down. Shift-click moves a stack between the
hotbar and the backpack.

## Resources and extraction

Ore does not sit in single blocks. It comes in **deposits**, and every ore block belongs to exactly one deposit.
A deposit holds a pool of ore units shared by all of its blocks. Look at an ore block to see its deposit's
size.

| Tier    | Where                                        | Units per block | Shared draw limit |
| ------- | -------------------------------------------- | --------------- | ----------------- |
| Outcrop | small clusters, half of them at the surface  | 100             | 60 per minute     |
| Vein    | long seams 20–60 blocks below the surface    | 1,000           | 240 per minute    |
| Lode    | rare, huge bodies deep down (y 12–25)        | 2,000           | 1,200 per minute  |

- **By hand** you keep 3 ore per block. The rest of that block's share of the pool is lost.
- **A miner** placed against any block of a deposit draws from the whole pool and delivers 60% of what it
  drills. All miners on one deposit share its draw limit. Output slows over the last 20% of the pool.
  Each time a block's worth is used up, the ore block nearest the miner turns into spent rock, even in
  chunks that aren't loaded. A worked-out deposit is gone for good.
- **Conveyor belts** carry items one block per second and run the way you were facing when you placed
  them. A belt ending in another belt's side merges into it. A belt fed only from one side turns the corner.
- **Storage boxes** accept items from belts that end in them. They push items into belts that lead away
  from them, and miners next to a box fill it directly. Right-click a box or miner to take its contents.

Craft machines in the build menu (E): a Miner Mk1 costs 10 iron ore, 6 copper ore and 12 stone. Four belts
cost 1 iron ore and 2 stone, and a box costs 6 logs and 2 iron ore. Hand-mining an outcrop or two covers
your first miner. After that, let it do the work.

## Sound designer

Press **O** in game (or **Sound designer** in the pause menu) to tune every sound with dials. You don't need
any sound-design experience to use it.

- **Materials** (Stone, Dirt, Grass, Sand, Wood, Leaves, Metal): each has ten dials with plain meanings, such as
  Pitch (deep ↔ high), Tone (muffled ↔ bright), Crunch (smooth ↔ gritty), Scatter (one hit ↔ many bits),
  Snap (soft ↔ sharp), Thud, Ring, Length, Volume and Variation. Hover over a dial for an explanation.
- **Listen** buttons play mining, breaking, placing, footsteps and landing on the material. Walk and mining
  loops keep playing while you turn dials. Pressing O while looking at a block opens that block's material.
- **Actions** adjusts footsteps, landing, pickup and so on for every material at once.
- Changes apply in the game immediately and are saved in the browser.
- **Copy settings** exports the whole design as JSON. To make a design the built-in default, paste it over
  DEFAULT_DESIGN in web/src/audio/settings.ts, since it uses the same shape.

## Architecture

```
┌────────────────────── Rust → wasm (crates/engine) ───────────────────────┐
│ worldgen ─► chunk storage ─► greedy mesher ─► event queue (mesh/unload)  │
│ player physics · raycast · mining/placing · item entities · inventory    │
│ ore deposits · factory (miners, belts, boxes) · crafting                 │
└───────────────▲──────────────────────────────────────┬───────────────────┘
      input, dt │                                      │ pointers into wasm memory
┌───────────────┴──── TypeScript platform (web/src) ───▼───────────────────┐
│ input.ts (pointer lock) · render/ (WebGL2) · ui/hud.ts (DOM)             │
└──────────────────────────────────────────────────────────────────────────┘
```

Rust owns all game state and every hot loop. TypeScript is a thin platform layer: it forwards input, uploads
data to the GPU, and draws the HUD.

Each frame:

1. `game.update(dt)` runs input, fixed-substep physics (120 Hz), targeting, mining and placing, items and
   the factory. It then writes one box instance (12 floats) per dropped item, belt item and machine part.
2. `game.work_step()` is called in a loop until a time budget runs out (6 ms, or 28 ms while loading).
   Each step generates or meshes one chunk, nearest first.
3. Mesh and unload events are drained. Mesh data is read with a `Uint32Array` view directly on wasm memory
   and uploaded, with no copy and no serialization. Sound events (dig, break, place, footstep, landing,
   pickup, drop) are read the same way. Each carries a sound material and a camera-relative position.
4. Chunks are frustum-culled and drawn front to back, opaque pass first, then cutout. All box instances go
   out in a single instanced draw call, read from wasm memory like the meshes.

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
- **Order-independent world generation.** Trees and ore deposits crossing chunk borders are derived from
  hashes of their source cell. Chunks can therefore be generated in any order, which prepares for moving
  generation to worker threads. Where deposits overlap, a fixed priority decides which one owns each block.
  The same rule answers "which deposit is this block in?", so ownership needs no stored map.
- **Factory state is small and lazy.** Belts hold a short list of items with their progress. Links between
  machines are rebuilt only when something is placed or removed. A deposit's pool is tracked only once
  something mines, drills or inspects it, and its draw budget is refilled on demand, not every tick.
- **Wasm SIMD128, fat LTO, `wasm-opt -O3`.**
- **Procedural audio.** Each material sound is layered from noise bursts (grainy or smooth, single or scattered),
  a tone filter, a low thud and tuned resonators. The designer's dials map directly onto these layers.
  Buffers are synthesized in 4 ms background slices and rebuilt only when a dial affecting them changes.
  Fixed seeds keep a sound's randomness stable while you tune it. Playback adds pitch jitter, distance
  falloff and stereo panning, and a compressor sits on the master bus.

Measured in Chrome with an RTX 3060: generating or meshing one chunk takes a median of about 0.1 ms, and
99% finish within about 1.1 ms. The full 8-chunk radius (about 2,150 chunks) streams in about 350 ms of CPU
time.

## Project layout

```
crates/engine/src/
  lib.rs        wasm-bindgen API (`Game`), frame update, mining/placing
  world.rs      chunk map, streaming, work queues, edits, render events
  worldgen.rs   terrain, cliffs, caves, ore deposit placement, trees
  deposits.rs   ore deposits: tiers, shapes, shared pools, draw limits, depletion
  factory.rs    miners, conveyor belts, storage boxes; their simulation and models
  recipes.rs    hand-crafting recipes
  mesher.rs     greedy mesher + ambient occlusion
  chunk.rs      32³ block storage
  block.rs      block registry and lookup tables
  player.rs     character controller
  physics.rs    swept AABB vs voxel collision
  raycast.rs    voxel DDA for targeting
  entities.rs   dropped items (physics, magnet pickup, instancing)
  inventory.rs  hotbar, backpack and the stack held on the cursor
  sound.rs      gameplay sound events for the host
  textures.rs   procedural 16×16 block textures
  noise.rs      seeded Perlin noise + fBm
web/
  index.html, src/main.ts   bootstrap + frame loop
  src/render/               WebGL2 renderer, shaders, matrix helpers
  src/input.ts              keyboard/mouse, pointer lock
  src/audio/settings.ts     sound design: dials, defaults, presets, save/load, copy/paste format
  src/audio/synth.ts        procedural foley synthesis (dials -> samples)
  src/audio/sound.ts        sound events -> Web Audio voices, buffer cache, previews
  src/ui/hud.ts             hotbar, target readout, pickup toasts, debug panel
  src/ui/inventory.ts       inventory and build menu (E)
  src/ui/sound-lab.ts       sound designer panel + volume/mute control
  src/ui/knob.ts            rotary dial widget
scripts/                    wasm build + dev watcher
.github/workflows/pages.yml CI: test, build, deploy to GitHub Pages
```

## Deployment

Every push to `main` runs `.github/workflows/pages.yml`. It runs the engine tests, builds the wasm and the Vite
bundle, and publishes `dist/` to GitHub Pages. The build uses relative asset URLs (`base: './'`), so it
works from any sub-path.

For debugging, the running game is exposed as `window.opencraft.game` in the devtools console. For example:

- `opencraft.game.give(8, 64)` gives a stack of iron ore. Block ids: 7 coal ore, 8 iron ore, 9 copper ore,
  12 belt, 13 miner, 14 box.
- `opencraft.game.teleport(0, 120, 0)` moves you.
- `opencraft.game.find_deposit(1)` returns `[x, y, z, ore]` for the nearest deposit of a tier
  (0 lode, 1 vein, 2 outcrop).
- `opencraft.game.skip_time(600)` runs the factory ten minutes ahead.
- `opencraft.game.block_at(x, y, z)` reads a block.

## Roadmap

The plan of record, with decisions, rules and the current milestone's steps, is
[docs/DEV_PLAN.md](docs/DEV_PLAN.md). Later milestones are in [docs/ROADMAP.md](docs/ROADMAP.md). In short:

Sandbox foundation:

- [ ] Fixed simulation tick, deterministic core driven by actions (groundwork for co-op)
- [ ] Save and load worlds (IndexedDB; edited chunks are already tracked)
- [ ] Co-op multiplayer: player-hosted over WebRTC
- [ ] Simulation and worldgen in Web Workers
- [ ] Water and transparent blocks
- [x] Full inventory screen
- [ ] Day/night cycle, sky and voxel lighting
- [ ] Biomes that shape resources, and a minimap

Factory layer (the Satisfactory half):

- [x] Ore deposits (outcrop, vein, lode) with shared pools, draw limits and depletion
- [x] Miner Mk1, conveyor belts (merging, corners) and storage boxes, crafted by hand
- [ ] Item registry separate from blocks (ingots, plates, parts) and machine recipes
- [ ] Machines: smelter, constructor, assembler
- [ ] Miner and belt tiers, splitters, belts that climb
- [ ] Prospecting: find veins and lodes without digging blind
- [ ] Power grid: generators, poles, consumption
- [ ] Research, and an optional endgame megaproject that doesn't end the game
- [ ] Terraforming machines: excavators, graders, tunnel borers
- [ ] Blueprints and construction drones
- [ ] Trucks and trains
