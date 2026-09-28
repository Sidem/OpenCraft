# OpenCraft roadmap (after Milestone 6)

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

## Milestone 7: Electronics, blueprints and drones (era 4, violet)

Its questions are in DEV_PLAN section 6; ask them before detailing it.

- **Electronics:** the arc furnace (silicon from deep quartz: the first reason for mine shafts),
  circuits, processors, violet science, Mk4 for belts and machines, solar panels and accumulators,
  scanner Mk2.
- **Logic:** sensors (box or belt fullness), switches, machines on or off by condition.
- **Blueprints:** select an area to copy it; place it as a ghost (rotate, see what is missing); ghosts
  can also be placed from the build menu, one by one or dragged out. A longer reach for ghosts.
- **Construction drones:** a drone port (3×3 pad) builds ghosts from materials in boxes beside it and
  tears down what you mark. Drones are items (motors, circuits): the factory builds its builders.
- **Flight and helpers:** the coal jetpack (the user's wish for flight early); a personal drone that
  fetches items from boxes.

## Milestone 8: Terraforming (era 5)

Moved here on 2026-09-28: its work drones are Milestone 7's drones, and big multi-block factories and
rail beds are what need flat land. **Already built:** sites in the core (`factory/sites.rs`: `Site`,
`Job` dig, fill or flatten, cell order, `MarkSite` / `RemoveSite`, `survey_site`, `api/sites.rs`; save
version 17); nothing works a site yet. Block 60 and item 273 are no longer reserved: take the next free
ids. Design carried over (was steps 6.2–6.6):

- **The planner** (hand item; plates, wire, glass): aims up to 64 blocks; right-click two corners, a
  panel (`web/src/ui/site.ts`) picks the job and height and shows the survey ("cut 1,240 · fill 310 ·
  930 to carry away · 12 ore · water"). Sites outlined (cut red, fill blue, pending amber), squares on the
  minimap, a tip "Plan the ground". Right-click inside a site opens it (progress, remove).
- **The excavator** (`factory/excavator/`, Earthworks tech): works the nearest site within 32, 30 kW,
  four drones cutting about 4 blocks a second (drones derived from each cell's progress, like the
  quarry's head). Ground, logs and leaves; ore and spent rock like hand mining with a warning first;
  bedrock, machines, belts and pipes stay; waits while flooded; output buffer to belts and boxes; several
  excavators share a site. A red Mk1 stripe; Mk2 and Mk3 by kits (TECH_ERAS section 1).
- **Fill and flatten:** fill from its buffer (dirt on top, else stone, rock, slag, later tailings: a list
  of fill materials), and from belts and boxes beside it. Dug blocks fill sites in range first, so a
  flatten balances itself and any fill site is a dump. Filling water makes dry land that stays dry.
  Concrete foundations as a finish.
- **Tunnels:** two points, 1 × 2, 3 × 3 or 5 × 5, slopes up to 1 in 2; the excavator's range counts from
  the tunnel face, so one at the mouth digs a long tunnel. Breaking into water waits.
- **Scale:** measure the worst tick with four excavators on a 64 × 64 flatten by the sea; big edits go
  through `set_block_anywhere_later`; drone sounds near the camera only.
- Facts from Milestone 5: only the sea is endless water (pumps can't lower it); water checks are capped
  per tick; reuse the quarry's patterns (`survey`, output buffer, flooded wait), don't generalise it.

## Milestone 9: Distance (era 6)

- **Bauxite** (generator version 5, bundling every later resource: bauxite, oil reservoirs, uranium):
  only in deserts and basalt fields, far from spawn. The electrolytic cell makes aluminium.
- **Trains** (rails of steel beams and concrete, stations, signals) or **trucks** on recorded routes;
  rail beds, cuttings, tunnels and bridges are Milestone 8's jobs.
- **Cargo drones** between drone ports; **the hover pack** (aluminium, batteries, processors) as flight's
  second tier; ziplines along power lines if cheap.

## Milestone 10: Fluids and chemistry (era 7, gold)

- **Rivers** carved into the height map, flowing to the sea; **hydro** with water wheels and dams.
- **Oil:** pumpjacks, a refinery with three outputs, a chemical plant; plastics, sulfur, acid. Oil
  products ride belts as canisters, so pipes stay water-only. Diesel generators. **Electrolysis**
  (hydrogen, oxygen) for fuel cells and later rockets.
- **Ore washing** (2 ingots per ore, tailings as fill); **mine shafts and hoists** for deep lodes.
- **Nuclear:** uranium, the centrifuge, the reactor (peaceful: overheating stops it). Gold science, Mk5.

## Milestone 11: Compute and photonics (era 8)

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
