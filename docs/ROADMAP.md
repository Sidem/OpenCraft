# OpenCraft roadmap (after Milestone 3)

Read this only when a milestone ends and the next one is being planned. `docs/DEV_PLAN.md` details the
**current** milestone only. At each milestone's cleanup step, move the next milestone from here into the
plan and detail it there (steps with where, how and done-when).

The order can change with the user's priorities. The determinism rules (DEV_PLAN section 3.4) keep every
feature co-op-safe even before networking exists. The agent rules (DEV_PLAN section 3.1) apply to all of it.
Every milestone ends with a cleanup step like DEV_PLAN step 3.10.

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
- Remember to bump `WORLDGEN_VERSION` (`worldgen/mod.rs`).

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
- Weather, mod support.
- A dedicated server: the engine crate compiles natively (feature-gate wasm-bindgen) and speaks WebSocket,
  with `libm` on both sides for identical math. Co-op itself is player-hosted (DEV_PLAN Milestone 3).
