# OpenCraft

A browser game in the space between **Minecraft** and **Satisfactory**: a voxel world you mine and build in,
growing towards resource extraction, automation and factories.

**Play it: <https://sidem.github.io/OpenCraft/>** (desktop browser with keyboard and mouse).

The simulation core is **Rust compiled to WebAssembly** (with SIMD). Rendering is **raw WebGL2**. There is no
game engine and no runtime dependencies. Every texture and sound effect is generated procedurally at startup,
so there are no asset files, and the whole production build is about 250 KB gzipped.

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
| Space               | Jump; in water, hold to swim up (at a bank, it climbs you out) |
| Shift               | Sprint                                    |
| C                   | Crouch (you won't walk off edges)         |
| Hold left mouse     | Mine the targeted block                   |
| Right mouse (hold)  | Place the selected block                  |
| Right mouse drag with belts | Lay a line of belts: press on the ground, drag to where it should end (it turns once and climbs or drops one-block steps by itself), release to build. Left-click cancels |
| R                   | Turn the belt or splitter/filter you point at; while holding a quarry, turn its dig box |
| Right mouse on a box | Open it like a chest: click moves stacks with the cursor, Shift-click moves a whole stack between box and inventory. Hold C to place against it instead |
| Right mouse on a miner | Take everything it holds |
| Right mouse on a smelter, constructor, filter, generator, lab or quarry | Open its panel. Hold C to place against it instead |
| E                   | Inventory and build menu                  |
| T                   | Research: choose what labs work on        |
| H                   | Skip the tip on screen (the menu can show tips again) |
| N                   | Show or hide the minimap (top right, north up; rings mark veins and lodes you've prospected, squares your machines) |
| 1–9 / mouse wheel   | Select hotbar slot                        |
| Q                   | Drop one item                             |
| F                   | Toggle fly mode (Space / C: up / down)    |
| Hold Tab            | Who is playing with you (in a shared game) |
| M                   | Mute / unmute sound                       |
| O                   | Open the sound designer                   |
| F3                  | Debug stats                               |
| Esc                 | Release the mouse; in a panel, close it   |

Mined blocks drop as items, which are pulled into your inventory when you get close. In the inventory screen
(E), click a slot to pick up a stack and click again to put it down. Shift-click moves a stack between the
hotbar and the backpack.

New worlds have a sea and ponds. Water flows: dig beside the sea and it pours in and refills any hole or
trench at or below its level, however far. Elsewhere water runs up to 7 blocks from its source, getting
shallower, and falls over edges; wall it off and the flow dries up. Blocks and machines can be placed
straight into water. In water you swim (Space rises), and dropped items float.

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

New worlds have **biomes**: plains and forest around where you start, sandy deserts over sandstone,
highlands over granite, wet lowlands thick with trees, and bare black basalt fields. The rock under
each biome decides which ores lie in it: coal and limestone in the lowlands, copper and quartz in the
highlands, quartz and limestone in the deserts, rich iron in the basalt, a bit of everything on the plains.
Deep veins and lodes give themselves away at the surface: rusty soil above iron, dark soil above coal,
blue-green soil above copper and pale soil above limestone and quartz. Ore rarely reaches the surface:
look for it where the rock is bare, on cliffs, bare mountain tops, deserts and basalt fields. Each ore keeps to
its own depth: limestone and coal lie shallow, iron deeper, copper and quartz deepest. Every new world has
a small patch of coal, iron and copper showing 40 to 80 blocks from where you start. Worlds made before
this keep the terrain and ore they were made with (the oldest ones, before biomes, are "classic terrain"),
so nothing you built there changes.

A day lasts 20 minutes, from sunrise through a golden dusk to a dark blue night with stars; the pause menu
shows the day and the time. Light is simulated per block: forest floors lie in shade, caves are dark even
at noon, and a shaft you dig lets the sky in. **Lamps** (glass, an iron plate and 2 copper wire make two)
light about 14 blocks around them in a warm glow, day and night.

Worlds are stored in this browser only. Clearing the site's data deletes them, so export any world you
want to keep. The game also keeps each world's previous save as a backup and uses it automatically if the
latest save can't be read.

## Playing together

Up to four players can build in one world. **Play together** in the pause menu:

- Type your name; others see it above your head.
- **Host this world** opens the world you're playing to others and shows a code and a link. Send the link
  (or the code) to your friends. Your browser runs the world and keeps it saved, so it stays open only
  while your tab does.
- To join, paste a friend's link or code into the box and press **Join**. Your own world is saved first.
  Friends who come back later get their things back where they left them.
- Hold **Tab** in game to see who is playing and their ping. **Leave** (or closing the tab) takes you out;
  when the host leaves, the game ends for everyone and **Back to my worlds** returns to yours.

Players connect directly (WebRTC). A small Cloudflare Worker (`signal/`) only introduces them; no game data
passes through it. Some networks (strict company or mobile networks) block direct connections, and the
game then says it couldn't connect.

## Resources and extraction

Ore does not sit in single blocks. It comes in **deposits**, and every ore block belongs to exactly one deposit.
A deposit holds a pool of ore units shared by all of its blocks. Look at an ore block to see its deposit's
size.

| Tier    | Where                                        | Units per block | Shared draw limit |
| ------- | -------------------------------------------- | --------------- | ----------------- |
| Outcrop | small clusters; few show, most are buried    | 100             | 60 per minute     |
| Vein    | long seams 8–70 blocks down, by ore          | 1,000           | 240 per minute    |
| Lode    | rare, huge bodies deep down (y 12–25)        | 2,000           | 1,200 per minute  |

- **By hand** you keep 3 ore per block (4 with an iron pickaxe). The rest of that block's share of the pool
  is lost.
- **Tools.** Hold a pickaxe (stone and ore), an axe (wood) or a shovel (dirt, grass, sand) in the hotbar to
  break those blocks faster: stone tools twice as fast, iron tools four times. Each block broken with the
  right tool uses it up a little (stone tools last 150 blocks, iron 600); the bar under its slot shows
  what's left. Stone tools cost 3 stone (1 for a shovel) and 2 logs; iron tools 3 iron plates (1 for a
  shovel) and 2 iron rods. Bare hands still break everything, only slower.
- **Saplings.** Leaves now and then leave a sapling behind, whether you break them or they fall off a
  felled tree. Plant it on dirt or grass and it grows into a tree after a few minutes, as long as
  nothing blocks its trunk.
- **Prospecting.** A scanner (4 iron plates, 6 copper wire, 4 screws) lists the deposits within 48
  blocks when you right-click with it: which ore, vein or outcrop, how far, which way, how deep and
  roughly how big; the arrows turn as you walk. A core drill (6 iron plates, 2 copper wire, 8 screws)
  held on the ground for 3 seconds tells you exactly what lies beneath: how many blocks and units are
  left, and between which heights. Neither wears out.
- **A miner** placed against any block of a deposit draws from the whole pool and delivers 60% of what it
  drills, while it has power (see Power). All miners on one deposit share its draw limit. Output slows over the last 20% of the pool.
  Each time a block's worth is used up, the ore block nearest the miner turns into spent rock, even in
  chunks that aren't loaded. A worked-out deposit is gone for good.
- **Conveyor belts** carry items one block per second. Hold right-click on the ground and drag to lay a
  whole line: outlines show where the belts go (red past the belts you have) and releasing builds it.
  A single belt runs the way you face; press R on a belt to turn it. A belt ending in another belt's side
  merges into it. A belt fed only from one side turns the corner.
- **Climbing and crossing.** Belts climb and drop one-block steps by themselves: a belt with a belt one
  block ahead and one up becomes a ramp up, and one below a belt one up behind it a ramp down. Stacked lifts carry items straight up, and the top one hands them on
  one ahead and one up. An underpass entry sends items under whatever is in front of it to the nearest
  exit facing the same way, up to 5 blocks ahead, so two lines can cross.
- **Storage boxes** accept items from belts that end in them. They push items into belts that lead away
  from them, and miners next to a box fill it directly. Right-click a box to open it like a chest (Take all empties it); right-click a miner to take its ore.
- **A smelter** melts iron or copper ore into ingots, one every 1.5 seconds while its fire burns (and two
  quartz ore into a block of glass every 2 seconds). Feed it ore
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
- **Power.** Miners, constructors, splitters, filters, labs, pumps and quarries need power; the smelter
  burns its own fuel. A coal generator turns coal ore (270 kJ) or logs (135 kJ), brought by belt, by a
  miner beside it or by hand, into stored energy and gives up to 60 kW, only as much as its grid draws,
  so fuel lasts longer under a light load. Power poles link to every pole within 10 blocks, and each
  generator and machine hangs on the nearest pole within 5 (you see the wires). A drilling Miner Mk1
  draws 5 kW (one coal runs it long enough to mine about 32), a Mk2 20 kW, a working constructor 15 kW, a
  splitter or filter 1 kW. Short of power, every machine on the grid slows down to match; with none, it
  stops. The first loop: a miner on coal next to a generator, a pole within reach of both, and a log to
  start the fire; from then on the miner fuels its own power.
- **Research.** Splitters, filters, lifts, underpasses and green science packs start locked (greyed
  out in the build menu). A research lab uses science packs to unlock them: press T, choose a tech, and
  every powered lab with the right packs works on it, one unit at a time (5 or 10 s each, one of each of
  the tech's packs). Red packs are an iron plate and 2 copper wire; green packs, unlocked by research, are
  2 belts and 4 screws. Belt Routing (10 red) unlocks splitters and filters, then Belt Lifts (20 red),
  Green Science (30 red), and with red and green packs Underpasses (15), Miner Mk2 (30) and Fast Belts
  (20). Fluid Handling (15 red, after Belt Routing) unlocks pumps, pipes and outlets. A lab draws 10 kW
  while it works.
- **Pumps and pipes.** A pump lifts 2 blocks of still water a second out of the water it touches, the
  highest first (5 kW). Pipes join on every side; an outlet at the other end pours the water out in front
  of it (it faces the way you did when you placed it): down to where it lands, then filling that pool from
  the bottom up. With nowhere to pour, the pumps stop. Drain a pond or a flooded pit this way; the sea
  refills whatever you take, so it can't be lowered.
- **Quarries.** A quarry digs the ground in front of it for real: stone, rock, soil and sand, about 2 blocks
  a second at full power (10 kW), layer by layer from its own level down. While you hold one, its box shows
  in amber with how many blocks it would yield; R turns it. Its panel picks the size (5 to 11 wide) and the
  depth (8 or 16 layers, to sea level, to bedrock), pauses it, and lists the ore veins the pit uncovered,
  which it leaves standing for miners. Its output feeds belts like a miner's. If the pit breaks into
  water, it waits until you pump the water out.
- **Upgrades.** The Miner Mk2 draws twice as fast as a Mk1 (where the deposit allows) and keeps 75% of
  what it draws instead of 60%, so the same deposit yields a quarter more ore. It needs power (20 kW while
  drilling). Fast belts move items at 2 blocks a second and mix freely with ordinary belts.
- **Machine panels.** Right-click a smelter, constructor, filter, generator, lab or quarry to see what it's doing, put items in
  straight from your inventory (ore, fuel, ingots, packs), and take what it made.

Craft machines in the build menu (E). It groups recipes by kind; type in its search box to find a recipe
or everything made from a material, and filter by what you can craft now, what misses materials and what
research still locks. Hover a recipe for its description and materials; click it to craft one, Shift-click
for up to five. A Miner Mk1 costs 10 iron ore, 6 copper ore and 12 stone. Four belts
cost 1 iron ore and 2 stone, a box costs 6 logs and 2 iron ore, a smelter 16 stone and 4 iron ore, a constructor 10 iron ingots,
4 copper ingots and 8 stone, a splitter 2 iron plates and 2 belts, a filter 2 iron plates, 2 copper
wire and 2 belts. Two lifts cost 2 iron rods and 2
belts, and an underpass entry or exit 2 iron plates and 2 belts. A coal generator costs 6 iron ore, 4
copper ore and 12 stone; two power poles an iron ore, a copper ore and a log; a research lab 6 iron plates,
8 copper wire and 4 belts; a Miner Mk2 a Mk1, 8 iron plates, 16 screws and 12 copper wire; two fast belts 2
belts, an iron plate and 4 screws; two lamps a glass block, an iron plate and 2 copper wire; a pump 6 iron
plates, 4 iron rods and 6 copper wire; four pipes 2 iron plates; an outlet 3 iron plates and 2 iron rods; a quarry 12 iron plates,
8 iron rods, 16 screws and 8 copper wire. Hand-mining an outcrop or two covers
your first miner, generator and poles. After that, let them do the work.

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
carries on identically. Co-op builds on this (lockstep): every player's machine runs the core, only actions
travel, the host orders them into per-tick frames, and machines compare state hashes every second, resyncing
from a snapshot if they ever differ. The view is local presentation and never feeds back into the core.

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
- **Greedy meshing with per-vertex AO and smooth light.** Faces merge only where AO and light are constant along
  the merge axis, so merging never changes the shading. Quads are split along the brighter diagonal to avoid seams.
- **Light in the mesher.** Sky and block light (0–15) are flooded over the chunk plus a 15-block margin while it
  meshes, so they are never stored and chunk borders always match; an edit relights the chunks it can reach.
- **5 bytes per vertex.** Position, face, AO and texture layer are packed into one `u32`, plus a light byte. UVs are derived in
  the shader from position, and texture wrapping tiles merged quads. One shared index buffer serves every chunk.
- **Texture array** (`TEXTURE_2D_ARRAY`) instead of an atlas. There is no bleeding, and mipmaps and
  anisotropic filtering work correctly.
- **Separate opaque and cutout passes.** Only leaves and plants use `discard`, so opaque geometry keeps early-z.
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
  23 underpass exit, 24 generator, 25 power pole, 26 research lab, 27 Miner Mk2,
  28 fast belt, 29 sapling, 30 granite, 31 sandstone, 32 basalt, 33 limestone, 34 quartz ore, 35 glass, 36–39 stained soils (grass a shade off), 40–43 stained sand, 44 lamp, 45 water, 46–52 flowing water, 53 pump, 54 pipe, 55 outlet, 56 quarry; items: 256 iron ingot, 257 copper ingot,
  258 iron plate, 259 iron rod, 260 screws, 261 copper wire, 262 red science pack, 263 green science pack,
  264–269 stone and iron tools, 270 scanner, 271 core drill.
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
- [x] Co-op multiplayer: player-hosted over WebRTC, 2–4 players
- [ ] Simulation and worldgen in Web Workers
- [x] Water: a sea and ponds, flowing water, swimming
- [x] Full inventory screen
- [x] Day/night cycle, sky and voxel lighting (sky light, lamps)
- [x] Minimap
- [x] A living surface: leaves of felled trees decay, grass grows back over bare dirt, saplings grow into trees
- [x] Tools that wear out: pickaxe, axe and shovel in stone and iron
- [x] Biomes: plains, desert, highlands, lowlands, basalt fields, each with its own rock
- [x] Ores that follow the rock: limestone and quartz join coal, iron and copper
- [x] Surface hints: stained soil above deep deposits
- [x] Prospecting: a scanner and a core drill

Factory layer (the Satisfactory half):

- [x] Ore deposits (outcrop, vein, lode) with shared pools, draw limits and depletion
- [x] Miner Mk1, conveyor belts (merging, corners) and storage boxes, crafted by hand
- [x] Item registry separate from blocks (ingots so far) and machine recipes
- [x] Smelter: ore and fuel into ingots
- [x] Constructor and parts (plates, rods, screws, wire), with a machine panel
- [ ] Machines: assembler
- [x] Splitters and filters
- [x] Belts that climb (ramps by placement, lifts) and cross (underpasses)
- [x] Drag-to-build belt lines, R to rotate
- [x] Miner and belt tiers (Miner Mk2, fast belts)
- [x] Prospecting: find veins and lodes without digging blind
- [x] Power grid: coal generators that burn only what is used, poles, brownouts; every miner runs on it
- [x] Pumps, pipes and outlets: drain ponds and flooded pits
- [x] Quarry: automated digging that leaves a real pit
- [x] Research: labs, science packs and a small tech tree
- [ ] An optional endgame megaproject that doesn't end the game
- [ ] Terraforming machines: excavators, graders, tunnel borers
- [ ] Blueprints and construction drones
- [ ] Trucks and trains
