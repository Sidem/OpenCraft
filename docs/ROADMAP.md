# OpenCraft roadmap (after Milestone 6)

Read this only when a milestone ends and the next one is being planned. `docs/DEV_PLAN.md` details the
**current** milestone only. At each milestone's cleanup step, move the next milestone from here into the
plan and detail it there (steps with where, how and done-when).

The order can change with the user's priorities. The determinism rules (DEV_PLAN section 3.4) keep every
feature co-op-safe even before networking exists. The agent rules (DEV_PLAN section 3.1) apply to all of it.
Every milestone ends with a cleanup step (DEV_PLAN section 3.1, "Session habits").

**The arc the user asked for (2026-09-27):** a steady, intuitive and fun growth of the player's reach:
building by hand → planning an area → machines and drones doing the work → building from afar and from
the air. Milestone 6 (terraforming, in the plan) starts it; its planner tool and work drones are what
Milestone 7 builds on.

## Milestone 7: Blueprints and construction drones

Its questions are in DEV_PLAN section 6; ask them before detailing it.

- **Blueprints:** select an area of machines and belts to copy it; place it as a ghost (rotate, see what
  is missing); ghosts can also be placed by hand from the build menu, one by one or dragged out.
- **Construction drones** (grown from Milestone 6's work drones): a drone port sends drones to build
  ghosts from materials in boxes, and to tear down what you mark. Range and drone count are research tiers.
- **Planning at scale:** a planner view that shows sites, ghosts and what they still need; a longer
  reach for placing ghosts than for building by hand.
- **Simple logic** where it helps construction: machines with an on/off condition (box full, belt full).

## Milestone 8: Transport and personal flight

- **Personal flight first** (the user's wish): a jetpack or hover pack so the player reaches and builds
  from above and further away; tiers could add range, speed and time aloft. Ziplines along power lines
  as a cheap early option.
- **Trains** (rails on the grid, stations, signals) or **trucks** on recorded routes, which create demand
  for rail beds, cuttings, tunnels and bridges (Milestone 6's jobs).

## Milestone 9: Fluids and depth

- **Rivers** carved into the height map, flowing downhill to the sea (still water, pumps, pipes and
  limited flow arrived in Milestone 5).
- **Steam generators** (water plus fuel) and **hydro** power with dams.
- **Ore washing:** a recovery bonus, producing tailings. **Byproducts as fill:** slag from smelting and
  tailings from washing become fill material for terraforming sites.
- **Mine shafts and hoists:** lodes sit 40+ blocks down, so lifting ore is a voxel-native logistics
  puzzle.
- **Oil:** reservoirs, pumpjacks, a refinery; plastics and rubber.

## Milestone 10: Endgame (optional, non-ending)

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
