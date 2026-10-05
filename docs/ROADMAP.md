# OpenCraft roadmap (after Milestone 8 is planned)

Read this only when a milestone ends and the next one is being planned. `docs/DEV_PLAN.md` details the
**current** milestone only. At each milestone's cleanup step, move the next milestone from here into the
plan and detail it there (steps with where, how and done-when).

The order can change with the user's priorities. The determinism rules (DEV_PLAN section 3.4) keep every
feature co-op-safe even before networking exists. The agent rules (DEV_PLAN section 3.1) and the content
architecture (`docs/TECH_TREE.md` section 8) apply to all of it. Every milestone ends with a cleanup step.

**The arc the user asked for:** a steady, intuitive and fun growth of the player's reach (2026-09-27:
building by hand → planning an area → machines and drones doing the work → building from afar and from
the air), carried by a tech tree where every technology opens something, improves efficiency or helps the
player, up to AI datacenters, drone swarms, laser links and satellite constellations (2026-09-28). Each
milestone brings one era of `docs/TECH_TREE.md` (the concept) and `docs/TECH_ERAS.md` (the detail).

## Milestone 8: Terraforming (era 5)

Moved into `docs/DEV_PLAN.md` at the end of Milestone 7 (2026-10-03).

## Milestone 9: Distance (era 6)

Moved into `docs/DEV_PLAN.md` at the end of Milestone 8 (2026-10-04); this is the original sketch.

- **Bauxite** (generator version 5, bundling every later resource: bauxite, oil reservoirs, uranium):
  only in deserts and basalt fields, far from spawn. The electrolytic cell makes aluminium.
- **Trains** (rails of steel beams and concrete, stations, signals) or **trucks** on recorded routes;
  rail beds, cuttings, tunnels and bridges are Milestone 8's jobs.
- **Cargo drones** between drone ports; **the hover pack** (aluminium, batteries, processors) as flight's
  second tier; ziplines along power lines if cheap.

## Milestone 10: Fluids and chemistry (era 7, gold)

Moved into `docs/DEV_PLAN.md` at the end of Milestone 9 (2026-10-05); this is the original sketch.

- **Rivers** carved into the height map, flowing to the sea; **hydro** with water wheels and dams.
- **Oil:** pumpjacks, a refinery with three outputs, a chemical plant; plastics, sulfur, acid. Oil
  products ride belts as canisters, so pipes stay water-only. Diesel generators. **Electrolysis**
  (hydrogen, oxygen) for fuel cells and later rockets.
- **Ore washing** (2 ingots per ore, tailings as fill); **mine shafts and hoists** for deep lodes.
- **Nuclear:** uranium, the centrifuge, the reactor (peaceful: overheating stops it). Gold science, Mk5.

## Milestone 11: Compute and photonics (era 8)

Moved into `docs/DEV_PLAN.md` at the end of Milestone 10 (2026-10-05); this is the original sketch.

- **Chip fab** (a clean room taking acid and ultra-pure water) making AI accelerators.
- **AI datacenters:** megawatts and coolant in, compute out onto a **data network** (the grid module's
  second channel); cooling towers close the loop.
- **Laser links** for power and data by line of sight (towers, mirrors; glass lets beams through).
- **What compute buys:** AI research and endless bonus techs, optimizer nodes (+speed in range), drone
  swarms, auto-routing (drag output to input, a belt route appears as ghosts), AI survey.

## Milestone 12: Orbit (era 9)

- **Rockets and a launch site:** parts from every line, fuel from electrolysis.
- **Satellites and constellations,** moving points in the night sky: survey (every deposit on the map),
  comms (data and drone ports anywhere in coverage, building from the map), power (beamed to rectennas),
  science (space data by laser downlink, the last research input). Drop pods to landing pads.

## Milestone 13: The megaproject (era 10; optional, non-ending)

- Theme to be decided (orbital ring, space elevator, interstellar probe or another). Built in phases,
  physically in the world, needing enormous resources. Completing it **unlocks** something rather than
  ending the game: asteroid mining by drop pod, resources beyond the finite ground. Play continues.

## Later / maybe

- A tower-defence mode as a world setting (the user's idea; explicitly not now).
- Subsidence: hollowed-out lodes collapse the ground above unless backfilled or supported.
- A world settings screen: ore richness, deposit size, peaceful toggle.
- Catch-up on return: factories run forward for the time the game was closed, capped.
- Weather, mod support (the data-driven content tables are the first step towards mods).
- A dedicated server: the engine crate compiles natively (feature-gate wasm-bindgen) and speaks WebSocket,
  with `libm` on both sides for identical math. Co-op itself is player-hosted (DEV_PLAN section 2).
