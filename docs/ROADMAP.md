# OpenCraft roadmap (after Milestone 1)

Read this only when a milestone ends and the next one is being planned. `docs/DEV_PLAN.md` details the
**current** milestone only. At each milestone's cleanup step, move the next milestone from here into the
plan and detail it there (steps with where, how and done-when).

The order can change with the user's priorities. The determinism rules (DEV_PLAN section 3.4) keep every
feature co-op-safe even before networking exists. The agent rules (DEV_PLAN section 3.1) apply to all of it.
Every milestone ends with a cleanup step like DEV_PLAN step 1.9.

## Milestone 2: Make it a game (content)

- **Item registry separate from blocks.** A `u16` item id; blocks become a subset of items. Procedural
  16×16 icons for non-block items. Needed for ingots, plates and parts.
- **First processing chain.**
  - A smelter takes ore plus fuel (coal or logs) and makes ingots.
  - A constructor makes ingots into plates, rods, wire and screws.
  - Machines get input and output buffers, belt I/O, and a machine panel UI (recipe, buffers, status).
  - Introduce the **machine registry** here, with the second machine kind: one file per machine plus one
    registration line.
- **Belt logistics:** splitter, merger, filter/sorter, one-block ramps, a vertical lift, and belts that pass
  under others. Keep everything on the grid; free-form curved belts fight the voxels.
- **Power:**
  - A coal generator; power poles with wire links within a range, drawn as simple line segments; consumers.
  - The grid is a graph: supply over demand gives a machine speed factor. Poles double as zipline anchors
    later.
- **Research:** a station or lab that unlocks recipes. The tech tree is data in Rust. Style is to be
  decided; see DEV_PLAN section 6.
- **Upgrades:** Miner Mk2 with higher recovery (proposed ~75%) and rate; faster belt tier.
- **Onboarding:** light hints (dig to find the outcrop, craft a miner, and so on).

## Milestone 3: Co-op multiplayer

- **Transport interface** in TypeScript (`send(bytes)`, `onMessage`) with three implementations:
  - Loopback, for tests,
  - `BroadcastChannel`, for two tabs on one machine; the fastest way to develop,
  - WebRTC data channels (reliable, ordered).
- **Signalling:** a tiny service that swaps connection offers for a room code or link, then steps aside.
  A small serverless function is enough. Hosting and account are the user's decision (DEV_PLAN section 6).
  Use public STUN, plus a TURN relay fallback for networks where direct connections fail.
- **Protocol:**
  1. `Hello` / `Welcome`: build hash must match, otherwise refuse with a message.
  2. Snapshot: the compressed save (DEV_PLAN step 1.6).
  3. Actions, stamped by the host with `current_tick + input_delay`, where the delay is 3–6 ticks.
  4. `PlayerState` at 10–20 Hz: position, look, flying.
  5. Loose item entities, host-authoritative.
  6. A checksum every ~60 ticks (`state_hash`), with resync from a host snapshot on mismatch.
- **Prediction:** show your own placed or broken blocks immediately, and roll back if the host rejects the
  action. Your own movement is local (co-op trust model).
- **Host streaming:** the host needs block data around every player for item physics. Load around all
  players, but mesh only around the local one.
- **Sim in a Web Worker.** Browsers pause `requestAnimationFrame` in background tabs, so a host that
  switches tabs would freeze everyone. Spike first:
  - (a) Zero-copy needs `SharedArrayBuffer`, which needs COOP/COEP headers. GitHub Pages can't set headers;
    the `coi-serviceworker` workaround exists.
  - (b) Alternatively, transfer mesh and instance buffers as transferable `ArrayBuffer`s (one copy).
  - (c) Verify how timers in dedicated workers behave in hidden tabs.
  - Decide from measurements.
- **UI:** Host game → share link; Join via link; player list and names; remote players drawn as simple
  box avatars through the instance renderer.
- **Later:** a dedicated server. The engine crate compiles natively (feature-gate wasm-bindgen) and speaks
  WebSocket, with `libm` on both sides for identical math.

## Milestone 4: Reasons to explore

- **Geology-driven ores:** rock types and biomes decide which ores appear where, for example copper in
  mountains, coal in lowland swamps, quartz and sand in deserts.
- **Surface hints:** rust-stained soil above iron, and similar.
- **Biomes that matter for resources and building,** not just colour.
- **Prospecting:** a scanner reveals deposits within a radius with size estimates; a core drill gives exact
  figures. Makes veins and lodes findable.
- **Minimap,** top-down from chunk heights and colours, with markers for deposits, machines and later
  rails.
- **Day/night cycle and voxel lighting:** sky light plus block light propagated in the mesher and stored per
  chunk; lamps. Makes caves and deep lodes atmospheric, and later gives solar power a reason to vary.
- Remember to bump `WORLDGEN_VERSION` (DEV_PLAN step 1.6).

## Milestone 5: Scale and terrain

- **Terraforming machines** (the signature feature). Start with "mark an area, pick a job"; no
  programming language needed:
  - an excavator digs a marked area and hauls the spoil to a box or dump site,
  - a grader flattens to a height,
  - a filler places fill,
  - a borer cuts tunnels and shafts.
- **Byproducts as fill:** slag from smelting and tailings from washing become fill material for
  terraforming, so waste feeds the terrain system.
- **Blueprints and construction drones:** copy a region of machines, place a ghost, and drones build it
  from storage (or tear it down).
- **Transport:**
  - personal: hoverpack, ziplines along power lines,
  - trucks on recorded routes,
  - trains: rails on the grid, stations, signals. They create demand for rail beds, cuttings, tunnels and
    bridges.
- **Simple logic:** sensors (belt full, box full) and on/off conditions on machines.

## Milestone 6: Fluids and depth

- **Water bodies** in worldgen: sea level, lakes, rivers. Start with still water plus pumps and pipes (a
  fluid network graph); add limited flowing water later only if it's affordable.
- **Steam generators** (water plus fuel) and **hydro** power with dams.
- **Ore washing:** a recovery bonus, producing tailings.
- **Mine shafts and hoists:** lodes sit 40+ blocks down, so lifting ore is a voxel-native logistics
  puzzle.
- **Oil:** reservoirs, pumpjacks, a refinery; plastics and rubber.

## Milestone 7: Endgame (optional, non-ending)

- Advanced research tiers: electronics, computers, aluminium, nuclear.
- **The megaproject** (theme to be decided, e.g. a rocket). It's built in phases, physically in the world,
  and needs enormous resources. Launching **unlocks** something rather than ending the game, for example
  an orbital survey revealing every deposit, or new resource sources. Play continues.

## Later / maybe

- A tower-defence mode as a world setting (the user's idea; explicitly not now).
- Subsidence: hollowed-out lodes collapse the ground above unless backfilled or supported. Unique, but
  costly.
- A world settings screen: ore richness, deposit size, peaceful toggle.
- Catch-up on return: factories run forward for the time the game was closed, capped (DEV_PLAN section 6).
- Weather, mod support, a native dedicated server (see Milestone 3).
