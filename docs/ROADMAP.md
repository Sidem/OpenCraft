# OpenCraft roadmap (after Milestone 8 is planned)

Read this only when a milestone ends and the next one is being planned. `docs/DEV_PLAN.md` details the
**current** milestone only. At each milestone's cleanup step, move the next milestone from here into the
plan and detail it there (steps with where, how and done-when).

**Agreed order after Milestone 12 (2026-10-16):** orbit polish, the feel-and-feedback interlude, the carry's pieces
in the existing eras, the megaproject theme (talk only), the ruins interlude, Milestone 13 (DEV_PLAN section 1).

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
- **Where you build matters** starts here (chosen 2026-10-08; see the interlude below): a datacenter
  needs less coolant underground or beside the sea. Fold it into DEV_PLAN steps 11.3–11.4 if those are not
  built yet, else into 11.13.

## Interlude: less hand-crafting (agreed with the user, 2026-10-09; done, steps L1–L3 in DEV_PLAN section 4)

Hand crafting is a lot of work early on. No XP or skill tree (it rewards doing the chore the game wants you to automate
and would be a second progression track beside research). Instead:

1. **`player_bonus(stat)`** (S): one place that sums a player's modifiers from researched techs, worn gear and any later
   perks, in permille like `Research::rate_permille`. Hand-crafting speed is the first stat; mining speed, reach and
   carry capacity can read it later. Core state only if a source is (techs and gear already are).
2. **Hand-crafting speed techs** (S): a few tiers on green, blue and violet packs (e.g. +25% each), read by
   `Recipe::hand_ticks` through the crafting queue. A new `Unlock` kind or a `Bonus`-like row.
3. **Worn craft-speed gear** (S): built as the Fabricator Gloves in a new Gloves slot.

Crafting from nearby boxes or by request was dropped by the user on 2026-10-11 and will not be built.

## Interlude after Milestone 11: feel and feedback (chosen by the user, 2026-10-08)

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
  and a setting turns it off. Lives in `web/src/audio/` from the existing sound events. Later, maybe: a
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

## Milestone 12: Orbit (era 9)

Moved into `docs/DEV_PLAN.md` section 4 (2026-10-09).

## Interlude after Milestone 12: ruins of the last factory (chosen by the user, 2026-10-08)

Placed before the megaproject so its logbooks can foreshadow it (whose theme is decided then); it needs only
the recycler and a new generator version, so it can move earlier.

- **Ruins (M).** Rare worldgen sites (new worlds only, a generator version bump): rusted machines, belts
  and poles of builders who came before and ran their ground dry, over collapsed shafts down to nearly
  exhausted lodes. That tells the finite-ground story (pillar 2) without words. Rusted blocks are their
  own ids; breaking them or feeding them to the recycler gives scrap metal and the odd circuit.
- **Logbooks** (an item with a reading panel) give real help: a bearing to an ore's biome, a recipe or tip
  ahead of time, a pin on the map. Near ruins hold early hints, far ones fragments about the late tech
  tree and the megaproject. The first ruin is 150–300 blocks from spawn, then about one per 512×512
  region. Placement is a pure function of the seed, like deposits.
- **Sealed rooms and relic machines** from the thread below (the carry): vaults, a sieve, a crane and a
  printer, the main home of that thread.

## Thread: the carry (p-adic numbers and fractals, by discovery; chosen by the user, 2026-10-11)

Mechanisms that behave exactly like p-adic numbers and fractals, so players find the ideas by playing.
**Mainly in the ruins, partly in the existing eras, no tech line of its own:** pieces in the eras ride
existing techs (the tally on logic, radiators on datacenters); ruin pieces are found, not researched.
Optional: main progression never waits on it, but the payoffs are real (cooling, antenna bands, loot).

