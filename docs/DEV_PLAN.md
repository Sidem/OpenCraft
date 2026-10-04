# OpenCraft development plan

**Status:** 2026-10-04 · Strategy controls built · Milestones 1–7 done (M7 committed locally, not pushed; co-op tested across machines by the user; no TURN for
now) · **Now: Milestone 8 (terraforming): planner, drone earthworks and tests built (steps 8.1–8.3, uncommitted); next 8.4 tunnels (to decide) and 8.5 cleanup** · The `art` branch is superseded; art work
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
| 2026-10-03 | **Blueprints are both** copied from built areas and planned as ghosts; ghosts are core state. **Drones take materials from boxes beside their port.** **Drones must be hard and rewarding:** a long research ladder on violet packs and several new parts, not three cheap techs. **Coal jetpack in M7**, hover pack in M9. |

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
- **Prospecting** (`prospect.rs`, `ui/prospect.ts`): the scanner lists deposits within 48 blocks (Mk2: 96) (ore,
  tier, live bearing and distance, depth, size band); the core drill (3 s hold) gives a column's exact
  figures. Queries only: no actions, the state hash never moves. Devices are tools that never wear.
- **Size:** about 261 KB gzipped in total (wasm 201.6 KB, JS 51.1 KB, CSS 6.4 KB; `vite build`).

### Known limitations and technical debt

1. Only the local player runs hand interaction; co-op relays mining/using gestures for avatar animation
   alongside body states, and applies the resulting actions through lockstep.
2. Two tabs on the same world overwrite each other's saves (the later save wins).
3. ~~Veins and lodes can only be found by digging~~: fixed by prospecting (4.6) and map marks (4.7).
4. In version 1 and 2 worlds the outcrop nearest spawn may be buried (version 3 has a starter set).
5. TypeScript mirrors a few engine constants: the 6 floats per sound event, and the
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
- Old saves keep loading: each format change bumps `SAVE_VERSION` (now 28) with a migration and a fixture
  test. Golden hashes are re-recorded only on purpose, noted in the step.
- New blocks and items append (the next free block is 76, item 332). Each new look gets a placeholder
  layer (`tex::COUNT` is 177) and a `docs/ART_HANDOVER.md` request line.

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
first, then logic, then the builders (blueprints, drones, flight). Steps 7.6–7.8 follow the user's answers
of 2026-10-03 (section 1).

- [x] **7.1–7.6 built** (full specs moved to `docs/CHANGELOG.md`, "Milestone 7 steps 7.1 to 7.6"): 7.1 arc furnace, silicon,
  circuit; 7.1b power cable; 7.2 violet science and Mk4; 7.2b bootstrap without raw ore and the timed hand-craft queue
  (save 23); 7.3 solar panel and accumulator; 7.3b wiring by hand (save 24); 7.4 Scanner Mk2; 7.5 the sensor (save 25);
  7.6a ghosts (save 26) and 7.6b blueprints.- [x] **7.7 Construction drones: the long way** (user, 2026-10-03: drones must be hard to reach and very rewarding; more
  research and parts). Gate: a ladder of techs on violet packs, none cheap: Robotics (actuators and servo parts) →
  Drone Power (a drone cell, made of a battery-like part from circuits, copper, steel and a motor) → Navigation (a
  processor-based guidance module, so after Processors) → Construction Drones → Swarm Logistics (more drones per port,
  a longer range). The drone is built in an assembler from several intermediates, none raw ore; the drone port is a big
  multi-block (3×3 pad) on power. Drones take materials from boxes beside the port (a later tech may extend this to a
  network). Steps: **7.7a** the part chain and techs, **7.7b** the drone port and its supply boxes, **7.7c** drones
  flying out and building ghosts (one at a time, visible), **7.7d** tear-down marks, **7.7e** swarm upgrades. Numbers
  go in `docs/TECH_ERAS.md` when 7.7a starts.
  **7.7a built (no save bump; golden hash re-recorded):** five techs on all four packs after Violet Science: Processors
  (100 units × 30 s) → Robotics (140 × 30, also needs Mk4 Machines, so Mk4 Logistics first) → Drone Power (160 × 35) and
  Navigation (180 × 35, needs Processors and Robotics) → Construction Drones (240 × 40): twelve techs in front of the
  drone. Six assembler-only parts (`item.rs` 321–326, `MACHINE_RECIPES` 31–36, `DRONE_RECIPES`): processor, servo,
  actuator, drone cell, guidance module, drone. Numbers in `docs/TECH_ERAS.md` section 3; icons `textures/robotics.rs`.
  **7.7b–e built (save 27, merged into 28):** the drone port (block 75, `factory/process/hangar.rs`: a 3×3×1 processor spec,
  `Pick::Hangar`, 40/60/90/140 kW only while drones are out) keeps drones as its input buffer, so belts, panel, saves and
  breaking needed nothing new; tiers Mk1–Mk4 by kits (`Swarm Logistics`, tech 29): reach 32/48/64/96 blocks, fleet 4/8/12/16.
  The core `drones/` (`Drones` in `Sim`, saved) launches a drone every 30 ticks per powered port at the nearest ghost in
  reach that no drone is on: a build takes its item from a storage box touching the pad (`factory/ports.rs`), flies at
  8 blocks a second (f64, `+ - * / sqrt` only), works 1 s and places it through `put_block` (the same code a player runs);
  a tear-down mark (`Action::MarkRemoval`, tag 36: a ghost of air) works as long as bare hands take (1–4 s) and breaks it
  through `dismantle`, the drops going into the boxes. Breaking a port brings its drones back as items; a drone whose port
  is gone drops as an item. `break_block`/`place_block` were split into `dismantle`/`put_block` for this. Presentation:
  `drone_view.rs` (drones as boxes), ghost mode left-click marks or unmarks (`update_marking`, red outlines, label).
  Port recipe: 24 steel plates, 12 circuits, 2 processors, 4 motors.
