# OpenCraft development plan

**Status:** 2026-09-28 · Milestones 1–5 done (co-op tested across machines by the user; no TURN for
now) · Milestone 6 (Terraforming): 6.1 done · **Next up: step 6.2 (the planner)** · The `art` branch is superseded;
art work continues from `main` (`docs/ART_HANDOVER.md`).

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
- **Now: Milestone 6: Terraforming** (section 4): a planner tool marks sites, excavators with work drones
  dig, fill, flatten and tunnel.
- **Art** (`docs/ART_HANDOVER.md`) says who owns which looks. When gameplay needs a new look, append a
  `tex` layer with a plain placeholder pattern and add a line to that file's request list.

Before you change code:

1. Read sections 0–4 of this file (section 3.1 carefully), then `docs/CODEMAP.md`, then the nested
   `CLAUDE.md` of the area you work in. Skim README.md only if you need the player's view.
2. Run `npm run build:wasm` (if `web/src/wasm` is missing) and `npm run check` to confirm a green baseline
   (238 engine tests).
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
| 2026-09-27 | **Water and world shape before terraforming** (Milestone 5): sea level, lakes, swimming, pumps and pipes. Rivers later (now Milestone 9). |
| 2026-09-27 | **Water flow is limited and deterministic:** a Minecraft-like spread, event-driven and capped per tick; no volume simulation. |
| 2026-09-27 | **Surface ore is rare and meaningful:** about 10× fewer exposed outcrops, only on bare rock; a starter set near spawn; ores in depth bands. Vein and lode counts stay. |
| 2026-09-27 | **A quarry in Milestone 5:** rock and soil get automated extraction that really digs a pit (the first excavator); it must be intuitive and satisfying to watch. |
| 2026-09-27 | **Order after Milestone 5: terraforming, then blueprints and construction drones, then transport** (personal flight first: jetpacks or hovering, and building from further away). The player should feel a steady, intuitive and fun growth of capability: building by hand → planning → automatic assembly. |
| 2026-09-27 | **Finding starter ore made easier** (play-test: iron under rusty soil was too deep to find by hand). Generator version 4 for new worlds: shallower bands, coal, iron and copper exposed 1.5× as often on bare rock, two starter patches of each; a world map (M) of explored ground with pins; an ore guide with depth bands; stained soil tells what lies how deep. Mute moved to K. |
| 2026-09-28 | **Glass from sand; brighter lights.** The smelter makes glass from sand (1 → 1); quartz gives 2. Torches reach 11 (full within 4), lamps 31 (full within 17): the light margin is 32, the most the 3×3×3 field allows. |
| 2026-09-28 | **Processed wood and ladders.** Logs saw into planks, planks into sticks; sticks make tools, torches, poles and ladders (a see-through frame: jump climbs, crouch descends, idle holds). |
| 2026-09-27 | **Miners need power; generators burn only what is used.** The first loop is a miner on coal feeding the generator that powers it. Energy is stored per generator (kJ); one coal (270 kJ) runs a Mk1 (5 kW) long enough to mine about 32 coal. The smelter stays a burner. |

### Proposed, not yet confirmed by the user

- Old saves keep loading across format changes where a migration is cheap. Every save since version 1
  still loads.
- Terraforming (Milestone 6): one excavator machine with work drones rather than separate grader, filler
  and borer machines; dug ground is material that fills other sites first and goes onto belts after (a
  dump is a fill site); ore in a site is cut like hand mining (lossy), with a warning when planning.

---

## 2. Where the code stands (after Milestone 5)

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

## 4. Now: Milestone 6: Terraforming

The user decided on 2026-09-27 (section 1): terraforming first, then blueprints and drones (Milestone 7),
then transport and personal flight (Milestone 8). Across all three the player's reach should grow
steadily and intuitively: building by hand → planning an area → machines doing the work.

**Goal:** the player marks an area with a planner tool, picks a job (dig down to, fill up to, flatten to
a height) and excavators do it with small work drones, visibly and block by block. Dug ground is material:
it fills other sites first, and the rest goes onto belts and into boxes. A "dump" is just a fill site.
Tunnels come last. The quarry (5.8) stays as the simple early machine; the excavator is its big brother.

**Facts from Milestone 5 this builds on:**
- The sea is the only endless water: sources form only at or below `SEA_LEVEL` (62). Pumps can't lower
  the sea or anything open to it. A cut below sea level floods when it opens to the sea; a fill placed
  into water is solid ground and the sea doesn't come back (land can be reclaimed).
