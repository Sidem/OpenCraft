# OpenCraft tech tree: the concept

How technology in OpenCraft grows from a stone pickaxe to AI datacenters, drone swarms, laser power links
and satellite constellations: the **lines** it grows along, how they **connect**, the colour-coded
**upgrade system**, and the **content architecture** that keeps all of it cheap to add. Per-era items,
recipes, machines and techs are in `docs/TECH_ERAS.md`; milestone order in `docs/ROADMAP.md`. Numbers are
first guesses to tune in play. Once an era is built the code is the truth; shrink its notes then.

## 1. Rules for every technology

- **Every tech does at least one of three things** (the concept map tags each): opens a **new
  possibility** (a machine, a material, a way to move or build), makes something **more efficient**
  (recovery, yield, speed, power per fuel, packs per unit), or **helps the player** directly (tools,
  flight, maps, automation of chores). A tech that does none of these is cut.
- **Bootstrapping.** Each era's machines are built from the era before, and its first products improve
  the previous era's machines. The player climbs by using what they just automated.
- **A new kind of problem per era** (DEV_PLAN pillar 5): throughput → heat and flux → precision and depth
  → terrain → distance → fluids → heat and bandwidth → orbit → scale.
- **Everything stays physical** (DEV_PLAN pillar 1): items ride belts, trains, drones or drop pods, never
  teleport. Only energy and data travel as flows (wires, fibre, laser beams, satellites).
- **Automation gates.** From blue science on, packs and kits are machine-made only.
- **The browser is the only limit.** Anything the Rust core can simulate deterministically in fixed ticks
  and WebGL2 can draw with instanced boxes, lines, glow and sky points is fair game (section 5).

## 2. The lines

Eleven lines run through every era. Arrows are the order techs arrive; **feeds** names the lines that
depend on it.

| Line | Arc | Feeds |
|---|---|---|
| **Extraction** | hand tools → miners Mk1–5 (recovery 60 → 97 %) → quarry → crusher (+50 % metal) → excavator → washer (2 ingots per ore) → laser drill → orbital survey → asteroid drop pods | materials, terrain |
| **Materials** | ingots, glass → bricks, quicklime → steel and slag → silicon → aluminium → plastics, acid → ultra-pure silicon wafers → light frames, heat shields | everything |
| **Manufacturing** | constructor → gears, motors → assembler (multi-block) → circuits, processors → chip fab (clean room) → AI accelerators → rocket parts | logistics, compute, space |
| **Logistics** | belts, splitters, lifts → belts Mk2–4 → silos → trains, stations → cargo drones → orbital drop pods | all production |
| **Power** | coal generator → generator Mk2 → steam (2× per coal) → solar, accumulators → diesel, hydro → nuclear → laser power links → orbital solar beamed down | every machine; compute |
| **Research and compute** | labs, red → green → blue → violet → gold packs → AI datacenters → compute research and endless bonus techs → space data | everything unlocks through it |
| **Control and data** | scanner, map → logic (sensors, switches) → data network (fibre) → laser data links → comms satellites (global coverage) | drones, compute, space |
| **Construction** | hands, belt drag → blueprints → construction drones → earthworks (excavator with drones) → tunnels → AI swarms and auto-routing → building from the map anywhere in coverage | terrain, logistics |
| **Player gear** | ladders, stone and iron tools → steel tools → jetpack → personal drone → hover pack → laser cutter → exosuit (reach, carry) | exploration |
| **Fluids and chemistry** | pumps, pipes → boiler water → oil (canisters) → plastics, sulfur, acid → electrolysis (hydrogen, oxygen) → coolant loops → rocket fuel | power, compute, space |
| **Space** | rocket parts → launch site → satellites (survey, comms, power, science) → constellations visible at night → megaproject | compute, power, logistics, extraction |

## 3. How the lines connect (the links that make it one tree)

- **Slag and tailings are fill.** The blast furnace (Industry) and washer (Chemistry) make byproducts that
  terraforming sites (earthworks) take as fill, so heavy industry and terrain feed each other.
- **Steam needs water, so power moves to the coast.** Boilers drink from pumps (Milestone 5); a sea pump
  never runs dry, a pond does.
- **Silicon lies deep.** Quartz sits 25–50 blocks down: electronics is the first reason for shafts, lifts
  and ladders, and later for hoists.
- **Drones before earthworks.** Construction drones (Electronics) are the excavator's work drones, so
  terraforming arrives once drones exist and multi-block factories and rail beds need flat land.
- **Rails need earthworks; aluminium needs rails.** Bauxite forms only far from spawn, so trains (and
  the jetpack before them) bring it home; rail beds, cuttings and tunnels are earthworks jobs.
- **Chips need chemistry.** The chip fab takes ultra-pure water and acid, so compute waits for fluids.
- **Datacenters need power and cooling.** Their heat goes into coolant loops (water) and their appetite
  (megawatts) drives nuclear power and laser links from distant plants.
