# OpenCraft roadmap

Future work only; read it when a milestone ends and the next is planned. `docs/DEV_PLAN.md` details the current work;
at each milestone's cleanup step, move the next item from here into the plan and detail it there (steps with where, how
and done-when). Finished work is in `git log` (the changelogs removed from `docs/history/` are at commit `025edaf`:
`git show 025edaf:docs/history/CHANGELOG_R.md`); settled choices are in `docs/DECISIONS.md`.

**Order (2026-10-16, R8 dropped 2026-10-22, the carry dropped 2026-11-14):** fire (FIRE, in the plan), the feel-and-feedback
interlude, Milestone 14. The open steps of the play-test response (L2–L6, 13.6b) are in `docs/DEV_PLAN.md` section 4. The order can
change with the user's priorities. Determinism (DEV_PLAN section 3.4), the agent rules (section 3.1) and the content
architecture (`docs/TECH_TREE.md` section 8) apply to all of it. Every milestone ends with a cleanup step.

**The arc the user asked for:** a steady, intuitive and fun growth of the player's reach (building by hand → planning an
area → machines and drones doing the work → building from afar), carried by a tech tree where every technology opens
something, improves efficiency or helps the player. Since 2026-10-16: needs before techs (`docs/DECISIONS.md`).

## Parked: pressure without combat (R8, decided "none for now" 2026-10-22; reopen only if the user asks)

- **Pressure without combat.** What enemies give Factorio is pressure: the factory can't be left alone. Peaceful options,
  all deterministic and built on existing systems: **subsidence** (hollowed lodes collapse unless backfilled or supported;
  already in "Later / maybe"), **groundwater** (deep mines below the water table seep and need pumps), **storms** (rare, from
  the core tick; lines in the open stop or wear unless roofed), **pollution** (smelters and generators spread a haze value
  per region that thins plants and dims solar; scrubbers clean it). Pick one or two; none may destroy the player's work.
  Later, with pollution, plants thin around a dirty base and come back when it is cleaned.

## Interlude: feel and feedback (chosen by the user, 2026-10-08)

Small features that make the factory more satisfying to watch, hear and read. All but the site bonuses are
presentation only (the state hash never moves; DEV_PLAN section 3.4), so they can be built in any order and
moved earlier if a session has room. Each one is its own module.

- **Timelapse of the world (S–M).** Every 5 game minutes, capture the world map's explored image
  (downsampled, about 256×256) and keep the frames in IndexedDB beside the save, never in core state. A
  History screen plays them from the first pickaxe to now in about 30 seconds, and exports a video with
  the browser's `MediaRecorder` from a canvas (no dependency). In co-op each player records their own.
  Later, maybe: a 3D replay from an action log, since the core is deterministic.
- **The factory as an instrument (S).** Machine cycles already land on fixed ticks: give each family a
  pitched note, quantised to a shared beat (e.g. every 15 ticks), so a balanced line has a groove and a
  starved or blocked machine leaves a gap you can hear before you see it. Volume falls off with distance,
  and a setting turns it off. **Limits (2026-10-16):** only the few nearest families sound at once, a hard distance
  cap, and the music (R4) quiets while it plays, so a big base never turns into noise. Lives in `web/src/audio/` from the existing sound events. Later, maybe: a
  sequencer block (core state, driven by logic) to compose with machines.
- **Follow one item (S).** Tag an item (one tag per player) on a belt or from an inventory; the tag moves
  with it, and through a machine it passes to the first output made with it. Its route is drawn on the
  world map and as a faint line in the world. **No camera follow** (the user gets motion sick). When it
  ends (in a pack, a silo or the recycler) a postcard shows where it was mined, distance, time, machines
  and transport used. Decide when planning whether the tag is core state (one field on a lane slot,
  co-op safe, a save bump) or a host-side watcher.
- **Seismic prospecting (M).** A thumper (powered, tech after the scanner Mk2) shows a vertical
  cross-section along its facing, 64 blocks wide down to bedrock: dense bands where ore lies, caves and
  voids, water, the player's own tunnels. It reads the true world (`*_anywhere`) blurred by noise that
  shrinks with its Mk, so reading it well is a skill. A query (only placing the thumper is an action). It
  sits between AI survey (11.12: wide but only a prediction) and survey satellites (Milestone 12: every
  deposit, but no depth structure), and doubles as an X-ray when planning tunnels and shafts (and
  subsidence, if that comes).
- **Where you build matters (S per bonus).** Places get real properties, each a pure function of seed and
  position (biome, height, depth below the surface), so never dependent on loaded chunks: solar by biome
  (desert strongest), datacenter cooling underground or by the sea (Milestone 11), and new machines as
  rows when wanted: a geothermal plant on vents in basalt fields (a worldgen feature, so a generator
  version) and a wind turbine whose output grows with height. The value is computed at placement, stored,
  and **shown on the ghost before placing** ("Solar here: 125%, desert"), so the player can choose a site.

## Fire: a flamethrower and burning trees (user, 2026-11-14; the next build after L2–L6)

Detail of the plan step FIRE (`docs/DEV_PLAN.md` section 4).