- Water checks are capped per tick (`MAX_WATER_UPDATES`); mass edits next to water queue many.
- The quarry (`factory/quarry/`): cells in a fixed order, `*_anywhere` edits, `QUARRIABLE`, an output
  buffer that feeds belts, waiting while flooded, `Factory.changed` for water and timers, cracks and
  sounds. Reuse its patterns (and `survey`), don't generalise it.
- Power: generators give only what is drawn; every miner needs power. Generator version 3 is frozen.

**Rules for all steps:** sites and their progress are core state (saved, hashed, changed only by actions
and ticks). Cell order never depends on loaded chunks. Big edits go through `set_block_anywhere_later`
so remeshing stays within the streaming budget. Measure the worst tick for a 64 × 64 site.

- [x] **6.1 Sites in the core.** `factory/sites.rs`: `Sites` (held by `Factory` like `research`, saved
  after the quarries; save version 17) with `Site { id, lo, hi (columns x, z), level, job }`, `Job::Dig |
  Fill | Flatten`, ids from a counter (never an index), at most `MAX_SITE` (64) on a side and `MAX_SITES`
  per world. When a site is made the core finds its highest and lowest ground once (`*_anywhere`) and
  keeps them as the cut and fill ranges. Cell order: cut layers from the top down, fill layers from the
  bottom up, rows back and forth; a fill cell counts only from its column's ground up (never into caves).
  `Action::MarkSite { lo, hi, level, job }` and `Action::RemoveSite { id }` (codec tags 21, 22). A
  `survey` query (loaded chunks only) counts cut and fill blocks, ore, water and the net spoil. Nothing
  works a site yet. **Done when:** tests for the order, limits, save and hash, and the codec pass.
  *Done 2026-09-28:* `MAX_SITES` 32; sites never overlap; `MarkSite { a, b, level, job }` takes the
  corners in any order; `level` is the ground's top when done; `high` / `low` are the highest block and
  the lowest column ground; `survey_site` also counts trees and unloaded columns; `api/sites.rs` has
  `mark_site`, `remove_site`, `sites`, `site_survey`. Marking 64 × 64 on fresh ground: 6–11 ms native.
