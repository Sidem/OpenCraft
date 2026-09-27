# OpenCraft roadmap (after Milestone 5)

Read this only when a milestone ends and the next one is being planned. `docs/DEV_PLAN.md` details the
**current** milestone only. At each milestone's cleanup step, move the next milestone from here into the
plan and detail it there (steps with where, how and done-when).

The order can change with the user's priorities. The determinism rules (DEV_PLAN section 3.4) keep every
feature co-op-safe even before networking exists. The agent rules (DEV_PLAN section 3.1) apply to all of it.
Every milestone ends with a cleanup step (DEV_PLAN section 3.1, "Session habits").

## Milestone 6: Scale and terrain

Its questions are in DEV_PLAN section 6; ask them before detailing it. Water (Milestone 5) comes first.

- **Terraforming machines** (the signature feature). Start with "mark an area, pick a job"; no
  programming language needed: an excavator digs a marked area and hauls the spoil to a box or dump
  site, a grader flattens to a height, a filler places fill, a borer cuts tunnels and shafts. Digging
  below sea level floods unless dammed and pumped (Milestone 5).
- **Byproducts as fill:** slag from smelting and tailings from washing become fill material, so waste
  feeds the terrain system.
- **Blueprints and construction drones:** copy a region of machines, place a ghost, and drones build it
  from storage (or tear it down).
- **Transport:** personal (hoverpack, ziplines along power lines), trucks on recorded routes, trains
  (rails on the grid, stations, signals), which create demand for rail beds, cuttings, tunnels and bridges.
- **Simple logic:** sensors (belt full, box full) and on/off conditions on machines.

Everything stays deterministic (DEV_PLAN section 3.4): machines change the world only through the core,
never through loaded chunks.

## Milestone 7: Fluids and depth

- **Rivers** carved into the height map, flowing downhill to the sea (still water, pumps, pipes and
  limited flow arrive in Milestone 5).
- **Steam generators** (water plus fuel) and **hydro** power with dams.
- **Ore washing:** a recovery bonus, producing tailings.
- **Mine shafts and hoists:** lodes sit 40+ blocks down, so lifting ore is a voxel-native logistics
  puzzle.
- **Oil:** reservoirs, pumpjacks, a refinery; plastics and rubber.

## Milestone 8: Endgame (optional, non-ending)

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
  with `libm` on both sides for identical math. Co-op itself is player-hosted (DEV_PLAN section 2).