- [ ] **FIRE A flamethrower and fire that burns trees to charcoal** (user, 2026-11-14). *Need:* wood is the only thing that
  clears a forest or feeds an early fire, and felling is slow. *Rules:* fire burns only what grows (what `felling/tree.rs`
  `trunk` finds as a tree, saplings, ground cover); never a placed block, machine or house; a tree burns whole, like a
  felling, so a fire never edits more blocks than a fall does. *Cheap by design (the contract of every step):*
  1. **The unit is a tree or a front cell, not a block.** A `Blaze` is one record per tree (trunk foot, logs, leaves, heat,
     progress); its blocks are touched twice at most (the crown, then the trunk), through `set_block_anywhere_later`.
  2. **Event-driven.** Only burning things exist, in a due-sorted queue like `sim/timers.rs`; nothing is scanned; a world
     with no fire does no fire work (a test, and `bench-flight.mjs ... still` stays at 0 meshes).
  3. **Flat load.** Each record steps every `FIRE_STEP` ticks, offset by `hash3(foot)`, so a forest fire costs the same
     every tick; `MAX_FIRE_UPDATES` a tick, the rest waits (as `water.rs`); `MAX_BLAZES` and `MAX_FRONT` cap the records.
  4. **Spread by sampling, bounded by heat.** Every `SPREAD_STEP` a blaze reads a few points (`SPREAD_SAMPLES`) in its
     crown radius picked by `hash3(foot, tick)` (never `Sim.rng`, so other streams don't move), never a neighbour scan. A
     new blaze starts with the parent's heat minus 1 and at 0 nothing spreads: total work is bounded by a geometric series.
  5. **No new per-column state for plants.** Ground cover is a hash drawn by the mesher; `world/cover_clear.rs` already holds
     one bit per bare column, so it also means "burnt": a grass fire is a front of positions that spreads only onto columns
     that bear cover and are not yet bare, sets their bit, and dies at water, bare ground or `FRONT_REACH` blocks from where
     it started.
  6. **Presentation costs nothing per flame:** one emitter per blaze (nearest `MAX_SHOWN` only), animated in the shader from
     the time uniform, glow through `halos`; no particle system, no per-block change while burning.
  - **FIRE1 Core** (`sim/fire/`: `blaze.rs`, `front.rs`, `tests.rs`): the queue, tree burn (crown, then trunk), saved
    (`SAVE_VERSION` bump, migration, fixture), `Action::Ignite` for tests and the creative panel. **Charcoal** is a new item
    (appended id): a burnt tree drops `charcoal_for(logs)` on the ground at its foot; a `FUELS` row; the item guide must
    answer it. *Done when:* tests (a tree burns to charcoal and its neighbours do or don't by heat; placed logs, houses and
    machines never burn; water stops it; caps hold; same hash at every frame rate; co-op pair agrees), and a bench
    `tests/fire_spikes.rs` on the `tree_spikes.rs` pattern: a 1,000-tree forest lit at its centre keeps a native host frame
    with the 6 ms work budget under 8 ms worst and the added core tick under 0.3 ms p99.
  - **FIRE2 Flamethrower** (`flame_hands.rs`, `Action::Flame`): a hand tool (item and tech gate: ask the user which pack),
    held use button sends a flame every `FLAME_SECONDS`; the core casts `FLAME_RAYS` rays up to `FLAME_RANGE`, each stopping
    at the first solid cell, and per ray ignites a tree, scorches 3×3 columns (the sickle's `ClearCover` rule, no drop) and
    starts a front; one fuel item (coal or charcoal) per `FLAME_SECONDS * N`, taken in the action. Synthesised sound
    (`audio/synth.ts`), no camera effect (motion sickness). *Done when:* a headless test clears a meadow and fells a tree to
    charcoal; an empty tank does nothing; a flame never lights a placed block or a body.
  - **FIRE3 Look** (presentation): blaze emitters, embers along fronts, a glow, smoke from a smouldering trunk
    (`factory/smoke.rs` pattern); art request line in `docs/ART_HANDOVER.md`. *Done when:* seen in the browser pane, and
    `frame-bench.mjs` with a big fire shows no regression against section 2.
  - **FIRE4 Manual and cleanup:** `PLAYING.md`, patch note, code map, budgets clean, plan compressed.
  - *Defaults chosen, change on request:* fire spreads between trees by itself (bounded by heat); rain puts nothing out
    (weather is presentation); old worlds' trees (the oak's `LOG`) don't burn, as they still mine block by block.

## Milestone 14: The megaproject, a space elevator (era 10; optional, non-ending; theme chosen 2026-10-16)

Milestone 13 (Settlements) took the number 13. Optional, built in phases, physically in the world, needing enormous
resources; completing it **unlocks** something rather than ending the game. Play continues.
- **Where:** it rises from the player's biggest town (a town stage unlocks its foundation), so the towns are its workforce
  and their supply routes its supply routes.
- **Phases** (a first sketch): the anchor and foundation, the tether climbing in sections (each a large order delivered to
  its base: steel, aluminium, plastics, chips, rocket fuel for the climbers), a counterweight station sent up by rocket,
  the climber cars. Each finished section shows: the tower grows visibly into the sky, seen from far away, lit at night.
- **What finishing unlocks:** asteroid mining. Climbers carry mining craft up; asteroid ore returns by drop pod to landing
  pads, resources beyond the finite ground (pillar 2's answer at the end).
- **Engine notes for planning:** a very tall structure must stay cheap to draw (boxes or a dedicated mesh, not 4,000 blocks),
  and its sections are core state like a machine's; the sky already draws satellites (`render/sky.ts`).

## Later / maybe

- A tower-defence mode as a world setting (the user's idea; explicitly not now; 2026-10-16: still peaceful, no combat).
- Subsidence: hollowed-out lodes collapse the ground above unless backfilled or supported (one of the parked R8 options above).
- Slabs, stairs and beams for building (R7's decor set is cubes only; needs non-cube meshing and hit boxes).
- A world settings screen: ore richness, deposit size, peaceful toggle.
- Catch-up on return: factories run forward for the time the game was closed, capped.
- Mod support (the data-driven content tables are the first step towards mods).
- A dedicated server: the engine crate compiles natively (feature-gate wasm-bindgen) and speaks WebSocket,
  with `libm` on both sides for identical math. Co-op itself is player-hosted (DEV_PLAN section 2).