- [ ] **6.2 The planner.** A hand item `PLANNER` (273; recipe of plates, wire and glass; no research).
  While held it aims up to `PLANNER_REACH` (64, like `belt_line.rs`'s far raycast): right-click sets the
  first corner, the second opens a small panel (`web/src/ui/site.ts`): job buttons, height −/+ (default:
  the first corner's surface), and the survey ("cut 1,240 · fill 310 · 930 to carry away · 12 ore ·
  water"). Mark queues the action; right-click inside a site with the planner opens it (progress,
  remove). Overlays: sites outlined in `render/outlines.ts` (cut red, fill blue, the pending one amber),
  squares on the minimap. A tip: "Plan the ground". **Done when:** a Game-level test marks and removes
  a site through the API; screenshots of marking, the panel and two sites.
- [ ] **6.3 The excavator digs.** Block 60 `EXCAVATOR`, `factory/excavator/` (its own folder: machine,
  drones model, tests), unlocked by an **Earthworks** tech (red and green packs). It works the nearest
  site whose closest cell lies within `EXCAVATOR_RANGE` (32) and draws 30 kW. Four drones (presentation
  only, derived from each cell's progress like the quarry's head) fly out, cut a block and fly back with
  it; about 4 blocks a second at full power. It digs ground, logs and leaves (leaves drop nothing, logs
  become logs); ore and spent rock go like hand mining (`HAND_YIELD`, costing the deposit), and the
  planner warns about them first. Bedrock, machines, belts and pipes stay. It waits while a cell is
  water ("flooded"). Output: a buffer feeding belts and boxes like the quarry's. Several excavators on
  one site share its cells. **Done when:** a 16 × 16 hill is cut to a level and a belt carries exactly its
  blocks; ore handling, sharing and a save mid-dig pass; screenshots of the drones at work.
- [ ] **6.4 Fill and flatten.** An excavator fills cells from its buffer: dirt for the top layer when it
  has some (grass spreads by itself through the block timers), else stone or rock. It takes fill from
  belts and boxes beside it (a sink for `QUARRIABLE` drops). Dug blocks go to fill sites in range first,
  then out, so a flatten site balances itself and a fill site anywhere in range is a dump. Filling into
  water replaces it. **Done when:** a flatten site ends level with the surplus boxed; a fill site in the
  sea makes dry land that stays dry; a hill carried into a hollow; screenshots before and after.
- [ ] **6.5 Tunnels.** A planner mode: two points make a straight tunnel (1 × 2, 3 × 3 or 5 × 5; slopes
  up to 1 in 2), dug from the first point by an excavator in range of it. The excavator moves its work
  point along the tunnel (range from the tunnel face, not the machine), so a long tunnel needs one
  excavator at its mouth. Breaking into water waits like a cut. **Done when:** a 64-block tunnel through
  a hill, a sloped one down to a vein, and a flooded breach that waits pass as tests; a screenshot.
- [ ] **6.6 Scale and feel.** Earthworks 2 (research): excavator Mk2 with 8 drones, range 48, twice the
  speed. Measure the worst tick and remeshing with 4 excavators on a 64 × 64 flatten next to the sea;
  fix what shows. Sounds and cracks for drones near the camera only. README, tips, balance numbers.
- [ ] **6.7 Milestone 6 cleanup.** Section 3.1 checklist, measured numbers, `docs/CODEMAP.md` current.
  Then ask the Milestone 7 questions (section 6) and move Milestone 7 in from the roadmap.

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

## 5. Roadmap after Milestone 6

Milestones 7–10 are in `docs/ROADMAP.md`; read it only when planning.

---

## 6. Open questions for the user

Ask these when the milestone that needs the answer comes up, not before. Record the answers in section 1
and adjust the steps.

| Needed by | Question |
|---|---|
| M7 | Blueprints: copied from what you built (select an area), drawn from scratch as ghosts, or both? Where do drones take materials from: boxes in range of a drone port, or anywhere on a network? |
| M8 | Personal flight: a jetpack that burns fuel, a powered hover pack, or both as tiers? And trains or trucks after it? |
| M10 | Megaproject theme (rocket ship or something else) and what launching unlocks. |

---

## 7. Working in this repo

See `docs/WORKFLOW.md`: commands, browser verification, Windows gotchas, deploying, working with the user,
and the balance numbers. Read the section you need.

---

## 8. Change log of this plan

- **2026-09-25:** Plan created (`3239215`); keeping the codebase small for agents is a top priority.
- **2026-09-25: Milestone 1 (Foundation) done** (`31bb05b` to `74fb476`): fixed tick, core `Sim`, actions,
  events, several players, canonical bytes, state hash, saves. Tests 56 → 74, wasm 81.5 KB gzipped.
- **2026-09-25: Milestone 2 (Make it a game) done** (`b49b766` to `3bf3f92`): items, machines and panels,
  belts, boxes, power, research, Mk2, tips; save version 9. Tests 74 → 113, wasm 120.3 KB. Lessons:
  measure wasm every step; the golden hash catches unintended core changes; prefer bare-`Factory` tests.
- **2026-09-26: Milestone 3 (Co-op) done** (pushed `c845691`): lockstep over bytes (`action/codec.rs`,
  `net/`), join snapshots and player keys (save version 10), avatars, host-owned loose items, a hidden tab
  that keeps ticking, the signalling Worker (STUN only), WebRTC, the "Play together" menu, pings, 10 s
  timeouts and in-place resync; 3.9 deferred. A client uses about 8 KB/s down, 4 KB/s up while idle.
  Lessons: keep heavy sort keys `#[inline(never)]`; key hidden-tab work off stalled frames; edit files
  with the file tools (PowerShell reads UTF-8 as ANSI). Tests 113 → 127; wasm 131.5 KB gzipped.
- **2026-09-26: Play-test notes P1–P5 done** (`233371b`): Escape hints before pausing, the minimap, block
  timers (`sim/timers.rs`, save version 11: leaf decay, grass), tools (wear is the stack count). The `art`
  branch started (`docs/ART_HANDOVER.md`). Tests 127 → 141; wasm 138.0 KB gzipped.
- **2026-09-27: Milestone 4 (Reasons to explore) done** (`8d8ee22` to `a4fc499`): generator versions
  (version 1 pinned), saplings, rocks, limestone, quartz, glass, biomes (version 2 about 10 % slower),
  ores by biome with soil hints, scanner and core drill, map marks, day and night, sky and block light
  (meshing 0.36 → 0.49 ms per chunk, +25 % mesh memory), lamps; the art salvage and the build menu grid.
  Save version 12. Tests 141 → 183; wasm 138.0 → 161.4 KB gzipped.
- **2026-09-27: Belt placement upgrade** (`0bc0e47`, user request): ramps derived from placement
  (`belt_shape::derive_slopes`), drag-to-build lines (`belt_line.rs`, plain `PlaceBlock`s, 3 per tick),
  `Action::Rotate` on R, research on T, tech 1 is Belt Lifts. Golden hash re-recorded. Tests 183 → 193.
- **2026-09-27: Milestone 5 redefined** at the user's request: Water and world shape (sea, ponds,
  swimming, limited flowing water, pumps and pipes, rare surface ore with depth bands; generator version
  3) comes before terraforming, which moved to Milestone 6.
  The user then added a quarry (step 5.8) so rock and soil get automated extraction.
