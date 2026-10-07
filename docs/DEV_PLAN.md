# OpenCraft development plan

**Status:** 2026-10-05 · Strategy controls built · Milestones 1–9 done and pushed (co-op tested across machines by the user; no TURN for
now) · **Milestone 10 (fluids and chemistry) is built (10.1–10.3 committed locally, 10.4–10.12 uncommitted, nothing pushed): the user reviews and tests it, then asks for the commit. Next: Milestone 11 (compute and photonics), step 11.1 (section 4)** · The `art` branch is superseded; art work
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
- **Milestones 7 and 8 are done** (`docs/CHANGELOG.md`): circuits, violet science, solar, logic, blueprints,
  construction drones, jetpack and gear; then terraforming by drone (dig, fill, flatten, tunnels).
- **Milestone 9 is done** (`docs/CHANGELOG_M9.md`): bauxite in far biomes, aluminium, bearings to far ground, trains,
  cargo drones and the hover pack.
- **Milestone 10 is built** (`docs/CHANGELOG_M10.md`; awaiting the user's review and commit): oil and canisters, plastics,
  diesel, electrolysis, ore washing, rivers and hydro, hoists, nuclear power, gold science, Mk5.
- **Next: Milestone 11: Compute and photonics** (section 4): chip fab, the data grid, AI datacenters and cooling, AI research
  and endless bonus techs, the optimizer, laser links, swarms, auto-routing, AI survey.
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
| 2026-10-06 | **Creative mode for new worlds** (user request, for testing everything): a world is normal or creative, chosen in the new-world form and never changed. Creative has every tech done and an "All items" tab in the inventory (click a stack, Shift-click one). Flying (F) already works in every world. |
| 2026-10-06 | **Analytics and machine efficiency** (user request): an Electricity and productivity screen (P) graphs the power the grids could give, the power used and asked for, and each item made a minute, over 1m to 3h, with coloured lines; every machine says how close to full speed it runs and what costs it the rest (low power, waiting for input, full output, no fuel...). **Presentation only:** history is not saved and starts when a world is opened. |
| 2026-10-06 | **One underpass, in tiers** (user request): entry and exit are the same item (pieces in a line facing the same way pair up by themselves, `factory/underpass.rs`); Mk1 to Mk4 reach 4 / 6 / 8 / 10 blocks at the speed of the belt of that Mk; a piece costs half its reach in belts of its Mk (a Mk1 pair is 4 belts). Dragging a belt line over a belt, machine or wall dives under it with the lowest underpass that reaches and is at least the belt's Mk; pairs the inventory lacks show red and are left out. The old exit block (23) still loads and is the same piece. No save-format change. |
| 2026-10-04 | **Terraforming is done by the drone ports** (no excavator machine); spoil goes into belts and boxes. **Far ground must be findable and reachable** (user): the player gets a rough bearing to the biome that holds an ore (step 9.3), and **trains** carry long-distance cargo (step 9.4; trucks are not planned). |

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

## 4. Now: Milestone 11: Compute and photonics

Milestones 6 (Industry), 7 (Electronics, blueprints, drones), 8 (Terraforming), 9 (Distance) and 10 (Fluids and chemistry)
are done; their step lists are in `docs/CHANGELOG.md` ("Milestones 6 to 8"), `docs/CHANGELOG_M9.md` and
`docs/CHANGELOG_M10.md`. Design: `docs/TECH_TREE.md` (sections 4 and 5) and `docs/TECH_ERAS.md` section 7.

**Rules that still bind every step:**
- `docs/TECH_TREE.md` section 8: tiers are data, processing machines are spec rows, recipes belong to
  categories, unlocks are one enum. No new `bool` per variant, no block per tier.
- Old saves keep loading: each format change bumps `SAVE_VERSION` (now 39) with a migration and a fixture
  test. Golden hashes are re-recorded only on purpose, noted in the step.
- New blocks and items append (the next free block is 98, item 379, tech 56, machine recipe 60). Each new look gets a
  placeholder layer (`tex::COUNT` is 246) and a `docs/ART_HANDOVER.md` request line.
- Fluids other than water ride belts as **canister items** (pipes stay water-only); empty canisters come back.

### Milestone 10 (Fluids and chemistry), built; the user reviews and tests it before it is committed

Oil and canisters, the pumpjack, refinery, cracker, chemical plant, diesel power, electrolysis, ore washing, the research
center, a terrain overhaul with rivers and water wheels (generator versions 6 and 7), hoists, nuclear power, gold science
and Mk5. Specs: `docs/CHANGELOG_M10.md`. **Measured:** 20 dry water wheels, 8 reactors, 8 refineries, 8 diesel generators,
4 winches and 4 riders asking the hoist rate each tick cost 19 µs a tick (worst 36 µs) on one grid (`bench_chemistry`,
ignored); water's worst tick is 0.09 ms. **Balance (to tune in play):** power per source: coal generator 60 kW, steam turbine
240 kW, water wheel up to 48 kW (free, but needs a river), diesel generator 400 kW, reactor 2 MW. Oil pays: a vein's pumpjack
(90 kW) fills about 22 canisters a minute and 3 of them refine into one diesel canister (40 MJ, the energy of 148 coal) plus
naphtha and heavy oil, so one pumpjack and one refinery (150 kW) feed several diesel generators. A fuel cell (4 uranium ore
and a steel plate, 300 MJ) runs a 100 kW load for 50 minutes; the reactor needs coolant, and a centrifuge draws 200 kW.
Washing turns a raw ore's 1.0 ingots into 2.0 for 60 kW and a pipe of water per washer. Mk5 costs 4 to 8 gold kits a
machine on top of the Mk4, so it stays a late-game spend.

### Milestone 11 steps

Goal: the factory learns compute. Chips come from chemistry (acid and pure water), datacenters turn megawatts and coolant
into **compute**, a second grid carries compute like power, and compute buys things: AI research, endless bonus techs, the
optimizer, drone swarms, auto-routing and AI survey. Laser links join far plants and datacenters by line of sight. Numbers
below are first guesses; tune in play (`docs/WORKFLOW.md` section 6). Order: chips, the data grid, datacenters and cooling,
what compute buys, laser links, helpers, cleanup.
- [ ] **11.1 Pure water, the chip fab and AI accelerators** (`factory/process/fab.rs`, `recipes/compute.rs`, `research/compute.rs`,
  `textures/compute.rs`; `Category::Fabrication`; techs Wafers, then Accelerators). **Pure water canister** = a chemical plant
  recipe (1 empty + 2 water + 1 sand → 1 pure water; pipes stay water-only). **Chip fab:** 4×4×3 clean room, 1 MW,
  `Pick::ByInput`, water inlet; **Wafer** 2 silicon + 1 acid canister + 1 pure water canister → 1 wafer + 2 empties back, 8 s;
  **AI accelerator** 2 wafers + 2 processors + 1 plastic → 1, 20 s. Hand recipe for the fab: 60 steel plates, 24 beams, 24 glass,
  16 processors, 24 circuits. Done when: a fab on power, fed by belt, makes accelerators; its status names what it waits for.
- [ ] **11.2 The data grid** (`factory/data.rs`; `wiring.rs` and `power.rs` are the model; fibre node block, tech Data Network). A
  second channel with the same node, link and balance code: fibre nodes link within 12 blocks, a consumer or producer
  attaches to the nearest node within 5, and each grid's compute (TF) is supply against demand with a satisfaction factor.
  Generalise the power grid by a `Channel` parameter rather than copying it; `Machine::describe` gets a compute line. Bumps
  `SAVE_VERSION` (node records; none in older saves). Done when: a fibre line between two test machines carries a supply
  to a demand and a shortage slows the consumer (tests); the model draws thin cable boxes.
- [ ] **11.3 AI datacenter** (`process/datacenter.rs`, `Energy::Compute`; block, textures, tech Datacenters after Wafers, Data
  Network and Nuclear Power or Diesel Power). 4×4×3 hall, 3 MW electric, 100 TF at full power, supplying the data grid. **Heat**
  uses the reactor's model (coolant in `steam.water`, heat in `progress`, `Status::Overheated` with hysteresis), so no new
  concept: without coolant it heats up and stops. Hand recipe 40 beams, 20 concrete, 16 accelerators, 32 copper wire. Mk tiers
  add racks (+50% compute for +50% power and heat). Done when: a datacenter on a grid with enough power and water gives
  compute, shuts down when overheated, and survives a save.
- [ ] **11.4 Cooling tower and the coolant loop** (`process/tower.rs`; 3×3×5, block, tech Cooling). Water in and out by pipes:
  a datacenter piped to a tower sends its hot water there and the tower returns it cool, losing only a trickle (open-loop
  datacenters keep spending water as in 11.3, so a river or pump is the other choice). Hand recipe: concrete, pipes, motors.
  Done when: a looped datacenter runs an hour of ticks without draining the pump's pool (test), and its status says
  "loop closed".
- [ ] **11.5 AI lab and compute research** (`process/ailab.rs` as a `Pick::Research` variant, `research.rs` compute cost). Techs
  gain an optional compute cost beside packs; an AI lab (2×2×2, on the data grid) spends compute per unit and the packs go
  further (every 2nd unit free, as at center Mk3). Late techs (Auto-Routing, AI Survey) need compute. Done when: a tech with
  a compute cost finishes only on a grid with a datacenter, and the tech tree says what is missing (as `needs_center` does).
- [ ] **11.6 Endless bonus techs** (`research/bonus.rs`, `Unlock::Bonus(kind)`; techs AI Research, then Mining, Machine Speed and
  Drone Speed). A bonus tech has levels, costs compute only, the cost grows 25% a level and each level adds 3% (mining draw,
  processor speed or drone speed) through the one place that computes rates (TECH_TREE rule 5). The level is research state
  (a save bump). Done when: levels repeat without end and rates in tests rise by the stated percentage.
- [ ] **11.7 Optimizer node** (`process/optimizer.rs`; tech Optimizer). A consumer on the data grid (20 TF): machines of the grid
  within 16 blocks run 25% faster while it gets its compute. Not stacking (the best node in range counts). Done when: a smelter
  in range shows ×1.25 and loses it in a compute shortage.
- [ ] **11.8 Laser power links** (`factory/laser.rs`; emitter and receiver blocks, tech Photonics). An emitter and a receiver
  within 128 blocks (Mk tiers to 512) with a clear line become one edge of the power grid at 90% efficiency; the edge is
  rechecked only when a block in its path changes (`block_anywhere`; glass lets the beam through) and the path is cached.
  Draw: an additive beam quad that flickers when blocked. Done when: two islands of power join through a beam, a block in the
  path cuts it and breaking that block restores it (tests), with no per-tick ray cost.
- [ ] **11.9 Mirrors and data beams** (`laser.rs`, mirror block). A mirror turns a beam 90° so lines can go round corners; a beam
  can carry the data channel instead of power (the receiver's mode is chosen in its panel). Done when: a beam through two
  mirrors carries compute between two datacenter sites.
- [ ] **11.10 Swarm hub and drone swarms** (`process/hub.rs`; tech Drone Swarms). A hub on the data grid beside a port adds 50% to
  its fleet and costs compute. Done when: a hub raises a Mk3 port's fleet from 16 to 24 and it drops back without compute.
- [ ] **11.11 Auto-routing** (`route.rs`, `Feature::AutoRoute`; tech Auto-Routing). Drag from a machine's output to another's
  input: an A* over voxels (a query, never core state) finds a belt route with lifts and underpasses and shows it as ghosts for
  drones or hands (`belt_line.rs` draws them). Done when: a route across a gap places ghosts in tests, never edits the world,
  and a blocked target says so.
- [ ] **11.12 AI survey** (`survey.rs`, a prospecting query; tech AI Survey). Predicts deposits within 256 blocks from the surface
  hints and the scanner's records (likely ore by stain and bearing, not the full truth), drawn as faint marks on the map.
  Queries only: the state hash never moves. Done when: it lists a known deposit's stain in a test and the map shows it.
- [ ] **11.13 Cleanup:** tips, balance, worst tick (a datacenter plant bench), README, CODEMAP; then move Milestone 12 (Orbit)
  in from the roadmap. The user reviews and tests the milestone before it is committed.

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

## 5. Roadmap after Milestone 11

Milestones 12–13 are in `docs/ROADMAP.md`; the tech tree through them is `docs/TECH_TREE.md` (concept)
and `docs/TECH_ERAS.md` (detail). Read them only when planning.

---

## 6. Open questions for the user

Ask these when the milestone that needs the answer comes up, not before. Record the answers in section 1
and adjust the steps.

| Needed by | Question |
|---|---|
| M7 | A colour-blind palette option for tier colours (pips and "Mk" text are there regardless)? Kits go in as one step at a time, never refunded: change? |
| M10 | Rivers and the new landforms are on new worlds only (generator version 7; built that way in 10.8): existing worlds keep their ground. Acceptable, or should they wait for a world-settings screen? Also: should far deposits seen once get a map pin (9.3 left it as a bearing only)? |
| M13 | Megaproject theme (orbital ring, space elevator, interstellar probe or other) and what completing it unlocks. |

---

## 7. Working in this repo

See `docs/WORKFLOW.md`: commands, browser verification, Windows gotchas, deploying, working with the user,
and the balance numbers. Read the section you need.

---

## 8. Recent changes

- **2026-10-07: Tools move whole** (user bug report: a worn shovel in a box rode a belt as 87 shovels). A tool's stack count is its uses, so every one-at-a-time mover split it. Now `tools::lot(item, count)` is the piece a mover takes: a tool's whole count, 1 of anything else. Belt items and a router's held item carry `n` (`BeltItem::n`, `Router::held_n`); `Buffer::feed` takes a lot, `deliver` / `Sinks::accept` hand `n` over all-or-nothing (a sink with no room for every use makes the belt wait); the recycler takes a lot too. Save version 39 (belt and router counts; older saves read 1); golden hash re-recorded. Boxes and inventories still pool worn tools of one kind into stacks, so a belt carries one tool per slot.
- **2026-10-07: Blast furnace on crushed iron** (user request). `BLAST_CRUSHED_RECIPE` (machine recipe 59, 2 crushed iron + coal + quicklime, same steel and slag), unlocked with Ore Crushing; the furnace is now `Pick::ByInput` (no recipe to choose, it takes ore or crushed iron and makes whichever it holds a full batch of; `recipe_using_if` and `Processor::next`). First version kept `Pick::Chosen` and the user could not get crushed iron in: a furnace's chosen recipe is on the old iron-ore recipe. A saved furnace's chosen recipe is simply ignored. The same for washed iron: `BLAST_WASHED_RECIPE` (60, unlocked with Ore Washing). No save bump. Tests 628 → 630. Next machine recipe 61.
- **2026-10-06: The recycler** (user request). Block 97, a 2×2×2 `Pick::Recycle` processor (`factory/process/recycler.rs`, six in-hatches, two out, 90 kW), the coin (item 378, stack 1024, `item::COIN`; not offered in creative), the Recycling tech (55, after Blue Science), `Processor::owed` (millicoins, saved for recyclers only, so no save bump: only new worlds hold one). What an item pays is `recipes/recycling.rs`: a const-built table over `RECIPES` and `MACHINE_RECIPES`, no per-item list. Raw 1 coin; a batch is worth `STEP` (2) × its inputs, shared over what it makes (millicoin precision, so a plank is ½); the cheapest recipe wins, so the route never changes the worth; `WASTE` (slag, tailings) and what is salvaged from only waste pay 1; tools go in whole (their count is their uses). `process/intake.rs` split out of `process/mod.rs`. Golden hash re-recorded (a new tech row). Tests 611 → 628. Next free block 98, item 379, texture 246, tech 56.
- **2026-10-06: Leaf despawn lag** (user report). Measured in the user's save: each decaying leaf remeshed up to 8 chunks at once, marked about 36 chunks dirty for light (the 32-block margin, everything below) and dropped their box-light cache entries, so about 70–80 ms a decay. Now `light::reach` limits the dirty chunks to what an edit can change (none for equal light behaviour, the sky column for sky-only blocks like leaves), leaf timers use `set_block_anywhere_later` (remesh waits for the streaming budget), and the box-light cache goes stale and is relit in `work_step` instead of in `Game::update`. About 28 ms of work a decay left, spread by the 6 ms browser budget; `update` 0.2 ms. No save bump (render cache only).
- **2026-10-06: Analytics and efficiency** (user request). `analytics/` (history rings, per-machine averages), `factory/efficiency.rs` (read-only samples and power totals), `SimEvent::Produced` (miners, quarries, processors, pumpjacks), `api/analytics.rs`, `ui/analytics.ts` (P) and `ui/analytics-chart.ts`; machine panels and `target_detail` show an efficiency line. No save bump, the core is untouched. `main.ts` now opens its screens from one table. Tests 590 → 596.
- **2026-10-06: One underpass in tiers** (user request). `factory/underpass.rs` (`UNDERPASS_SPAN`, `derive_passes` rederives entry and exit at every relink; block 23 is a non-placeable legacy block), a `Family` row and items `UNDERPASS_MK2..MK4` (375–377), belt-priced recipes, tech unlocks beside the belt tiers; `belt_line/pass.rs` makes dragged lines dive under obstacles (red when unaffordable or too wide). No save bump. Tests 596 → 609.
- **2026-10-06: Stack controls** (user request). Right-click takes half a stack (rounded up), again adds half of what is left to the held stack; Shift-right-click with a stack held puts one item down, outside the screen it throws one; with an empty hand it still moves every stack of that item. Works on box slots too; tools move whole. New actions `RightClickSlot`, `RightClickBox`, `ThrowCursor` (tags 48–50, no save bump). `Sim::apply` moved to `action/apply.rs` (`action.rs` was over budget). Tests 588 → 590.
- **2026-10-06: Creative mode.** `mode.rs` (`Mode`, `Sim::mode`, `creative_items`), `api/creative.rs`, `ui/creative.ts`; a world's mode is saved (save version 38) and hashed, so co-op peers agree; the new-world form picks it. Golden hash re-recorded on purpose (the mode byte). Tests 583 → 588.
- **2026-10-05: Milestone 10 cleanup (step 10.12).** `bench_chemistry` (19 µs a tick, worst 36 µs), the balance note, README section on fluids and chemistry, the Milestone 10 step list moved to `docs/CHANGELOG_M10.md`, Milestone 11 moved in from the roadmap with a step list. No code change besides the ignored bench. Milestone 10 awaits the user's review and commit.
- **2026-10-05: Gold science and Mk5 (step 10.11).** Gold pack and kit (366–367), Mk5 items (368–374), techs Gold Science (53) and Mk5 Machines (54), a fifth pack in `PACKS`, three hints. No save bump. Tests 581 → 583. Next free block 97, item 375, texture 243, machine recipe 59, tech 55.
- **2026-10-05: Nuclear power (step 10.10).** Blocks 95–96, item 365, the Nuclear Power tech (52): centrifuge and a water-cooled reactor that shuts down when it overheats. No save bump. Tests 574 → 581. Next free block 97, item 366, texture 241, machine recipe 57.
- **2026-10-05: Hoists (step 10.9).** Blocks 93–94, the Hoists tech (51): shafts that a powered winch turns into a 9 blocks/s lift. No save bump. Tests 570 → 574. Next free block 95.
- **2026-10-05: Terrain overhaul and hydro (step 10.8).** Generator version 7 (rivers, lakes, mesas, bigger ranges: `worldgen/rivers.rs`, `landform.rs`), block 92 and the Hydropower tech (50): the water wheel. Golden hash re-recorded, no save bump. Tests 558 → 570.
- **2026-10-05: Steps 10.1–10.7** (new ground, canisters and pumpjack, refinery, chemical plant, diesel and electrolysis, ore washing, research center): see section 4, tests 527 → 558.
Earlier entries live in [CHANGELOG.md](CHANGELOG.md).
