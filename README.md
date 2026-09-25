# OpenCraft

A browser game in the space between **Minecraft** and **Satisfactory**: a voxel world you mine and build in,
growing towards resource extraction, automation and factories.

**Play it: <https://sidem.github.io/OpenCraft/>** (desktop browser with keyboard and mouse).

The simulation core is **Rust compiled to WebAssembly** (with SIMD). Rendering is **raw WebGL2**. There is no
game engine and no runtime dependencies. Every texture and sound effect is generated procedurally at startup,
so there are no asset files, and the whole production build is about 115 KB gzipped.

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
| `npm run check`     | Pre-commit checks: format, lints, tests, types, sizes       |

URL parameters: `?seed=1234` starts a new world with that seed (saved like any other), and `?rd=12` sets
the render distance in chunks (2 to 24, default 8).

## Controls

| Input               | Action                                    |
| ------------------- | ----------------------------------------- |
| WASD                | Move                                      |
| Space               | Jump                                      |
| Shift               | Sprint                                    |
| C                   | Crouch (you won't walk off edges)         |
| Hold left mouse     | Mine the targeted block                   |
| Right mouse (hold)  | Place the selected block. Belts run the way you face |
| Right mouse on a box | Open it like a chest: click moves stacks with the cursor, Shift-click moves a whole stack between box and inventory. Hold C to place against it instead |
| Right mouse on a miner | Take everything it holds |
| Right mouse on a smelter, constructor or filter | Open its panel. Hold C to place against it instead |
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

## Saving and worlds

Your world saves itself: every minute, whenever you pause (Esc), and when you switch tabs or close the page.
Opening the game again brings back the world you played last. Press **Continue** and everything is where
you left it, machines still running.

The **Worlds** list in the pause menu manages several worlds:

- **New world** takes a name and an optional seed. A number gives that exact world, any other text works
  as a seed too, and leaving it blank picks one at random.
- **Play** switches to another world (the current one is saved first).
- **Export** downloads a world as an `.ocworld` file, and **Import** adds one, so you can keep a backup or
  move a world to another browser.
- **Delete** removes a world for good, after asking.

Worlds are stored in this browser only. Clearing the site's data deletes them, so export any world you
want to keep. The game also keeps each world's previous save as a backup and uses it automatically if the
latest save can't be read.

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
- **Climbing and crossing.** A ramp up hands items on one block ahead and one up; a ramp down takes them
  from a belt one block up behind it. Stacked lifts carry items straight up, and the top one hands them on
  one ahead and one up. An underpass entry sends items under whatever is in front of it to the nearest
  exit facing the same way, up to 5 blocks ahead, so two lines can cross.
- **Storage boxes** accept items from belts that end in them. They push items into belts that lead away
  from them, and miners next to a box fill it directly. Right-click a box to open it like a chest (Take all empties it); right-click a miner to take its ore.
- **A smelter** melts iron or copper ore into ingots, one every 1.5 seconds while its fire burns. Feed it ore
  and fuel (coal ore burns 8 seconds, a log 4) by belt or from a miner next to it; it sorts them itself.
  It burns fuel only while smelting, and pushes ingots into a belt leading away. Its lamp shows green when
  working, red when out of fuel and yellow when its output is full.
- **A constructor** shapes ingots into parts: iron plates (2 ingots each), iron rods, screws (4 from a rod)
  and copper wire (2 from a copper ingot). Right-click it to choose what it makes; changing the choice
  gives back the ingots it held. Belts bring the input and a belt leading away takes the parts. Parts build
  splitters and filters, and the machines still to come (better miners).
- **Splitters and filters** sit in a belt line. A splitter shares items between the belts leading away in
  front, to the left and to the right, skipping any that are full. A filter sends the item you choose in
  its panel straight on and everything else to the sides.
- **Power.** Constructors, splitters and filters need power; miners and smelters don't. A coal generator burns
  coal ore or logs (by belt or by hand) into 60 kW, only while its grid needs it. Power poles link to every
  pole within 10 blocks, and each generator and machine hangs on the nearest pole within 5 (you see the
  wires). A working constructor draws 15 kW, a splitter or filter 1 kW. Short of power, every machine on the
  grid slows down to match; with none, it stops.
- **Machine panels.** Right-click a smelter, constructor, filter or generator to see what it's doing, put items in straight
  from your inventory (ore, fuel, ingots), and take what it made.

Craft machines in the build menu (E): a Miner Mk1 costs 10 iron ore, 6 copper ore and 12 stone. Four belts
cost 1 iron ore and 2 stone, a box costs 6 logs and 2 iron ore, a smelter 16 stone and 4 iron ore, a constructor 10 iron ingots,
4 copper ingots and 8 stone, a splitter 2 iron plates and 2 belts, a filter 2 iron plates, 2 copper
wire and 2 belts. Two ramps (up or down) cost an iron plate and 2 belts, two lifts 2 iron rods and 2
belts, and an underpass entry or exit 2 iron plates and 2 belts. A coal generator costs 12 iron ingots, 8
copper ingots and 10 stone; two power poles an iron ingot, a copper ingot and a log. Hand-mining an outcrop or two covers
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
┌──────────────────────── Rust → wasm (crates/engine) ─────────────────────────┐
│ Game (lib.rs, api/): what the host calls; acts for the local player          │
│  ├─ core: Sim        deterministic: tick, world edits, factory, deposits,    │
│  │                   inventories, rng. Changes only by applying Actions at   │
│  │                   a tick, and reports back with SimEvents                 │
│  ├─ authority        every player's body (physics), loose items, pickups     │
│  └─ view (local)     camera, targeting, mining, streaming, greedy meshing,   │
│                      box instances, sounds, toasts                           │
│ save.rs: the core's bytes (also its state hash) + bodies + items ⇄ save file │
└───────────────▲──────────────────────────────────────────────┬───────────────┘
      input, dt │                                              │ pointers into wasm memory
┌───────────────┴───────── TypeScript platform (web/src) ──────▼───────────────┐
│ input.ts · render/ (WebGL2) · ui/ (DOM) · audio/ · save/ (IndexedDB)         │
└──────────────────────────────────────────────────────────────────────────────┘
```

Rust owns all game state and every hot loop. TypeScript is a thin platform layer: it forwards input, uploads
data to the GPU, draws the HUD and stores saves.

Inside the engine, the core only changes when a queued action is applied at a tick, so the same actions
always produce the same world. Tests compare state hashes tick by tick, and a world reloaded from a save
carries on identically. This is the groundwork for co-op, where every player's machine will run the core and
the host will be the authority. The view is local presentation and never feeds back into the core.

Each frame:

1. `game.update(dt)` runs the simulation in fixed 60 Hz ticks (physics at 120 Hz, targeting, mining and
   placing, items, the factory), so it behaves the same at any frame rate. The camera is interpolated
   between the last two ticks. It then writes one box instance (12 floats) per dropped item, belt item and
   machine part.
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

`crates/engine` is the Rust engine, which owns all game state. `web/` is the TypeScript host (rendering, input,
UI, audio), and `scripts/` holds the build, dev and check scripts. [docs/CODEMAP.md](docs/CODEMAP.md) lists
every module and explains how to add blocks, recipes, machines, API methods, UI panels and sound materials.

## Deployment

Every push to `main` runs `.github/workflows/pages.yml`. It builds the wasm, runs `npm run check`, builds the Vite
bundle, and publishes `dist/` to GitHub Pages. The build uses relative asset URLs (`base: './'`), so it
works from any sub-path.

For debugging, the running game is exposed as `window.opencraft.game` in the devtools console. For example:

- `opencraft.game.give(8, 64)` gives a stack of iron ore. Block ids: 7 coal ore, 8 iron ore, 9 copper ore,
  12 belt, 13 miner, 14 box, 15 smelter, 16 constructor, 17 splitter, 18 filter, 19 ramp up, 20 ramp down, 21 lift, 22 underpass entry,
  23 underpass exit, 24 generator, 25 power pole; items: 256 iron ingot, 257 copper ingot, 258 iron
  plate, 259 iron rod, 260 screws, 261 copper wire.
- `opencraft.game.teleport(0, 120, 0)` moves you.
- `opencraft.game.find_deposit(1)` returns `[x, y, z, ore]` for the nearest deposit of a tier
  (0 lode, 1 vein, 2 outcrop).
- `opencraft.game.skip_time(600)` runs the game ten minutes ahead, silently (dropped items older than five
  minutes despawn).
- `opencraft.game.block_at(x, y, z)` reads a block.

## Roadmap

The plan of record, with decisions, rules and the current milestone's steps, is
[docs/DEV_PLAN.md](docs/DEV_PLAN.md). Later milestones are in [docs/ROADMAP.md](docs/ROADMAP.md). In short:

Sandbox foundation:

- [x] Fixed simulation tick, deterministic core driven by actions (groundwork for co-op)
- [x] Save and load worlds: autosave, several worlds, export and import
- [ ] Co-op multiplayer: player-hosted over WebRTC
- [ ] Simulation and worldgen in Web Workers
- [ ] Water and transparent blocks
- [x] Full inventory screen
- [ ] Day/night cycle, sky and voxel lighting
- [ ] Biomes that shape resources, and a minimap

Factory layer (the Satisfactory half):

- [x] Ore deposits (outcrop, vein, lode) with shared pools, draw limits and depletion
- [x] Miner Mk1, conveyor belts (merging, corners) and storage boxes, crafted by hand
- [x] Item registry separate from blocks (ingots so far) and machine recipes
- [x] Smelter: ore and fuel into ingots
- [x] Constructor and parts (plates, rods, screws, wire), with a machine panel
- [ ] Machines: assembler
- [x] Splitters and filters
- [x] Belts that climb (ramps, lifts) and cross (underpasses)
- [ ] Miner and belt tiers
- [ ] Prospecting: find veins and lodes without digging blind
- [x] Power grid: coal generators, poles, consumption and brownouts
- [ ] Research, and an optional endgame megaproject that doesn't end the game
- [ ] Terraforming machines: excavators, graders, tunnel borers
- [ ] Blueprints and construction drones
- [ ] Trucks and trains
