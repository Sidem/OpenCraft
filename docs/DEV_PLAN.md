# OpenCraft development plan

**Status:** 2026-09-29 · Milestones 1–6 done (co-op tested across machines by the user; no TURN for
now) · **Now: Milestone 7 (Electronics, blueprints and drones). Next up: step 7.3 (solar power and
accumulators)** · Terraforming is Milestone 8 (its sites step is built and stays in the core) · The `art` branch is superseded; art work
continues from `main` (`docs/ART_HANDOVER.md`).

> **This project is written entirely by AI coding agents.** Every session starts cold, and every line an
> agent has to read costs tokens and time. **Keeping the codebase small, modular and cheap to read is as
> important as any feature.** A sprawling codebase makes every future feature slower and more expensive,
> and that cost compounds. The rules in **section 3.1 are mandatory** and override convenience. When a
> change would break them, restructure first.

This is the plan of record. It says where the game is heading, what exists today, and exactly what to build
next. README.md covers setup and how the game works for players; this file covers direction and work.

---

## 0. Handover: read this first

You are picking up a working browser factory game (Rust → wasm engine, TypeScript/WebGL2 host), live at
<https://sidem.github.io/OpenCraft/>.

- **Milestone 1 (Foundation) is done** (section 8): a fixed 60 Hz tick, a deterministic core changed only
  by actions, several players in the engine, state hashes, and worlds that save in the browser.
- **Milestone 2 (Make it a game) is done** (section 8): items, smelter, constructor, belt logistics, power,
  research with science packs, Mk2 upgrades and onboarding tips.
- **Milestone 3 (Co-op) is done** (section 8): 2–4 players in one world over WebRTC, the host's browser as
  the authority, a Cloudflare Worker to connect them, hosting and joining from the menu.
- **Milestone 4 (Reasons to explore) is done** (section 8): biomes and rock provinces, ores that follow
  geology, surface hints, prospecting, map marks, day and night, sky light and lamps, a new build menu.
- **Milestone 5 (Water and world shape) is done** (section 8): sea and ponds, swimming, flowing water,
  pumps, a quarry, rarer surface ore (generator version 3), and powered miners.
- **Milestone 6 (Industry) is done** (section 8): tiers as data, one processing machine, multi-block
  footprints, upgrade kits, the assembler, steel, blue science, Mk3 everything, steam power, the crusher
  and the silo.
- **Now: Milestone 7: Electronics, blueprints and drones** (section 4): the arc furnace, circuits, violet
  science and Mk4, solar power, logic, then blueprints, construction drones and the jetpack.
- **Tech tree:** `docs/TECH_TREE.md` is the concept (lines, links, the far end, upgrades, and the content
  architecture every step follows: its section 8); `docs/TECH_ERAS.md` has each era's items, recipes,
  machines and techs. Read the concept once and the era you build.
- **Art** (`docs/ART_HANDOVER.md`) says who owns which looks. When gameplay needs a new look, append a
  `tex` layer with a plain placeholder pattern and add a line to that file's request list.

Before you change code:

1. Read sections 0–4 of this file (section 3.1 carefully), then `docs/CODEMAP.md`, then the nested
   `CLAUDE.md` of the area you work in. Skim README.md only if you need the player's view.
2. Run `npm run build:wasm` (if `web/src/wasm` is missing) and `npm run check` to confirm a green baseline
   (305 engine tests).
3. Work through the current milestone in step order. Each step lists where, how and when it's done. Do
   one step, or one clean part of a step, per session, and stop in a green, committed state.
4. When a step is done, tick its checkbox here, update the **Status** line at the top, and add a line to
   the change log (section 8). Update `docs/CODEMAP.md` in the same commit as any structural change. Keep
   this file truthful; the next session relies on it.

Ground rules (details in `docs/WORKFLOW.md`):

- **Never push to `main` without asking the user.** Every push deploys the live site.
- Commit only when the user asks. So far the user has had finished work committed and pushed straight
  to `main` when they asked for a deploy.
- Windows machine: use PowerShell and the file tools; the Bash tool fails here.
- **Keep the codebase agent-friendly (section 3.1).** File budgets, tests in separate files, docs as maps.
- Performance comes first, and dependencies must be justified. Explain things to the user in plain language.

---

## 1. Where the project is heading

A browser game between **Minecraft** and **Satisfactory**: a fully editable voxel world that you mine,
shape and industrialise. Not a Minecraft clone; its conventions can be broken freely.

### Pillars

1. **Everything is physical.** Items travel on belts, machines are blocks in the world, the world is the
   material. No magic teleporting storage.
2. **Finite ground pushes you outward.** Deposits run dry after a long time. Local depletion drives
   expansion, and distance, depth and scale become the problems each tier solves.
3. **Terrain is something you engineer.** Cut, fill, flatten, tunnel, flood. Terraforming machines are the
   game's signature feature; no other game in the genre has them.
4. **Automation replaces your hands at every scale:** mining first, then logistics, then construction.
5. **Every tier adds a new kind of problem, not just recipes:** quantity → power → distance → depth and
   terrain → fluids → scale.

### Decisions (settled with the user)