- **Pure discovery, names only** (user's choice). The game never explains. When a player does the thing,
  a discovery shows only its name ("3-adic address", "Cantor set"); logbooks hint with a drawing or a
  number sequence, never a formula. So every mechanism must carry the idea by itself: digits are shown
  right to left with a leading "…" ("…0110"), so carries and endless digits are visible.
- **Primes 2, 3 and 5 each lead where they work best** (table at the end).
- **The thread, for agents** (never shown in game): a binary counter's carry length is the ruler
  sequence 1 2 1 3 1 2 1 4 (the 2-adic size of n); Tower of Hanoi moves follow it, and Hanoi's state map
  is a Sierpinski triangle; Pascal's triangle mod p is a fractal because the power of p in a binomial is
  the number of carries (Kummer); a carry that never ends gives …1111 = −1, so 1 + 2 + 4 + … = −1 in the
  2-adics. Two numbers are p-adically close when their last digits agree, which is a tree.

### In the existing eras

- **Discoveries (S).** One module: a list of named entries, each a pure check on state (a splitter tree
  4 deep, a tally below zero, a vault ring opened), a toast with the name, a page in the menu. Decide at
  planning whether it is core state (shared per world in co-op, a save bump) or kept beside the save.
- **Splitter addresses (S, primes 2 and 3, presentation only).** A splitter round-robins its outputs
  (`factory/router.rs`), so a cascade sends item n to leaf n mod 2^k (two outputs) or n mod 3^k (three).
  Looking at a belt fed by a tree at least 3 deep shows its address, first split rightmost ("…0110",
  "…210"). Items share their first j splitters exactly when they agree mod p^j: the tree is the p-adic
  integers. Discoveries "2-adic integers", "3-adic integers".
- **The tally (S, Electronics retrofit, prime 2, core state).** A logic block that counts the box or
  belt behind it: +1 for each item in, −1 for each item out, shown on 16 binary lamps. Below zero it rolls
  to all lamps lit (…1111). A sensor rule can fire on one bit, so it doubles as a divider ("every 8th
  item"), a practical reason to build it. Discoveries "2-adic −1", "two's complement".
- **Fractal radiators and the analyzer (M, Compute retrofit, prime 3).** A radiator block for datacenter
  cooling; a connected cluster cools by its exposed faces (counted when blocks change, from core state),
  so a Menger sponge (keep 20 of 27, repeat) beats a solid cube by far per block. A handheld analyzer (a
  query) box-counts a structure at scales 1, 3, 9, 27 and 1, 2, 4, 8, 16, takes the best fit and shows its
  dimension: slab 2.0, cube 3.0, Menger sponge 2.73, Cantor dust 1.89, Sierpinski tetrahedron 2.0.
  Discoveries "fractal dimension", "Menger sponge".
- **Antenna bands (S, fold into Milestone 12 step 12.3).** A comms ground station built from antenna
  blocks gains one band (one more link or more range) per level of a recognised fractal (Sierpinski
  carpet or triangle, matched by template). Discovery "fractal antenna".

### In the ruins (the main part)

Each piece is a sealed room or relic in a ruin site, with loot behind it (scrap, circuits, logbooks,
relic recipes deeper in).

- **Hensel vaults (M, primes 5 and 3, and 10).** A door of rings, each a dial of p digits; the door shows
  a target as a digit string. Ring k opens when the number on rings 1..k, squared, ends in the target's
  last k digits. The first ring is found by trial, then each ring has exactly one right digit, so the
  number grows from the right forever (rewards shrink after about 8 rings).
  - The 5-door: x² = …4444 (−1 in base 5); answers 2 → 12 → 212 → … Discoveries "5-adic integers",
    "√−1".
  - The 3-door: x² = …2222 (−1 in base 3). No digit fits the first ring; trying all three opens a side
    passage. Discovery "no 3-adic √−1".
  - The 10-door: x² = x in decimal. Two endless answers (…890625 and …109376) open two halves; the last
    lock multiplies them to …000000. Discovery "zero divisors" (why p must be prime).
- **Cantor sieve (S, prime 3).** A wall of lamps fed by a three-way splitter cascade the player rebuilds
  from the ruin's broken one. The door opens when exactly the leaves whose base-3 address has no digit 1
  light up (every middle output dumped, at every level). Lamps sit in address order with the first split
  as the coarsest, so the lit wall is Cantor dust. Discovery "Cantor set".
- **Hanoi crane (S–M, prime 2).** A gantry crane over three pads and a stack of crates of falling size;
  a crate never sits on a smaller one. Moving the stack opens the room; the shortest solution is 2^n − 1
  moves and its rhythm is 1 2 1 3 1 2 1 4. A panel traces the states visited, which fill a Sierpinski
  triangle. Discoveries "Tower of Hanoi", "ruler sequence".
- **The layer printer (M, relic, primes 2, 3, 5).** A broken machine in a deep ruin, repaired with scrap
  and circuits, then craftable. It prints upward onto a construction site (drones place the blocks, as in
  terraforming): each cell is the sum mod p of its neighbours in the layer below, the digit as a block
  colour, p set on the machine. One seed grows Pascal's triangle mod p (Sierpinski for 2); square layers
  grow 3D fractal pyramids; two seeds overlap predictably. Its prints suit radiators and antennas.
  Discoveries "Pascal's triangle mod 2/3/5", "Sierpinski triangle".

| Prime | Leads |
|---|---|
| 2 | splitter addresses, the tally, the Hanoi crane, the printer's default |
| 3 | three-way addresses, the Cantor sieve, Menger radiators, the 3-door that cannot open |
| 5 | the √−1 vault, the printer mod 5 |
| 10 (2 and 5) | the x² = x door: zero divisors, why primes |

**Order:** discoveries, splitter addresses and the tally fit any interlude (all small); radiators and the
analyzer any time after Milestone 11; antenna bands with step 12.3; the ruin pieces with the ruins
interlude, after its generator version.

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