- [x] **7.8 Flight and helpers** (answer: coal jetpack now, hover pack in M9): the jetpack (4 steel plates, 2 motors,
  2 circuits; 10 s of thrust a coal) and the personal drone (which, like the construction drone, needs the long ladder).
  **Built (save 28):** `helpers/` (core, per player in `PlayerCore.helpers`, saved): `Action::Jetpack { on }` (tag 37)
  burns a coal from the pack per 600 ticks of thrust (the pack stands in for a fuel slot) and the body lifts at 12 m/s²
  net up to 6 m/s while `Player::thrust` (set from the core by `authority.rs`); the hands send it when jump is held in the
  air (`helper_hands.rs`). `Action::Fetch { item, at }` (tag 38): with the personal drone in the pack, Y sends it to the
  nearest box within 32 blocks holding the held item; a round trip at 12 blocks a second (at least 1 s) brings a stack.
  Items 330/331 (tools of `DEVICE_TIER`), techs Jetpack (needs Robotics, 120 × 30 s) and Personal Drone (needs Construction
  Drones, 160 × 35 s); recipes: jetpack 4 steel plates, 2 motors, 2 circuits; personal drone 1 drone and 2 circuits.
- [x] **7.9 Feel, balance, cleanup.** Tips, research times played through, worst tick and wasm measured,
  README, CODEMAP; then move Milestone 8 in from the roadmap and ask its questions.
  **Done:** three new tips (ghosts, drones, fly and fetch), `bench_drones` (2,000 ghosts and a port: 3.6 µs a tick, worst
  61 µs), wasm 312.6 KB gzipped (`vite build`: wasm 832.6 KB raw; 241 KB at step 7.3: most of the growth came with the
  textures, blueprints, strategy camera and avatars; the drones add icons and about 10 KB of code), 431 tests (4 ignored benchmarks).
  Research times follow `docs/TECH_ERAS.md` section 3. Milestone 8 questions are open (see the status line).

### Milestone 8 (Terraforming, era 5): in progress

Design from `docs/TECH_ERAS.md` section 4. **The user's answers (2026-10-04):** the earthworks are done by the
**drone ports** of Milestone 7 (no separate excavator machine); spoil goes into **belts and boxes**, and fill sites in
range are served from the same boxes first. **Built:** sites in the core (`factory/sites.rs`: `Site`, `Job` dig, fill or
flatten, `MarkSite` / `RemoveSite`, `survey_site`, `api/sites.rs`; saved); the drone ports work them
(`drones/earthworks.rs`).

- [x] **8.1 The planner** (`site_hands.rs`, `ui/site.ts`): item 332 (3 iron plates, 4 copper wire, 2 glass, a circuit),
  tech Earthworks (needs Construction Drones, 140 × 35 s, all four packs). Right-click a corner block, then the opposite
  one (reach 64); the site panel picks the job and the level and shows the survey (blocks to dig and fill, ore, trees,
  water, unseen columns, spoil); right-click a marked site to look at it or remove it. Outlines (job colour for the
  extent, white for the level, cyan for the box being marked) go into `placement_box`.