- **Lasers need line of sight.** A beam stops at the first solid block (glass lets it through): towers,
  mirrors and terraforming clear the way.
- **Rockets need everything:** aluminium frames, chips, hydrogen and oxygen from electrolysis, steel
  pads on flat concrete, and power.
- **Satellites pay every line back:** survey satellites put every deposit on the map (extraction), comms
  satellites let drone ports and data reach anywhere in coverage (construction, control), power
  satellites beam energy down to rectennas (power), science satellites downlink space data (research).
- **Compute pays back too:** AI research makes packs go further, the optimizer speeds machines on the
  data network, swarms fly more drones per port, auto-routing plans belt lines for the player.

## 4. Eras, research and colours

| Era | Milestone | Research input | Tier colour | The new problem | Headline |
|---|---|---|---|---|---|
| 0–2 Iron, mechanical | M1–M5 (done) | red, green packs | Mk1 red, Mk2 green | quantity, throughput | miners, belts, power, labs, quarry |
| 3 Industry | **M6 (now)** | blue packs | Mk3 blue | heat and flux | upgrades, assembler, steel, steam |
| 4 Electronics | M7 | violet packs | Mk4 violet | precision, depth | circuits, logic, blueprints, drones, jetpack |
| 5 Earthworks | M8 | violet packs | — | terrain | planner, excavator, tunnels |
| 6 Distance | M9 | violet packs | — | distance | aluminium, trains, cargo drones, hover pack |
| 7 Chemistry | M10 | gold packs | Mk5 gold | fluids | oil, plastics, electrolysis, washer, nuclear |
| 8 Compute | M11 | gold packs + compute | — | heat and bandwidth | chip fab, AI datacenters, laser links |
| 9 Orbit | M12 | compute + space data | white (not a tier) | orbit | rockets, satellites, constellations |
| 10 Megaproject | M13 | everything | — | scale | the final construction, which unlocks more |

**Research inputs change form:** items (packs) through era 7, then **compute** (a flow from datacenters
over the data network), then **space data** downlinked from science satellites. Late techs cost packs
plus compute; endless bonus techs (mining productivity, belt speed, drone speed …) cost only compute and
grow each level, so datacenters always have work.

## 5. The far end: what it does and how the browser does it

| Technology | Gameplay | Engine (Rust core) | Drawing (WebGL2) |
|---|---|---|---|
| **AI datacenter** (4×4×3 hall) | turns megawatts and coolant into compute (TF); Mk tiers add racks | a processor spec with a flow output; heat as a flow into coolant loops | instanced racks, blinking light layers |
| **Data network** | carries compute from datacenters to consumers | a second channel beside power in the grid module (fibre nodes like poles) | cables as thin boxes |
| **Compute consumers** | AI labs (research), optimizer nodes (+speed for machines in range), swarm hubs (more drones per port), satellite uplinks | demand on the data channel, like power demand | status lights |
| **Auto-routing** (player helper) | drag from a machine's output to another's input: a belt route with lifts and underpasses appears as ghosts for drones or hands | A* over voxels as a query; the result is ordinary `PlaceBlock` actions or ghosts | the ghost line of `belt_line.rs` |
| **Laser links** (emitter and receiver towers, mirrors) | move power or data up to 64–512 blocks by tier at 80–98 % | a link edge valid while a line-of-sight ray (`*_anywhere`) is clear; rechecked only when a block in its path changes | additive beam quads, a flicker when blocked |
| **Rockets and launch site** | assemble a rocket from parts and a payload, fuel it, launch | a machine with stages and a countdown; the payload becomes a satellite record | boxes, exhaust particles, camera shake near it |
| **Satellites and constellations** | survey, comms, power, science; coverage grows with each launch | a list of satellites (shell, phase) whose positions come from the tick in integer maths; coverage is a pure function of the list | moving points in the night sky (`render/sky.ts` stars) |
| **Orbital solar** | power satellites beam to rectenna fields (5×5) while one is overhead | supply on the power channel from coverage | a beam from the sky at night |
| **Drop pods** | ship items from a launch site to any landing pad in comms coverage | a pod entity with a flight time, then a box delivery | a falling capsule and a dust puff |
| **Building from the map** | place blueprints on the world map; drones build them anywhere in coverage, even unloaded | ghosts in core state, built through `*_anywhere` | ghosts on the map |

All of it is plain numbers stepped at 60 Hz (deterministic, co-op safe) plus a few new draw paths
(beams, sky points, particles). No new crates are needed; each machine is mostly a data row.

## 6. Tiers and the upgrade system

**The one rule players learn: a machine's stripe colour is its Mk; the kit of the next colour raises it
one step.** Packs share the colours, so research, kits and machines read as one system: Mk1 red
`#d94a3d`, Mk2 green `#4caf50`, Mk3 blue `#3f7fd9`, Mk4 violet `#9a5bd6`, Mk5 gold `#e0b02f`. Every stripe
also shows 1–5 pips, and readouts say "Mk3" (colour is never the only cue).