| Date | Decision |
|---|---|
| 2026-09-25 | **Finite but long-lived deposits.** A factory lasts a long time but not forever. |
| 2026-09-25 | **Efficiency model.** Hand mining is lossy, machines are efficient; recovery rate is a progression axis. Numbers will need tuning. |
| 2026-09-25 | **Peaceful** for the foreseeable future. No enemies or combat. A tower-defence-style mode may come much later, as a world setting. |
| 2026-09-25 | **Open-ended.** An optional final construction (e.g. a rocket ship) needing enormous resources and advanced research. Completing it must **not** end the game; it should unlock something. |
| 2026-09-25 | **Built entirely by AI coding agents.** Keeping the codebase from growing unwieldy is of utmost importance; token efficiency and fast iteration drive architecture choices (section 3.1). |
| 2026-09-25 | **Co-op multiplayer must be possible**: other people can join a world. |
| 2026-09-25 | **Research is Factorio-style**: labs use science packs crafted from increasingly complex parts (<https://wiki.factorio.com/Science_pack>). |
| 2026-09-25 | **Upgrades raise recovery**, not just speed (Mk2 miner ≈ 75%), so upgrading extends a deposit's life. |
| 2026-09-25 | **Bulk materials** (stone, sand, clay, gravel) stay effectively infinite for quarries. |
| 2026-09-25 | **Approach for co-op** (proposed by Claude, adopted with this plan): player-hosted over WebRTC; a deterministic simulation core driven by tick-stamped actions (Factorio-style); player movement and loose items replicated from the host. Revisit only if Milestone 3 prototyping shows a real problem. |
| 2026-09-25 | **Co-op hosting on Cloudflare**: a Cloudflare Worker for signalling and Cloudflare TURN as the relay. The user owns the account. |
| 2026-09-25 | **Co-op size: 2–4 players.** This sets the bandwidth and performance budgets. |
| 2026-09-26 | **No TURN relay for now**: STUN only. Players whose networks block direct connections can't join; adding the two TURN secrets later (`docs/WORKFLOW.md` section 4) needs no code change. |
| 2026-09-26 | **Tools wear out and come in tiers.** A better pickaxe keeps slightly more ore by hand, but nowhere near what machines recover (step P5). |
| 2026-09-26 | **Ores follow geology.** Coal, iron and copper are placed by rock type and biome (for example copper in mountains, coal in lowlands); limestone and quartz are added; visible rock types (granite, sandstone, basalt) say what lies beneath. |
| 2026-09-26 | **Deposit rates stay as today** (how many outcrops, veins and lodes); tune after play. Surface outcrops superseded on 2026-09-27. |
| 2026-09-26 | **The world pauses while the game is closed.** No simulating missed time. |
| 2026-09-26 | **New generation rules for new worlds only.** Worlds made before Milestone 4 keep generator version 1 and play as before. |
| 2026-09-27 | **Belts:** ramps come from placement (no research), lines are dragged out, R rotates, research moved to T. |
| 2026-09-27 | **Water and world shape before terraforming** (Milestone 5): sea level, lakes, swimming, pumps and pipes. Rivers later (now Milestone 10). |
| 2026-09-27 | **Water flow is limited and deterministic:** a Minecraft-like spread, event-driven and capped per tick; no volume simulation. |
| 2026-09-27 | **Surface ore is rare and meaningful:** about 10× fewer exposed outcrops, only on bare rock; a starter set near spawn; ores in depth bands. Vein and lode counts stay. |
| 2026-09-27 | **A quarry in Milestone 5:** rock and soil get automated extraction that really digs a pit (the first excavator); it must be intuitive and satisfying to watch. |
| 2026-09-27 | **Order after Milestone 5: terraforming, then blueprints and construction drones, then transport** (personal flight first: jetpacks or hovering, and building from further away). The player should feel a steady, intuitive and fun growth of capability: building by hand → planning → automatic assembly. *Order superseded on 2026-09-28 (Industry first).* |
| 2026-09-27 | **Finding starter ore made easier** (play-test: iron under rusty soil was too deep to find by hand). Generator version 4 for new worlds: shallower bands, coal, iron and copper exposed 1.5× as often on bare rock, two starter patches of each; a world map (M) of explored ground with pins; an ore guide with depth bands; stained soil tells what lies how deep. Mute moved to K. |
| 2026-09-28 | **Glass from sand; brighter lights.** The smelter makes glass from sand (1 → 1); quartz gives 2. Torches reach 11 (full within 4), lamps 31 (full within 17): the light margin is 32, the most the 3×3×3 field allows. |
| 2026-09-28 | **Processed wood and ladders.** Logs saw into planks, planks into sticks; sticks make tools, torches, poles and ladders (a see-through frame: jump climbs, crouch descends, idle holds). |
| 2026-09-27 | **Miners need power; generators burn only what is used.** The first loop is a miner on coal feeding the generator that powers it. Energy is stored per generator (kJ); one coal (270 kJ) runs a Mk1 (5 kW) long enough to mine about 32 coal. The smelter stays a burner. |
| 2026-09-28 | **A deeper tech tree** (user request): each tier is built from the one before and makes every aspect more efficient (extraction, yield, logistics, power, processing, research, reach); machines several blocks big with ports for more inputs and outputs; an easy upgrade system: tier stripes coloured like the science packs (red, green, blue, violet, gold), upgrade kits applied in place, belt lines upgraded by dragging. Design: `docs/TECH_TREE.md`. |
| 2026-09-28 | **Industry is the next milestone (6); terraforming moves later** "where it makes more sense" (supersedes the 2026-09-27 order). Every technology opens a possibility, improves efficiency or helps the player. Targets include AI datacenters, building with drones, satellite constellations, and energy and data sent by laser; the only limit is the stack (Rust, wasm, WebGL2 in a browser). |
| 2026-09-28 | **Modular, open content architecture** (`docs/TECH_TREE.md` section 8): variants such as tiers are data (`tier: u8` into a table), never a `bool` like today's `fast` belt or `mk2` miner, nor a block per variant; processing machines are rows of one generic machine; recipes belong to categories. |
| 2026-09-28 | **The order after Industry** (proposed by Claude, confirmed): 7 Electronics, blueprints and drones (with the jetpack) · 8 Terraforming (its excavator flies Milestone 7's drones) · 9 Distance (aluminium, trains) · 10 Fluids and chemistry · 11 Compute and photonics · 12 Orbit · 13 The megaproject. **Blue packs and upgrade kits from blue on are machine-made only.** |

### Proposed, not yet confirmed by the user

- Old saves keep loading across format changes where a migration is cheap. Every save since version 1
  still loads.
- Terraforming: one excavator machine with work drones rather than separate grader, filler and borer
  machines; dug ground fills other sites first and goes onto belts after (a dump is a fill site); ore in
  a site is cut like hand mining (lossy), with a warning when planning.

---

## 2. Where the code stands (after Milestone 6)

**Milestone 6 added** (the bullets below still describe Milestones 1–5; `docs/CODEMAP.md` is current):
tiers as data up to Mk3 for every family, upgrade kits, one processing machine driven by `ProcessSpec`
rows (smelter, constructor, assembler, blast furnace, boiler, steam turbine, crusher, silo), multi-block
footprints, steel and blue science, steam power as a second kind of power source, 16 techs, 14 tips,
save version 21.

### Architecture

Rust owns all game state and hot loops (`crates/engine`, compiled to wasm with wasm-bindgen). TypeScript
(`web/src`) is a thin platform layer: input, WebGL2 rendering, DOM UI, Web Audio. Bulk data (chunk meshes,
box instances, sound events, textures) is read zero-copy from wasm memory through `*_ptr` / `*_count`
accessors. `Game` (`lib.rs`) is a thin facade: its JS-facing API is split by area into `api/*.rs` and acts
for the local player (`Game.local`). The deterministic core is `Sim` (`sim.rs`: tick, world, factory with
deposits and research, each player's inventory, rng); `Sim::state_hash` fingerprints it through the
canonical encoding in `bytes.rs`, which `save.rs` also reads back for saves. The authority
(`authority.rs`) owns every player's body and the loose items. The local player's hands (mining, placing,
footsteps) live in `interaction.rs`. `Game` changes the core only by queuing `Action`s (`action.rs`),
applied at the next tick; the core answers with `SimEvent`s (`events.rs` reacts). `docs/CODEMAP.md` maps
every module.

Each frame, `web/src/main.ts`:

1. forwards input (`set_move`, `look`, `set_mining`, `set_using`, actions from `input.ts`),
2. calls `game.update(dt)`. This runs streaming, then as many fixed 60 Hz ticks as the frame time adds up to
   (`Game::run_tick`: every body's physics, the local player's targeting, mining and placing, item
   entities, all queuing actions, then the core's `Sim::step`, then `handle_sim_events` for item spawns,
   sounds and toasts), then interpolates the camera between the last two ticks and writes box instances,
3. runs `begin_work()` + `work_step()` under a time budget (generate or mesh one chunk per step),
4. drains mesh and unload events to the renderer, plays sound events, renders, updates the HUD.

**Co-op** is lockstep: only actions travel. The host stamps every action (its own too) for
`tick + INPUT_DELAY` (4) and sends one frame per tick; clients step their core only through the ticks
they have frames for, and compare state hashes every 60 ticks (a mismatch resyncs from a snapshot).
Bodies are each machine's own (states at 20 Hz); loose items run on the host only (views at 10 Hz). The
core never learns about the network: `net/` (Rust) moves bytes, `web/src/net/` runs the session over a
`Transport` (WebRTC between machines, BroadcastChannel between tabs). Budgets for 4 players: under
~10 KB/s per client outside snapshots, and the host's frame under 2 ms more than solo.

### What exists

- **World:** 32³ chunks, 256 tall; seeded terrain with cliffs, caves and trees; greedy mesher with AO;
  streaming nearest-first; edits kept when chunks unload (`World.saved`). Generator versions: a world
  keeps its own (`WorldGen::version`); version 2 adds biomes (`worldgen/biome.rs`: plains, desert over
  sandstone, highlands over granite, lowlands, basalt fields; spawn always plains) and ores by biome
  (`worldgen/geology.rs`, including limestone and quartz) with stained soil hints above veins and lodes.
  Version 3 (frozen) makes surface ore rare (only on bare rock, a starter set 40–80 blocks out),
  puts ores in depth bands (`worldgen/strata.rs`), and adds the sea (level 62) and ponds
  (`worldgen/water.rs`). Version 4 (current) only retunes that: shallower bands, more exposed metal, two
  starter patches each 28–110 blocks out. `worldgen/tests.rs` pins versions 1–3.
- **Water** (block 45 still, 46–52 flowing): drawn blended with underwater fog (`render/water.ts`);
  flow is core state, event-driven and capped per tick (`sim/water.rs`); the sea is the only endless
  water. Bodies swim and items float (`player`, `entities`).
- **Day and light:** a 20-minute day from the core tick (`daytime.rs`), sky, sun, moon and stars
  (`render/sky.ts`); sky and block light 0–15 computed while a chunk meshes (`light.rs`: the chunk plus a
  32-block margin), smoothed per vertex (a byte after the vertices); light sources (`light::SOURCES`): lamps
  (block 44, full within 17, reach 31) and torches (block 57, full within 4, reach 11).
- **Deposits** (`deposits.rs`, placed by `worldgen/ore.rs`): outcrops, veins and lodes of coal, iron and
  copper (100 / 1,000 / 2,000 units per block, shared draw caps 60 / 240 / 1,200 per minute). A pool is
  shared per deposit, output tapers over the last 20%, and blocks turn to `SPENT_ROCK` as it drains, even
  in unloaded chunks. Hand mining keeps `HAND_YIELD` = 3 per block and costs the deposit one block.
- **Factory** (`factory/`, one file per machine kind, the `MACHINES` table and a `Machine` trait; one kind
  can serve several blocks through extra table rows):
  - Miners: Mk1 (1 unit/s, 60% recovery, 5 kW) and Mk2 (2 units/s, 75%, 20 kW). The quarry
    (`factory/quarry/`, 10 kW) digs a box of ground into a real pit; pumps, pipes and outlets
    (`factory/pipes.rs`, `pumping.rs`) move water.
  - Belts (1 block/s; fast belts 2) with side-joins, corners, back-pressure, ramps, lifts and underpasses
    (`belt_shape.rs`); splitter and filter (`router.rs`); storage boxes (24 slots, open like a chest).
  - Smelter (ore plus coal or logs → ingots) and constructor (ingots → plates, rods, screws, wire), with
    machine recipes and fuels as data (`recipes.rs`). Right-click opens a machine panel (`panel.rs`,
    `ui/machine.ts`): status, buffers, recipe or filter choice, put-in and take buttons.
  - Power (`power.rs`): coal generators that store fuel energy and give only what is drawn (up to 60 kW),
    poles that link within 10 blocks into grids, machines on the nearest pole within 5, brownouts as a
    speed factor.
  - Research labs (`lab.rs`) working through the tech tree (`research.rs`, key T) with red and green
    science packs; seven techs unlock routing, lifts, underpasses, green packs, Mk2, fast belts and fluid
    handling.
  - Models are instanced boxes; status readouts come from `describe`.
- **Inventory** (`inventory.rs`): 36 slots (hotbar 0–8), a cursor stack, click, shift-click and
  quick-move. **Crafting** (`recipes.rs`): hand recipes in the build menu (key E, `ui/crafting.ts`): a grid by `recipes::Group` with search, filters (can craft, missing, locked) and a hover card.
- **Onboarding tips** (`hints.rs`, `ui/hints.ts`, key H skips): ten tips from finding ore (pointing at the
  map) to the quarry.
- **Maps** (presentation only): the explored map (`minimap/atlas.rs`: every column seen, kept after
  unloading, stored in the browser beside the save) feeds the minimap (key N) and the world map (key M,
  `ui/worldmap.ts`: pan, zoom, pins). Marks (`minimap/marks.rs`): diamonds for ore seen at the surface,
  rings for prospected veins and lodes, squares for machines; pins are the player's notes (`ui/pins.ts`,
  in the world's record). Beside the map, the ore guide (`ore_guide.rs`, `ui/ore-guide.ts`) charts each
  ore's depth band from the world's own generator; pointing at stained soil tells the ore below and its
  depth.
- **Block timers** (`sim/timers.rs`, core state): leaves of a felled tree decay (half-life 5 s), grass
  spreads onto bare dirt beside it (30 s) and turns to dirt under a solid block (15 s), saplings grow.
  Timers start only from block changes, never from scanning chunks.
- **Saplings** (`sim/saplings.rs`): leaves drop one 1 time in 25 (broken or decayed); planted on dirt or
  grass, it grows after 60 s plus a 90 s half-life into a tree shaped like generated ones
  (`worldgen::tree_blocks`); drawn as crossed quads (`Render::Plant`, mesher faces 6 and 7).
- **Items** (`item.rs`): `ItemId(u16)`. Ids below 256 are the blocks with the same number (0–59, see
  `block/mod.rs`; 29 is the sapling, 30–43 Milestone 4's rocks, ores, glass, stained soils and sand, 44 the lamp, 45–52 water, 53–55 pump, pipe and outlet, 56 the quarry, 57 the torch, 58 planks, 59 the ladder); from 256: iron and copper ingots, iron plate, iron rod, screws, copper wire, red and green
  science packs, the six tools (264–269), scanner (270), core drill (271) and stick (272). Every item is drawn as a textured box.
- **Sound:** procedural foley, 7 materials including metal, and a sound designer (key O).
- **Debug API** on `window.opencraft.game`: `give`, `teleport`, `run_ticks`, `skip_time`, `find_deposit`,
  `block_at`, `toggle_fly`, `set_look`, `add_player`, `remove_player`, `state_hash`, `debug_desync`.
- **Saving** (`save.rs`, `web/src/save/`): worlds autosave to IndexedDB; the menu lists, creates,
  exports and imports them. Save version 17; every version since 1 loads.
- **Co-op** (`net/`, `web/src/net/`, `signal/`, `ui/coop.ts`, `ui/players.ts`): "Play together" in the
  menu hosts the open world (a room code and link from the signalling Worker) or joins from a pasted link
  or code; up to 4 players; a returning player gets their things back (player keys, `Sim.away`). Avatars
  with name tags, a Tab player list with ping, join and leave notices, readable refusals (full, another
  version, no such game, no connection), a hidden co-op tab keeps ticking, a silent peer times out after
  10 s, and a client that differs from the host or falls 5 s behind resyncs in place. URL shortcuts for
  testing: `?host`, `?host=<room>` / `?join=<room>` (tabs of one browser), `?join=<code>`, `?relay`.
- **Tools** (`tools.rs`, recipes at the end of `RECIPES`): pickaxe, axe and shovel in stone (150 uses, 2×)
  and iron (600, 4×, an iron pickaxe keeps 4 ore); a tool's stack count is its uses left, shown as a wear
  bar (`showAmount` in `ui/hud.ts`). Placeholder looks in `textures/tools.rs`.
- **Prospecting** (`prospect.rs`, `ui/prospect.ts`): the scanner lists deposits within 48 blocks (ore,
  tier, live bearing and distance, depth, size band); the core drill (3 s hold) gives a column's exact
  figures. Queries only: no actions, the state hash never moves. Devices are tools that never wear.
- **Size:** about 261 KB gzipped in total (wasm 201.6 KB, JS 51.1 KB, CSS 6.4 KB; `vite build`).

### Known limitations and technical debt

1. Only the local player has hands: another player's mining and placing reach this machine only as
   their results (by design; co-op sends actions, not hands).
2. Two tabs on the same world overwrite each other's saves (the later save wins).
3. ~~Veins and lodes can only be found by digging~~: fixed by prospecting (4.6) and map marks (4.7).
4. In version 1 and 2 worlds the outcrop nearest spawn may be buried (version 3 has a starter set).
5. TypeScript mirrors a few engine constants: `INSTANCE_FLOATS`, the 6 floats per sound event, and the
   order of sound materials and event kinds. Replace them with getters when touching that code.
6. Item and belt instances aren't interpolated between ticks (only the camera is); optional polish.
7. Balance is untested by real play: pack costs, research times and Mk2 costs will need tuning
   (`docs/WORKFLOW.md` section 6 lists the numbers).
8. In co-op every action applies about 70 ms plus the round trip after it's made; your own block edits
   aren't shown early (section 4, "instant feedback"). A name change applies the next time you host or
   join. Hosting can't be stopped without leaving the page.
9. Co-op over the internet has been tested by the user on two machines (STUN only); the relay is untested.
10. ~~A block timer next to an unloaded chunk generates it for each read~~: fixed in 5.6 (`World` keeps the
    last 8 generated chunks).
11. Light is recomputed per meshed chunk (0.43 ms per chunk in `bench_meshing` after the torch change); an
    edit relights the chunks within 20 blocks, and all below, over the following frames. A working smelter
    doesn't glow (its light would depend on factory state).
12. The explored map and pins live in one browser, per world: a co-op client keeps neither (it never
    saves), and pins aren't shared with other players. World export doesn't carry them.

---

## 3. Engineering rules for all new work

### 3.1 Keep the codebase small and cheap to work on (TOP PRIORITY, mandatory)

**Why this matters more than usual:** only AI agents develop this project, and each session starts with no
memory of the code. Tokens go to four things:

1. reading code to understand it (the biggest cost),
2. exploring to find where things live,
3. failed build and test iterations, and the output they print,
4. re-learning the project every session.

Structure decides all four. A 400-line module behind a clear interface is cheap to change forever; a
1,000-line god object taxes every feature that touches it. **Treat context cost like frame time: a budget
you measure and defend.** If a change would break these rules, restructure first, in its own commit.

**Architecture: changes stay local**

- **One feature, one module or folder.** Adding a machine, item, recipe or UI panel means new files plus at
  most one registration line in a central place. If you find yourself editing five files for one feature,
  the structure is wrong; fix it.
- **No god objects.** The wasm facade (`Game`) is thin: its API is split by area into `api/*.rs` files
  (several `#[wasm_bindgen] impl Game` blocks are allowed), and each method only forwards to a module.
- **Content is data.** Blocks, items, recipes, machines, tech tree and ore settings are tables. Code is
  written per *kind* of behaviour, never per individual item.
- **Simulation and presentation are separate modules** (section 3.4). A UI change never requires reading
  simulation code, and the reverse.
- **Boring, explicit code.** Shallow call chains, no macro tricks, no deep generic towers, no ECS framework.
  Plain data-oriented modules: typed storage in `Vec`s plus one function per system. Don't build
  abstractions ahead of need. Introduce a registry when the second instance of a kind arrives (e.g. the
  machine registry arrived with the smelter).
- **Variants are data, never booleans.** A tier, material or mode is a field indexing a table; a new
  tier or machine that only differs in numbers is a row. Content lint tests keep the tables consistent.
  The full rules: `docs/TECH_TREE.md` section 8.

**Size budgets** (enforced by `scripts/check-size.mjs`, part of `npm run check`)

| Kind | Soft limit (warn) | Hard limit (fail) |
|---|---|---|
| Rust / TS source file, excluding tests | 400 lines | 600 lines |
| CSS file (one per UI component) | 300 lines | 500 lines |
| Any `CLAUDE.md` | 60 lines | 100 lines |
| This plan (read every session) | 600 lines | 800 lines |
| Other docs (`CODEMAP`, `ROADMAP`, `WORKFLOW`) | 300 lines | 500 lines |

Count all lines, blank ones included: `(Get-Content f).Count`, not `Measure-Object -Line`.

- Before adding to a file over its soft limit, split it.
- **Tests never live inline in implementation files.** Declare `#[cfg(test)] mod tests;`, which resolves to
  `src/foo/tests.rs` from both `src/foo.rs` and `src/foo/mod.rs`. Reading a module must not pull in its
  tests.

**Readability: cheap to skim**

- **Every module starts with a header** (`//!` in Rust, a top comment in TS) of at most ~15 lines: what it
  owns, its key invariants, and how to extend it ("to add a machine: …").
- **Public items first, private helpers after**, so the top of a file is its interface.
- **Unique, searchable names** (`belt_step`, not a tenth `update`), so one search finds the right spot.
- **One source of truth.** Never mirror constants between Rust and TS; expose a getter (like `hand_yield()`)
  or generate it.
- **Delete dead code immediately.** No commented-out code, no old and new paths living side by side after a
  refactor.
- **Comments explain *why*,** briefly. Don't narrate what the code already says.

**Docs: maps, not history**

- `docs/CODEMAP.md`: one line per module (what it owns) plus "how to add X" recipes.
  **Update it in the same commit as any structural change.** It replaces most exploration.
- **Nested `CLAUDE.md` files** in `crates/engine/` and `web/src/` hold subsystem conventions. Claude Code loads
  them only when working in that folder, so they cost nothing elsewhere. Keep them short.
- **This plan details only the current milestone.** When a milestone finishes, compress it into section 8
  (a few lines) and move the next milestone in from `docs/ROADMAP.md`, detailed to the level of the
  current one (steps with where, how and done-when). Future milestones stay as short bullet lists in the
  roadmap, which is read only when planning.
- **Operational reference lives in `docs/WORKFLOW.md`** (commands, verification, gotchas); read only the
  section you need.
- The root `CLAUDE.md` holds rules and pointers only.

**Feedback loops: fast and quiet**

- **`npm run check`** runs format check, clippy with warnings as errors, engine tests, typecheck
  and size budgets, and prints only a summary and the failures. Run it before every commit.
- **Prefer headless tests over the browser.** Scenario tests in Rust (build a layout through actions, run
  ticks, assert on state or `describe` text) and text debug dumps (e.g. an ASCII map of an area with
  machines and belt contents) are far cheaper than driving the browser pane. The browser is for final
  visual proof only. From Milestone 1 on, bugs should become replayable action scripts.
- **Keep tool output small:** quiet flags (`cargo test -q`), and filter long output to what matters.
- **Let the compiler do the searching:** newtypes (`ItemId`, `PlayerId`) and exhaustive `match`es make the
  compiler list everything a change must touch.

**Session habits**

- One scoped step per session; finish green and committed. Big sessions are where tokens disappear.
- Read line ranges and symbols, not whole files. Use the code map. For a broad search, use a search
  sub-agent so only its conclusion enters the main context.
- End every session by updating this plan (checkboxes, status, change log) and the code map.
- **Every milestone ends with a cleanup step:** size check clean, code map current, plan compressed, dead
  code gone, the next milestone moved in from the roadmap.

### 3.2 Performance

- Hot loops and all game state stay in Rust. TypeScript stays thin.
- Pass bulk data zero-copy (views on wasm memory); don't serialise per frame.
- Measure before and after anything that could cost frame time.

### 3.3 Wasm size

- Report gzipped sizes when the build grows noticeably (`npx vite build` prints them).
- Avoid formatting floats in Rust (`{:.1}` pulls in about 25 KB). Round to integers, or format in
  TypeScript.
- Avoid new instantiations of the std sorts (about 5–9 KB each). Use the insertion sort
  `math::sort_small_by_key`.
- No new crates without a reason worth their size. Hand-written serialisation beats serde+bincode here.

### 3.4 Determinism (required for co-op; enforced from Milestone 1)

The **deterministic core** (world edits, factory, deposits, inventories, crafting, research) must produce
identical state on every machine given the same seed and the same actions. Therefore core code must not
depend on:

- frame time: advance only in fixed ticks,
- which chunks happen to be loaded or meshed locally: use the `*_anywhere` accessors, which generate or
  read the stored copy,
- the camera, the local player, UI state or presentation queries: queries must never create or modify core
  state,
- iteration order of hash maps: iterate `Vec`s or sorted keys wherever order affects results,
- wall-clock time, `Math.random`, or anything from JS,
- (for a future native server) platform math: `sin`/`cos`/`powf` may differ between wasm and native. The
  `libm` crate on both sides fixes that when the time comes.

Everything that changes the core goes through an **action** applied at a tick boundary. Things that are only
presentation (camera, sounds, particles, meshes, HUD, readouts) never feed back into the core.

### 3.5 Code style

- Section 3.1 comes first. Beyond it, match the surrounding code: comment density, naming, idioms.
- Every engine feature gets unit tests in its module. Run `cargo test --workspace --release`.
- Keep README.md in sync for anything player-visible, and keep this plan in sync for everything else.

---

## 4. Now: Milestone 7: Electronics, blueprints and drones

Milestone 6 (Industry) is done: the steps below are its summary; the full specs are in git history
(`git log -- docs/DEV_PLAN.md`). Milestone 7 is moved in from `docs/ROADMAP.md`. Design: `docs/TECH_TREE.md`
(sections 1, 6, 7 and 8) and `docs/TECH_ERAS.md` section 3 (era 4: every number, recipe and tech).

**Rules that still bind every step:**
- `docs/TECH_TREE.md` section 8: tiers are data, processing machines are spec rows, recipes belong to
  categories, unlocks are one enum. No new `bool` per variant, no block per tier.
- Old saves keep loading: each format change bumps `SAVE_VERSION` (now 22) with a migration and a fixture
  test. Golden hashes are re-recorded only on purpose, noted in the step.
- New blocks and items append (the next free block is 72, item 320). Each new look gets a placeholder
  layer (`tex::COUNT` is 158) and a `docs/ART_HANDOVER.md` request line.

### Milestone 6 (Industry), built

- [x] **6.1 Tiers as data** (`factory/tiers.rs`): `tier: u8`, numbers in `*_TIERS` arrays, `FAMILIES` names each
  tier's item. **6.2 Unlocks, categories, content lint** (`research::Unlock`, `recipes/machine.rs`
  `Category`, `recipes/tests.rs`). **6.3 Upgrade kits** (`factory/upgrades.rs`, `Action::Upgrade` tag 23,
  tier stripes; one step at a time, never refunded).
- [x] **6.4 One processing machine** (`factory/process/`: a `ProcessSpec` row per machine; smelter and
  constructor with tiers; save 18). **6.5 Footprints and the assembler** (`factory/footprint/`,
  `action/multiblock.rs`; the anchor holds the block, other cells `MACHINE_PART` 62; ports by side; save
  19). **6.6 Steel** (blast furnace with a byproduct port, slag, steel items and tools).
- [x] **6.7 Blue science and Mk3** (blue pack and kit, three-slot labs, save 20, Mk3 belts, miners,
  smelters (electric), processors) and **6.7b Mk3 for the rest** (poles `factory/pole.rs`, boxes, pumps,
  quarries, labs; generator Mk2; save 21; tier recipes in `recipes/tiers.rs`).
- [x] **6.8 Steam power** (`factory/process/steam.rs`): boiler (block 66, 2×2×2) and steam turbine (67, 3×2×2) are
  spec rows (`Energy::Boiler` / `Turbine`). A boiler burns 540 kJ a coal into steam and takes water (2,000 kJ a
  unit) from a pump on its pipe net; a turbine touching a boiler is a power source `power.rs` asks for what its
  grid lacks (up to 240 kW, two a boiler). Tests: a sea-pump plant holds a 405 kW load for a full research unit;
  a pond plant runs dry; the two-turbine limit; a save round trip.
- [x] **6.9 Crushing and bulk storage:** the crusher (block 68, 30 kW, machine recipes 21–25, items 306–307: 2 ore
  → 3 crushed, slag → sand) and the silo (69, `Pick::Store`, 144 slots, in on three sides, out the front). Techs
  14–16; textures 141–150 (`textures/heavy.rs`), recipes in `recipes/heavy.rs` and `tooling.rs`.
- [x] **6.10 Feel, balance, cleanup:** tips for kits, the assembler, steel and steam (`hints.rs`); golden hash
  re-recorded. Tests 286 → 296, wasm 232.7 KB gzipped, `bench_plant` (20 machines): 1.3 µs a tick, worst 2.15 ms.
### Milestone 7 steps

Goal: the player's reach grows from building by hand to machines doing it. Circuits and violet science
first, then logic, then the builders (blueprints, drones, flight). Steps 7.6–7.8 wait for the user's
answers to the section 6 questions (blueprints, drone materials, jetpack); do 7.1–7.5 first.

- [x] **7.1 Arc furnace and silicon** (built): the arc furnace (block 70, 2×2×2, `Category::Arc`, 120 kW: 1 quartz
  ore + 1 coal → 1 silicon, 4 s; hatches like the assembler) is a spec row; silicon (item 308) and the circuit
  (309: assembler, 1 silicon + 3 copper wire + 1 iron plate → 2) are machine recipes 26–27; the Electronics tech
  (r g b, 80 × 20 s) unlocks all three; textures 151–154. The tech table moved to `research/techs.rs`. Golden
  hash re-recorded (no save bump). Quartz checked: v4 band 25–50 blocks down; a geology test mines it.
- [x] **7.1b Power cable and the pole tool** (built, user request): the cable (block 71, hand recipe ×4 from 1
  iron plate + 2 copper wire, `recipes/wiring.rs`) is a `Kind::Pole` of stored tier 3 (`CABLE_TIER`, no save
  bump): cables link by touching (diagonals too) and to a pole within that pole's machine reach; machines hang
  on a pole first, else a cable within 2. `power_tools.rs` (ghost, wire, outlines, label, all presentation plus
  ordinary `PlaceBlock` actions): with a pole in hand and one near, a ghost pole stands at full link reach
  along the horizontal view, following the ground (`plan_pole`), and a click places it; holding places the next
  whenever the player has walked within `HOLD_REACH` of its spot; R (or crouch) frees it to go where aimed.
  With cables, a click hangs up to 64 down the aimed cell's column to the ground (crouch: one). Tests in
  `power_tools/tests.rs`, `factory/pole/tests.rs`. The cable wears the copper wire texture.
- [x] **7.2 Violet science and Mk4** (built): violet pack (310) and kit (311), assembler recipes 28–29; `PACKS` has four
  entries, labs a fourth slot (save 22); the cable's stored tier moved to 15 so tier 3 is the Mk4 pole. Mk4 items
  312–319 (hand recipe: Mk3 plus violet kits): belts 8/s, miners 6/s · 92 % · 90 kW, processors ×5, substation 32 · 16,
  lab ×4 · every 3rd unit free; boxes, pumps, quarries stop at Mk3. Techs 18–20; numbers pinned in `factory/tiers/tests.rs`.
- [x] **7.2b Bootstrap without raw ore, and timed hand crafting** (built): no hand recipe uses metal ore. Stone makes the
  furnace (Smelter); ore and fuel go in by hand for ingots; hand recipes (Materials) turn ingots into plates, rods, screws
  and wire; the first miner, belts, box, generator and poles are made of those. `crafting.rs`: `Action::Craft` queues an
  order (`CraftQueue` per player, max 12, save 23); its `plan` adds the part crafts the inventory can't cover, in
  order, and the materials are paid at once; a craft takes `hand_ticks` (90 + 30 per material, max 20 s); cancelling or
  leaving refunds it all (`Action::CancelCraft`, tag 26). UI: `ui/craftqueue.ts` chips, times in the build menu card.
  Tests: `crafting/tests.rs`, `recipes/tests.rs` (no raw ore), `tests.rs` scenarios.- [ ] **7.3 Solar power and accumulators.** Solar panel (block, 10 kW by day: `daytime.rs`, no state of its
  own) and accumulator (2×2×2, stores 10 MJ) as power sources and sinks in `power.rs`'s one balance.
  **Done when:** a test keeps a load running through a night on solar plus accumulators (sized).
- [ ] **7.4 Scanner Mk2.** Range 96, shows quartz (`prospect.rs`); Advanced Scanning tech.
- [ ] **7.5 Logic.** A sensor (box or belt fullness), a switch and a lamp signal (1 circuit, 1 plate each);
  a machine turns on or off by condition. New core state, so a save version and a design note first:
  keep it to "a sensor beside a machine's box or belt turns the pole's link on or off", data-driven.
- [ ] **7.6 Blueprints** (waits for the answer): select an area, copy it; place it as a ghost (rotate, see
  what is missing); ghosts from the build menu; a longer reach for ghosts.
- [ ] **7.7 Construction drones** (waits for the answer): a drone port (3×3 pad) builds ghosts from
  materials in boxes beside it (or on a network) and tears down what you mark; the drone item (1 motor,
  2 circuits, 2 steel plates). The machines that work sites are Milestone 8's.
- [ ] **7.8 Flight and helpers** (waits for the answer): the coal jetpack (4 steel plates, 2 motors,
  2 circuits; 10 s of thrust a coal) and the personal drone.
- [ ] **7.9 Feel, balance, cleanup.** Tips, research times played through, worst tick and wasm measured,
  README, CODEMAP; then move Milestone 8 in from the roadmap and ask its questions.

Open items from Milestone 3 (the user's to unblock; do them when they come up):

- **The relay (TURN):** left out for now (section 1). If friends can't connect, the user adds the TURN
  secrets (`docs/WORKFLOW.md` section 4); then test with `&relay` on both ends.
- **Instant feedback for your own edits** (was step 3.9): measured, not built. On one machine a client's
  action applies about 90 ms after it is made (the host's own, about 76 ms: `INPUT_DELAY` is 4 ticks);
  over the internet add the round trip, so roughly 110–170 ms. Build it only if placing and breaking feel
  laggy in real play: show your own block edits at once in the render cache, undo them if the core
  disagrees, never touch the core (`interaction.rs`).
- A client's every resync logs the tick and both hashes; turn any real one into a replayable test.

---

## 5. Roadmap after Milestone 7

Milestones 8–13 are in `docs/ROADMAP.md`; the tech tree through them is `docs/TECH_TREE.md` (concept)
and `docs/TECH_ERAS.md` (detail). Read them only when planning.

---

## 6. Open questions for the user

Ask these when the milestone that needs the answer comes up, not before. Record the answers in section 1
and adjust the steps.

| Needed by | Question |
|---|---|
| M7 | Blueprints: copied from what you built (select an area), drawn from scratch as ghosts, or both? Where do drones take materials from: boxes beside a drone port, or anywhere on a network? Flight: a coal jetpack now, an aluminium hover pack in M9: agreed? (Steps 7.6–7.8 wait.) |
| M7 | A colour-blind palette option for tier colours (pips and "Mk" text are there regardless)? Kits go in as one step at a time, never refunded: change? |
| M9 | Trains, trucks, or both? |
| M13 | Megaproject theme (orbital ring, space elevator, interstellar probe or other) and what completing it unlocks. |

---

## 7. Working in this repo

See `docs/WORKFLOW.md`: commands, browser verification, Windows gotchas, deploying, working with the user,
and the balance numbers. Read the section you need.

---

## 8. Change log of this plan

- **2026-09-25: Plan created; Milestones 1 and 2 done** (`3239215` to `3bf3f92`): fixed tick, core `Sim`,
  actions, state hash, saves; items, machines, belts, power, research, Mk2, tips. Tests 56 → 113, wasm
  120.3 KB. Lessons: measure wasm every step; the golden hash catches unintended core changes.
- **2026-09-26: Milestone 3 (Co-op) done** (pushed `c845691`): lockstep over bytes, join snapshots, player
  keys, avatars, the signalling Worker (STUN only), WebRTC, pings, timeouts, in-place resync; 3.9
  deferred. About 8 KB/s down, 4 up while idle. Lessons: heavy sort keys `#[inline(never)]`; edit files
  with the file tools (PowerShell reads UTF-8 as ANSI). Tests 113 → 127; wasm 131.5 KB gzipped.
- **2026-09-26: Play-test notes P1–P5 done** (`233371b`): Escape hints before pausing, the minimap, block
  timers (`sim/timers.rs`, save version 11: leaf decay, grass), tools (wear is the stack count). The `art`
  branch started (`docs/ART_HANDOVER.md`). Tests 127 → 141; wasm 138.0 KB gzipped.
- **2026-09-27: Milestone 4 (Reasons to explore) done** (`8d8ee22` to `a4fc499`): generator versions
  (version 1 pinned), saplings, rocks, limestone, quartz, glass, biomes (version 2 about 10 % slower),
  ores by biome with soil hints, scanner and core drill, map marks, day and night, sky and block light
  (meshing 0.36 → 0.49 ms per chunk, +25 % mesh memory), lamps; the art salvage and the build menu grid.
  Save version 12. Tests 141 → 183; wasm 138.0 → 161.4 KB gzipped.
- **2026-09-27: Belt placement upgrade** (`0bc0e47`): ramps derived from placement, drag-to-build lines
  (`belt_line.rs`), R rotates, research on T. Tests 183 → 193.
- **2026-09-27: Milestone 5 (Water and world shape) done** (`becc673` to `c1c61f9`): generator version 3
  (rare surface ore, depth bands, starter set), the sea and ponds, swimming, flowing water as core state
  (only the sea is endless), pumps, pipes, outlets, the quarry. Save versions 13–15. Tests 193 → 225;
  wasm 161.4 → 192.5 KB. Lessons: test the next step's scenario early (two-source water blocked one);
  PowerShell array patches misfire on a single pair.
- **2026-09-27: Power rework and 5.9** (`a682a28`): every miner needs power; generators store fuel energy
  and give only what is drawn (coal 270 kJ); the coal-miner-feeds-its-generator loop. Save version 16.
  Tests 225 → 226. Then Milestone 6 (Terraforming) was detailed.
- **2026-09-27: Torches and finding ore** (`11eee3b`): torches (`sim/torches.rs`), light sources as rows
  of `light::SOURCES`; generator version 4 (shallower bands, more exposed metal, two starter patches),
  the explored map and world map (M) with pins, ore guide, stained-soil readout. Tests 226 → 234; wasm
  192.5 → 201.6 KB (`to_lowercase` alone cost 14 KB, avoided).
- **2026-09-28: Sites in the core, wood, glass, lights, lifts** (`b33f98c` to `2d89533`): `factory/sites.rs`
  (save version 17; tags 21, 22), planks, sticks, ladders, sand → glass, torch reach 11, lamp 31
  (`bench_meshing` 0.56 ms per chunk), climbable lifts. Tests 234 → 240; wasm 204.5 KB.
- **2026-09-28: Tech tree and Industry** (user request): `docs/TECH_TREE.md` (the concept, upgrades, the content
  architecture) and `docs/TECH_ERAS.md` (per era). Industry became Milestone 6, terraforming moved to 8 (its
  sites step, built as 6.1, stays in the core); roadmap 7–13. No code changed.
- **2026-09-28: 6.1–6.6** (`8c22a45` on): tiers as data (`factory/tiers.rs`), `research::Unlock`, recipe
  categories, the content lint, upgrade kits (`Action::Upgrade` tag 23); one processing machine
  (`factory/process/`, save 18) with Masonry; multi-block footprints and the assembler (save 19); steel (the
  blast furnace with a byproduct port). Tests 240 → 270. Ctrl also crouches, and closing the tab mid-game asks.
- **2026-09-29: 6.7–6.7b:** blue science and Mk3 everything (labs with three pack slots, save 20; tiers for the rest, save 21). Tests 270 → 286.
- **2026-09-29: Milestone 6 (Industry) done** (6.8–6.10): steam (boiler, turbines as power sources in `power.rs`), the
  crusher, the silo, techs 14–16, four tips. Tests 286 → 296; wasm 232.7 KB gzipped. Lessons: write doc comments
  with the file tools or single-quoted here-strings (double-quoted PowerShell here-strings eat backticks); a plant
  test with a 405 kW load caught two turbine bugs a unit test would not. Milestone 7 moved in from the roadmap.
- **2026-09-29: Play-test notes after Milestone 6** (`Testing 5.ocworld`; pushed 28e1bf9): sort buttons for
  the backpack and boxes (`Action::SortInventory` tag 24, `SortBox` tag 25); boxes are lit by their cell
  (`world/boxlight.rs`); saved worlds open without their old guests (`Game::release_guests`), and a returning
  key replaces its old connection at once. Tests 296 → 303; wasm 235.2 KB gzipped.
- **2026-09-29: Box screen bug, step 7.1 and 7.1b:** a small box opened after a larger one showed the larger box's
  extra empty slots (`ui/inventory.ts` hides them). Step 7.1 built (arc furnace, silicon, circuits, Electronics tech).
  7.1b: the power cable block and a pole tool (ghost at full reach, hold to chain, R to place freely). Tests 305 → 316.
- **2026-09-29: Step 7.2** (violet science, Mk4 for eight families; save 22). Tests 316 → 323; wasm 235.5 KB gzipped.
- **2026-09-29: Step 7.2b** (bootstrap without raw ore; the hand-craft queue with timed crafts and automatic part crafts; save 23). Tests 323 → 336; wasm 239.3 KB gzipped.