- [x] **8.2 Drones work sites:** a port picks the first cell in its reach that needs work (cut layers top down, then fill
  layers bottom up) after ghosts of equal distance; a cut breaks the block by hand and puts the drops into the pad's
  boxes (it waits while they are full); a fill brings dirt (top layer) or stone, dirt, sand, grass from the boxes and
  places it where the cell is free, the cells above are free and the ground below is solid; so dug ground fills other
  sites from the same boxes. A finished site removes itself.
- [x] **8.3 Fill and flatten:** the same cell scheme (a flatten is a dig and a fill in one site).
- [ ] **8.4 Tunnels:** two points, 1×2, 3×3 or 5×5, slopes up to 1 in 2, worked by drones from the tunnel face; breaking
  into water waits. Open: whether to build it (reuses the cell scheme).
- [ ] **8.5 Cleanup:** worst tick with many drones on a 64 × 64 flatten by the sea, minimap squares for sites, a tip,
  README (planner keys done), colour-blind check of the outline colours.

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
| M7 | A colour-blind palette option for tier colours (pips and "Mk" text are there regardless)? Kits go in as one step at a time, never refunded: change? |
| M9 | Trains, trucks, or both? |
| M13 | Megaproject theme (orbital ring, space elevator, interstellar probe or other) and what completing it unlocks. |

---

## 7. Working in this repo

See `docs/WORKFLOW.md`: commands, browser verification, Windows gotchas, deploying, working with the user,
and the balance numbers. Read the section you need.

---

## 8. Recent changes

- **2026-10-04: Steam through pipes.** A boiler's faces now have named ports (`BOILER_PORTS` in `process/specs.rs`; `Port::cell` / `Which` picks one cell of a side, `Role::Water` and `Role::Steam` are pipe roles): on each of back, left and right one cell is a coal inlet (belt) and the other a water inlet (pipe), and the two front cells are steam outlets; a turbine has a steam inlet on both cells of its right end (the end away from its generator). A pipe joins a port only on that face (`Processor::pipe_ports`); steam no longer passes between touching machines. Turbines take steam from the boilers on the same pipe network (`steam::link`, still two per boiler, lower index first). A network is `Fluid::Water`, `Steam` or `Mixed` (both: works for neither, pipes turn red, machines say so). Water pipes are blue banded and steam pipes pale and red banded (`textures/piping.rs`, `tex::PIPE_WATER/PIPE_STEAM`, `COUNT` 180). `process/boiler_view.rs` became `steam_view.rs` (coal chutes, water and steam nipples on flanges, gauge; the turbine's inlets); the boiler's firebox door moved up the front. No save or hash change, but saves with a turbine standing against a boiler lose that link and need pipes. Tests 438 → 441.

- **2026-10-04: Boiler connections drawn.** The boiler lost its generic belt hatches: coal chutes (dark funnel, coal on top) mark free belt inlets, a steel flange marks every pipe touching the tank (`process/boiler_view.rs`, from `Steam::taps`), and a water gauge on the front fills with the water held. Pipes and pumps beside a boiler now draw an arm into it (`Pipework::arms`). A boiler now draws water from every network touching it (`Steam::nets`; it used to listen to the first one only, so a second pump on its own network did nothing). Presentation plus a derived-state fix: no save or hash change. Tests 435 → 438.

- **2026-10-04: Terraforming by drone (steps 8.1–8.3).** The Planner (item 332, tech Earthworks, 140 × 35 s) marks two corners (reach 64) and opens the site panel (`ui/site.ts`: job, level, survey, remove). Drone ports now work sites (`drones/earthworks.rs`): cuts break by hand into the pad's boxes (they wait while full), fills bring dirt or stone from the boxes; a finished site removes itself. `Site::done` is an unsaved cursor cache. `Research` got a manual `Default` (33 techs). Tests 431 → 435, golden hash re-recorded (a tech was added), no save bump. Wasm size not re-measured.

Earlier entries live in [CHANGELOG.md](CHANGELOG.md).