- Stripes: belts on their side rails, machines as a band around the housing; the same colour on the
  build-menu tile, a corner chip on slot icons and in the target readout ("Miner Mk2 · next: blue kit ×4").
- **Kits**, one item per tier (green, blue, violet, gold). Hold kits and right-click a machine: it
  upgrades in place, keeping direction, contents, recipe, riding items, links and power. **Hold
  right-click and drag along belts** to upgrade a line as far as the kits last (label "12 belts · 12 kits
  (you have 9)"). Lifts, ramps and underpasses are belts.
- Kits per step: belts, routers, poles, pipes 1; single-block machines 4; multi-blocks 8. A craft makes 4.
  Breaking returns the upgraded machine, never the kit. The build menu also shows derived rows ("Belt Mk2
  = Belt + 1 green kit").
- A machine unlocked in a later era starts at Mk1 and can take older, cheaper kits at once.
- What each tier changes, per family: `docs/TECH_ERAS.md` section 1.

## 7. Multi-block machines

- A **footprint** w × d × h turned by the facing (R turns the ghost). The anchor cell holds the machine's
  block; the others hold one shared `MACHINE_PART` block. Every cell maps to the machine; breaking any
  cell returns it, at its tier, with its buffers. Power reaches it through any cell.
- **Ports** are faces of footprint cells, drawn as framed hatches with an arrow in or out. Belts pointing
  into an input port deliver; belts leading away from an output port take. Input ports accept any of the
  recipe's inputs; each output port serves one output (product at the front, byproduct at the side). A
  machine whose output has nowhere to go stops and says which ("Slag has nowhere to go").
- Placing needs every cell free (air, plants, water); the ghost is amber, red where blocked.

## 8. Content architecture: modular and open

The fast belt is a `fast: bool` and the Mk2 miner a `mk2: bool` that leak into power, textures and saves.
That pattern must not spread. These rules hold for every step from Milestone 6 on:

1. **Variants are data.** A tier, material or mode is a field indexing a table (`tier: u8` into the
   family's stats array), never a `bool` or a block per variant. A new tier is a row, not a code path.
2. **Behaviour per kind, numbers per row.** Every machine that turns inputs into outputs (smelter,
   constructor, assembler, blast furnace, crusher, arc furnace, refinery, chip fab, datacenter …) is one
   generic **processor** driven by a `ProcessSpec` row: block, footprint, ports, recipe categories,
   energy (burner, electric, none), buffers, model parts. A new module only for new behaviour (belts,
   miners, excavators, pumps, grid nodes, labs, drone ports, laser links, launch site).
3. **Recipes belong to categories, not machines** (smelting, pressing, assembly, chemistry, fabrication
   …), with several outputs (byproducts) and flows (water, compute) as well as items. Machines list the
   categories they take; a better machine reuses the same recipes.
4. **Unlocks are one enum:** a recipe, a machine recipe, an upgrade (family, tier), a feature (jetpack,
   blueprints, auto-routing, orbital map …) or a bonus level. Code asks `research.has(Feature::…)`.
5. **One place computes rates:** family tier × research bonuses × optimizer × power satisfaction.
6. **Grids are channels.** Power is the first; data (compute) the second, with the same node, link and
   balance code. Wire range, laser line of sight and satellite coverage are link providers. Generalise
   `power.rs` when the data channel arrives (Milestone 11), not before.
7. **Saves stay open:** a common machine header (kind, position, direction, tier, rotation) then the
   kind's bytes; ids append only; every format change bumps the save version with a cheap migration.
8. **Models are data too:** a spec's box parts plus an automatic tier band and port hatches.
9. **Content lint tests** keep the tables honest: every item has a source and a use (or is an end
   product), every category has a machine, every tech is reachable with no cycles, every unlock exists,
   every family's tiers are contiguous. They are cheap and catch what an agent misses.
10. **Queries and presentation stay out of the core.** Auto-routing, AI survey readings and sky
    satellites are queries or drawing; anything that changes the world is an action.

## 9. How efficiency compounds (targets)

| Setup | Ore per deposit unit | Ingots per ore | Ingots per unit |
|---|---|---|---|
| Hand, iron pickaxe | 0.004 | 1 | 0.004 |
| Miner Mk1 or Mk2, smelter | 0.60 or 0.75 | 1 | 0.60 or 0.75 |
| Miner Mk3, crusher | 0.85 | 1.5 | 1.28 |
| Miner Mk5, crusher, washer | 0.97 | 2 | 1.94 |
| Asteroid drop pods (after the megaproject) | outside the finite ground | | |

- A deposit gives 3.2× as much metal at Mk5 as at Mk1. Each miner tier roughly fills the belt tier below
  it (Mk3 3.4 ore/s against a Mk1 belt's 2.9), so upgrading miners pushes belt upgrades.
- **Power per coal:** 270 → 340 → 540 kJ; then solar, diesel (3,000 kJ a canister), nuclear, orbital.
- **Research:** Mk3 labs make every fifth unit free, Mk4 every third; AI research and bonus techs later.