- **2026-09-27: Milestone 5 (Water and world shape) done** (`becc673` to `c1c61f9`): generator version 3
  (version 2 pinned; rare surface ore on bare rock, depth bands, a starter set 40–80 blocks out; measured
  0.24–0.5 exposed outcrops per column, was 3.3), the sea (level 62) and ponds, water drawn blended with
  underwater fog, swimming and floating items, flowing water as core state (deviation: the sea is the
  only endless water), pumps, pipes and outlets (Fluid Handling), and the quarry (preview box, gantry,
  panel). Save versions 13–15. Tests 193 → 225; wasm 161.4 → 192.5 KB gzipped (`vite build`; the quarry
  about 11 KB of it). Lessons: a rule that looks right in the plan (two-source water) can block a later
  step, so test the next step's scenario early; PowerShell array patches misfire on a single pair.
- **2026-09-27: Power rework** (user request, before 5.9): every miner needs power (Mk1 5 kW);
  generators store fuel energy (kW·ticks; coal 270 kJ, log 135 kJ) and give only what is drawn, up to
  60 kW, so one coal runs a Mk1 long enough to mine about 32 coal. Generator and poles are made from ore
  so the first loop (a coal miner feeding its own generator) comes right after the first miner; a new
  tip (10 of 10). Save version 16 (old fire ticks convert to energy); golden hash re-recorded. Old
  worlds' Mk1 miners stop until a pole and generator reach them. Tests 225 → 226.
- **2026-09-27: 5.9 done** (Milestone 5 cleanup, `a682a28`): plan compressed, section 2 and the README
  brought up to date, sizes measured.
- **2026-09-27: Milestone 6 (Terraforming) detailed** after the user set the order (terraforming, then
  blueprints and drones, then transport with personal flight first). The roadmap is renumbered: 7
  Blueprints and drones, 8 Transport and flight, 9 Fluids and depth, 10 Endgame.
- **2026-09-27: Torches and stronger lamps** (user request, before 6.1): block 57, 8 from a log and a coal
  ore, on any solid block, dropping when it goes (`sim/torches.rs`). Light sources are rows of
  `light::SOURCES` (strength, loss per block): a lamp 20 losing 1 (full within 5, reaches 19; was 14), a
  torch 12 losing 2 (reaches 5) in a second short field. The light margin grew to 20; `bench_meshing`
  measured 0.54 → 0.43 ms per chunk. `block/tables.rs` split out. Tests 226 → 229.
- **2026-09-27: Finding ore** (play-test request, before 6.1): generator version 4 (new worlds; version 3
  now pinned in `released_versions_never_change`): bands coal 6–22, iron 8–28, copper 12–34, metal
  exposure 0.42 (was 0.28), two starter patches each. The explored map (`minimap/atlas.rs`, the
  minimap now draws from it), the world map on M with pins (mute moved to K), ore diamonds, the ore
  guide, the stained-soil readout. Golden hash re-recorded. Tests 229 → 234; wasm 192.5 → 201.6 KB
  gzipped with the torches (`to_lowercase` alone cost 14 KB, avoided).
- **2026-09-28: 6.1 done** (sites in the core): `factory/sites.rs` (`Sites` on `Factory`, save version 17),
  `Action::MarkSite` / `RemoveSite` (tags 21, 22), `survey_site`, `World::is_air_anywhere` (scans skip the
  sky), `api/sites.rs`. Golden hash re-recorded. Tests 234 → 238; wasm 201.6 → 204.5 KB gzipped.
- **2026-09-28: Wood and ladders** (user request, before 6.2): planks (block 58, 4 from a log, a weak fuel),
  sticks (item 272, 4 from 2 planks), ladders (block 59, 3 from 4 sticks; `player.rs` climbs them, the
  body step asks which block is in a cell). Tools, torches (now 4 from a stick and a coal), poles and the
  box use processed wood; a Materials build-menu section. The excavator moves to block 60, the planner to
  item 273. Tests 238 → 239.
- **2026-09-28: Glass from sand, brighter lights** (user request): the smelter turns sand into glass (quartz
  now 1 → 2); `light::SOURCES` torch 24 (reaches 11), lamp 32 (full within 17, reaches 31), `MARGIN` 20 → 32.
  `bench_meshing` 0.43 → 0.56 ms per chunk. The research panel's header named R; research is on T.