- **2026-10-03: Drones, jetpack, personal drone (steps 7.7b–7.9, save 28).** The drone port (a 3×3 pad on power, tiers by kits) keeps drones; they fly out one at a time from powered ports to build your ghosts from the storage boxes touching the pad and to tear down blocks you mark (left-click in ghost mode), putting the drops in the boxes. A coal jetpack (hold jump in the air) and a personal drone (Y fetches the held item from a box within 32 blocks). Milestone 7 is done.
- **2026-10-03: The drone ladder (step 7.7a).** Five violet techs (Processors, Robotics, Drone Power, Navigation, Construction Drones; twelve techs deep from the start of Violet Science, 920 units of all four packs for the last five) and six assembler-only parts that make a drone: processor, servo, actuator, drone cell, guidance module, drone (about 14 motors' worth of circuits and steel each). No save bump; golden hash re-recorded. The port and the flying come next.
- **2026-10-03: Blueprints (step 7.6b).** Ghost mode gains area copy: Z marks two corners, Enter copies the machines in the box (`blueprint/`: block, facing, tier, offset; turned in quarter turns, multi-block machines keep their shape), the use button stamps them as ghosts through the new `PlantGhost` action, and the L panel (`ui/blueprints.ts`) lists, renames, holds and deletes blueprints. They live in the browser per world like pins, not in the save. Settings and plain blocks are not copied yet.
- **2026-10-03: Ghosts (step 7.6a, save 26).** Planned blocks and machines are core state (`ghosts.rs`: a cell, block, facing, tier; `PlaceGhost` / `RemoveGhost`, tags 33/34), so co-op peers and later drones share them. B toggles ghost mode (`ghost_mode.rs`, presentation): right-click plants the held block's ghost for free or removes the aimed one, R turns it, reach 16, no mining, cyan outlines and a needs list. Placing the real block on a ghost uses its facing and clears it. Golden hash re-recorded.

- **2026-10-03: Logic (step 7.5, save 25).** The sensor block (Logic tech, violet science): reads the box, silo or belt behind it and cuts the power wire of the machine in front by a rule (switch, or fullness thresholds with hysteresis); right-click steps rules, R turns, a lamp shows the state. New `Kind::Sensor` with its list saved after the quarries, `Action::SetSensor`, `Hooked::off` / `Power::off` (readouts say "Switched off by a sensor"). Tests: 405 pass (5 new), golden hash re-recorded. Not tried in the browser yet.

- **2026-10-03: Scanner Mk2 (step 7.4).** Advanced Scanning (r g b v) unlocks the Scanner Mk2 (item 320: a scanner, 3 circuits and 2 steel plates by hand): it lists deposits within 96 blocks instead of 48. Range is data (`prospect::SCANNERS`); the panel shows the range of the scan taken. Tests: 400 pass (one new), golden hash re-recorded (a tech was added), no save bump.

- **2026-10-03: Belts carry bodies; frame-rate fix** (user report: 60 fps in some places, about 16 in others, with a save). Cause, found by loading the save in a headless release test: the light cache for boxes (`world/boxlight.rs`) held 24 chunks, evicted oldest first, and the save's machines and belt items in view span 27, so every frame lit all 27 chunks again (about 1.6 ms each, 45 ms a frame; places with 24 or fewer chunks ran fine). The cache now holds 128 chunks (4 MB at most) and evicts the least recently read; that frame costs 0.3 ms. A body on the ground in a belt's cell is carried along at the belt's speed (`Factory::conveyor_at`, `Player::conveyor`, set by `authority.rs`; collision still applies, a crouched body stops at an edge, footsteps ignore the carry, lifts are still climbed). Presentation/authority only: no save or core change. Tests 393 → 399.

- **2026-10-02: Strategy controls** (user request, approved after local review): V cycles first/third/overhead follow; G frees or follows, Home follows; WASD pans, wheel zooms, right-click walks, X stops, Ctrl-right-click uses/places within body reach. Smoothed ground height, zoom and view transitions; bounded routes around obstacles with one-block jumps and safe refusal. Only bodies discover columns; the camera reloads known 3D terrain and covers unknown ground with fog. Save-free development `/?preview`; no save/core format changes. Tests 378 → 392.

- **2026-10-02: Comfort settings** (user felt nauseous): a "Comfort settings" section in the pause menu (`ui/comfort.ts`, `comfort/settings.ts`, localStorage): field of view, mouse sensitivity, a movement vignette (`ui/vignette.ts`), a bolder crosshair with style, size, thickness and opacity; the eye eases between standing and crouching (`camera.rs`). Presentation only. Tests 364 → 366.
- **2026-10-02: Third-person view and Kestrel avatar** (comfort option, V toggles): eased, collision-tested right-shoulder camera; camera-ray targeting still checks hand reach and visibility. Procedural ivory/teal survey robot with articulated walking, crouching, airborne/water and working poses, held items and correct crosshair-facing head; pickaxe points and axe edges lead the mining stroke. Co-op relays pose flags and velocity; box stride comes from the engine. Save-free `/character-preview.html` for review. Tests 366 → 378; saves and core hashes unchanged.
