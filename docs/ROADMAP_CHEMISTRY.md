# OpenCraft roadmap: chemistry, water and the codex (requests from play, 2026-11-16)

Unordered requests, kept apart from `docs/ROADMAP.md` and `docs/ROADMAP_REQUESTS.md` to keep each under its budget; pointed to
from `docs/DEV_PLAN.md` section 4. When one is built, move it into the plan, then drop it here. Rules for all: plan section 3.1
(small modules, content is data), section 3.4 (determinism), a placeholder layer and an `ART_HANDOVER.md` line for each new look.

## Chemistry and water (unordered)

- **CHEM: the chemical tree, made to teach.** *Design step CHEM0 done 2026-11-16* (the user accepted the draft ranking below;
  decisions in `docs/DECISIONS.md`). Rules for every family: it answers a need the player has now or the elevator will have
  (needs before techs); no multiplier layers (cooling, fertiliser, "higher yield" solve a problem, they add no percentage); it
  uses what exists (the one exception: gas fields, new worlds and the deposits upgrade); old recipes, rows and saves are never
  changed (a new stream or product is a new row or machine, never an edit of `Distil`); fluids stay barrels; recipes are data
  rows; every product ends in a sink (check: `recipe-graph.mjs` lists no new "made, used nowhere" item); a recipe's card names
  the real process in one line.
  **Dead-end report** (`npm run build:wasm`, then `node scripts/recipe-graph.mjs`, 2026-11-16): 375 named items, 331 recipes
  (176 by hand, 155 in machines, 52 of those upgrader rows). 16 items are raw inputs (ores, crude barrel, scrap). 31 are made and
  used in no recipe: tools, gear, vehicles, buildings (ends by design) and, in chemistry, **Lubricant Barrel (no consumer)**,
  Diesel Barrel (a fuel: generators only), Fuel Cell (reactor), Cargo Drone, Drop Pod (pods). Neither made nor used: the Recycling
  Coin, eight woods, bamboo (by design). Every chemical product has one sink: naphtha → plastic; acid and pure water → the chip
  fab; hydrogen and oxygen → rocket fuel; sulfur → acid and yellow concrete. *Blind spot:* fuels, placing a building, town
  orders and drone fuel are not recipe inputs, so each "used nowhere" line is a question, not a verdict.
  **What stops a real base** (the 37.6 h save, each chemistry machine's status after one second): both refineries and the cracker
  say "Diesel Barrel has nowhere to go" (the base has no diesel generator); two of three chemical plants say "Output full" and
  the third waits for naphtha; two of three pumpjacks wait for empty barrels. One closed loop (refine twice, crack the heavy oil
  once, plastic from the naphtha) turns 6 crude barrels into 6 plastic, 3 diesel and 2 sulfur, so **the jam is diesel and
  sulfur, not lubricant**: a refinery runs only while every stream leaves.
  **The elevator's bill** (first guesses, tune at M14; the city stage's order is 16,000 items, `settlements/orders.rs`). *M14A:*
  anchor 16,000 concrete, 2,000 steel beam, 1,000 steel plate; four tether sections, each 1,000 steel beam, 500 aluminium plate,
  500 plastic, 100 processor, 20 rocket fuel barrels. Chemistry's share: 2,000 plastic is 2,000 crude barrels (about 15% of a
  130,000-unit vein, `TECH_ERAS.md` section 6) and 80 rocket fuel barrels (about 8 minutes of one electrolyser): small. *M14B:*
  eight more sections ask for **carbon fibre** (sized so the whole tether needs more heavy oil than one vein holds, which
  sends the player to a second field: pillar 2) and climber cars (motors, batteries, 50 lubricant barrels each, liquid oxygen
  and hydrogen as fuel, **helium** to pressurise their tanks). The elevator pulls nothing of diesel and sulfur, so those two
  need their own wants: pigments (sulfur, acid), power (diesel, gas).
  **Decisions (user, 2026-11-16, recommended picks accepted):**
  1. **Boil-off: none.** A liquefied gas keeps forever in its barrel.
  2. **Gases keep the barrel; no pressure or cold container type.** With no boil-off a second type changes nothing in play, yet
     costs an empty-container item, its return loop, belts that mix two empties and a layer per fluid. Liquefied gases wear a
     frost band, one data field of the item row (`textures/chemistry.rs` bands). Each new fluid still needs a barrel texture
     layer: 40 cube layers are free, so ROOM's wider item layers come first (`docs/ROADMAP_ORDER.md` "Audit").
  3. **Lubricant is an ingredient of rotating-machine buildings** (air compressor, chiller, gas turbine, wellhead separator) and
     of M14B's climber cars, so heavy oil → lubricant gains its sinks with the machines that really burn it. No existing recipe
     changes (a saved assembler mid-batch would stall). `recipes/tests.rs` `NO_USE_YET` holds only lubricant until the first such
     row lands; delete the entry then.
  4. **Diesel gets no new sink.** The stall is a lesson (the stalled-machine card names the diesel generator and the recycler,
     which destroys anything for coins). A fuel-burning locomotive is rejected: trains have no fuel mechanic, and adding one
     changes running trains.
  5. **Ranking** (accepted): CHEM-TEACH, CHEM-DYE, CHEM-GAS, then CHEM-AIR, CHEM-COOL and CHEM-CARB beside the elevator they feed
     (built back to back before M14A; alone they would be stock with no pull). Parked: nitrogen compounds, salt and alkali,
     the small families. Cut: every multiplier sink (oxygen or coke for furnace yield, higher-yield silicon, liquid nitrogen
     datacenter loops, denser rocket fuel, fertiliser growth), superconducting cable (wires lose nothing), heat exchangers,
     dry ice, LH₂.
  **Accepted families** (tech rows are appended at the bottom of `techs::TECHS` by their own step; `needs` are today's indices;
  every unlocked recipe's ingredients must come from a tech before it: the lint
  `every_ingredient_comes_from_a_tech_before_the_tech_that_needs_it`). New block ids come from ROOM (`u16`).

  | Family | Problem it solves | Tech (needs; packs) | Machines and recipes | Real process on the card |
  |---|---|---|---|---|
  | **TEACH** | cards say what, not why; the diesel jam is unexplained | none | `recipes/process_notes.rs`: one line per chemical product, shown on the item page and the recipe card | each row below |
  | **DYE** pigments | decor and light wants (GLASS, LED); a sink for sulfur, acid, ores, quicklime | Pigments (45; red, green, blue, violet) | chemical plant rows only, 0 block ids: 6 bases (red from iron ore, green from copper ore, black from coal, white from quicklime, yellow from sulfur, blue from copper and acid) and about a dozen mixes, one tinted pouch model | blue: Cu + 2 H₂SO₄ → CuSO₄ + SO₂ + 2 H₂O; red: iron oxide, rust |
  | **GAS** natural gas | oil runs dry (a vein lasts about 9 h); a second feedstock and power source | Gas Wells (43, 22; four packs), Gas Processing (Gas Wells, 45; four packs) | gas well (a pumpjack variant) on a gas field (own hash, not `EXTRAS`; new worlds and the deposits upgrade, WORLDS), wellhead separator (associated gas for old worlds), gas plant (sweeten, then methane, ethane, LPG), gas turbine, steam reformer (hydrogen without the electrolyser), ethylene to polyethylene (a plastic without naphtha); about 5 ids | Claus: 2 H₂S + SO₂ → 3 S + 2 H₂O; SMR: CH₄ + H₂O → CO + 3 H₂ |
  | **AIR** | gate for cold and carbonising: nitrogen and oxygen | Air Separation (47, 53; five packs) | air compressor (lubricant in its recipe), distillation column: nitrogen, oxygen, argon barrels; 2 ids | Linde: compress, cool, expand, separate by boiling point (N₂ −196 °C, O₂ −183 °C) |
  | **COOL** | M14B's liquid oxygen; helium; LNG for far gas fields (pillar 5) | Cryogenics (Air Separation, 60; five packs) | chiller (lubricant), liquid oxygen, nitrogen-rejection step that takes helium from gas (most fields under 0.1%, a few 1–2%; numbers to tune), LNG barrels; 1 id | helium stays gas to 4 K, the rest liquefies first; LNG: methane at −162 °C |
  | **CARB** carbon fibre | M14B's tether; the one sink for heavy oil and for nitrogen | Carbon Fibre (43, Air Separation; five packs) | one row on an existing furnace: heavy oil and plastic, nitrogen barrels; 0 ids | pitch spun and carbonised near 1,000 °C under nitrogen |

  **Steps, one session each (parts are separate sessions only for GAS), each with its own done-when.** Every step also needs: a
  headless test, `recipe-graph.mjs` clean of new dead ends, a placeholder layer and an `ART_HANDOVER.md` line per new look, the
  manual (`FACTORY.md`) and a patch note.
  - **CHEM-TEACH.** The notes table and its API getter, the card line, notes for today's chemistry (Distil, Crack, plastic,
    acid, lubricant, electrolysis, rocket fuel, enrichment). *Done when:* a test says every product of the refinery, cracker,
    chemical plant, electrolyser and centrifuge has a note; the note shows on an item page and a recipe card in the browser pane;
    a refinery stopped on diesel shows a card naming the generator and the recycler (a test, unless the generic need lesson
    already says it).
  - **CHEM-DYE.** *Done when:* each hue is made from raw in a headless test; every base and mix has a sink (a coloured
    concrete, glass or lamp, as GLASS plans); acid and sulfur have their second sink; the lint passes for the new tech.
  - **CHEM-GAS** (a) fields, well, plant: gas fields found by the scanner, a well fills barrels, the plant gives methane, ethane,
    LPG, nitrogen; *done when* the fields' placement leaves every ore deposit of the bench save identical (WORLDS' pin test). (b)
    turbine and reformer: *done when* a turbine runs a grid on methane and the reformer makes hydrogen. (c) polyethylene and
    associated gas: *done when* a wellhead separator gives an old world's pumpjack gas by a hash of the deposit. Helium and LNG
    wait for the chiller.
  - **CHEM-AIR.** *Done when:* the column makes nitrogen, oxygen and argon from air and water; the compressor's recipe takes
    lubricant and `NO_USE_YET` is gone; the recipe graph shows nitrogen and oxygen waiting for COOL and CARB (argon is codex
    only).
  - **CHEM-COOL.** *Done when:* the chiller makes liquid oxygen (frost band) and a gas plant with a chiller takes helium; a test
    shows no boil-off over a long run; LNG barrels move by train or cargo drone.
  - **CHEM-CARB.** *Done when:* carbon fibre is made from heavy oil, plastic and nitrogen; the elevator's recipe can ask for it.
  - **Parked** (no sink today, so no pull): nitrogen compounds (Haber–Bosch ammonia, fertiliser, nitric acid; the game has no
    farms), salt and alkali (Solvay soda-lime glass; glass already exists), smaller families (synthetic rubber, fibreglass, a
    second battery). Reopen one when a recipe or a town order asks for it. Elements these would light stay dark in the codex.
- **PUMPDOWN: pumping out a flooded shaft whose breach was sealed** (analysis done 2026-11-16, revised the same day after the
  user's two points: the ocean test must be very efficient, and an outlet pouring into the ocean just deletes the water; build
  step not scheduled). *Findings.* `sim/water.rs` `water_rule`: at or below `SEA_LEVEL` (62) an air or flowing cell next to a
  source (sideways or above) becomes a source, and the rule never asks whether that source is connected to the sea, so a
  cut-off pool is as un-pumpable as the sea. Measured with a throwaway headless test on the `sim/water` harness (removed
  again): a shaft with a tunnel from the sea, flooded, the tunnel plugged, a pump emulated taking the highest source every 10
  ticks: a 1-wide shaft with the plug against it drains completely; 2-wide and 3-wide shafts, after 400 removals, hold all 12
  and 27 sources (each hole refills from a neighbour within `WATER_DELAY` = 5 ticks). Also: water does not rise to sea level
  inside a shaft; a layer floods only where a source touches it sideways, so it stops at the breach's height. *Verdict:*
  feasible and cheap; the missing piece is one bit of information, which water is the sea.
  *Design: store the answer in the block, so asking is one read.* At or below `SEA_LEVEL` there are two source blocks: `WATER`
  (the ocean feeds it: an invariant) and `STILL_WATER` (appended id; cut off). The sea rule copies only `WATER`; pumps take
  both; the look, light and swimming treat them alike (about 32 `WATER` mentions in 16 files, some only textures and models;
  the `LIQUID` table). Three rules keep the bit true:
  1. **Outlet into the ocean:** a unit that lands on `WATER` at or below `SEA_LEVEL` is consumed and nothing is placed
     (infinity plus a finite amount); landing on a still pool or above sea level adds a source as today, of the neighbour's kind.
  2. **Cut:** when the core replaces a liquid cell at or below `SEA_LEVEL` by a solid (a block placed in water; `Sim::water_changed`
     sees every edit; pumps and outlets do not count, a hole in a sea-fed body refills), run the **lockstep fragment search**:
     start a search from each liquid neighbour of the cell and advance them one cell at a time in turn; searches that touch
     merge into one group. A group whose searches all ran dry is a cut-off fragment: its sources become `STILL_WATER`. Because the
     body was ocean-fed before the cut, at most one fragment still is, so stop as soon as only one group is still running (it is
     the ocean's side). The ocean's size never matters: the work is about two cell visits per cell of the cut-off fragments. A
     total cap (20,000 visits) leaves an unresolved body as `WATER`, which is today's behaviour.
  3. **Merge:** a `STILL_WATER` cell touching `WATER` on any face becomes `WATER` (one local check, spreading through the water
     queue), so pulling the plug floods the shaft again.
  *Old saves* hold cut-off pools as `WATER` and will never see a cut event. A pump whose intake is `WATER` at or below
  `SEA_LEVEL` checks once per removal with a goal search: connected as soon as it reaches a cell in a chunk nobody has ever
  edited (the generated ocean: next to a shore, one step), cut off if the body runs dry (then it turns `STILL_WATER`), unresolved
  at the cap. No save format change (block ids only); the search reads through the `*_anywhere` accessors in a fixed order, so
  it is deterministic; the golden hashes move only in a test that cuts a pool.
  *Measured* (prototype, debug build, hand-built ocean of 28,800 cells; pops are the hardware-independent number): a
  31-cell room cut off from the ocean, 62 visits; a 20,392-cell pool cut off, 40,784 visits (2.0 per cell, 11 ms with an
  unoptimised prototype that rebuilds a set every round; plain breadth first over the same pool measured 2.3 ms); a block
  put into the open ocean, 6; one of two channels cut (still joined round the loop), 2,492; with a cap of 5,000 the big pool
  is left alone. *Rejected:* a search inside the water rule (256 checks a tick, one search each: a cave flood becomes very
  expensive); a saved set of drained cells (new hashed state and clearing rules).
  *Still true:* time is cells ÷ the slower of pump rate and outlet (`OUTLET_RATE` 4 a second): 540 cells (3×3, 60 deep) take
  at least 135 s. *Done when (if built):* a headless test floods a 3×3 shaft, plugs it, pumps it empty into the ocean through an
  outlet (the sea unchanged); removing the plug floods it again; an old-save pool is found cut off by its pump; the sea still
  cannot be lowered and the big-flood test passes unchanged; the cut search is benched in release.

## Codex: a periodic table and achievements (user, 2026-11-16; a larger new aspect: a design step first)

The idea: an unlocking periodic table where the player sees which elements they have discovered and synthesised, and in
which forms; achievements beside it. It also makes the game teach, and gives the long chemistry tree a map.
- **One small piece of state, everything else derived.** A saved, world-wide bitset of *items ever obtained* (every item and
  block id, about 500 bits; world-wide like `Sim::taught`, so co-op shares it): set when an item first enters a player
  inventory or a machine's output: fold the core's `SimEvent::Produced` (every producer emits it already) plus hand crafts
  and pickups in `Sim::step`, one place, no hooks at call sites; a scan every 30 ticks would miss items that pass through. The table, the element cards and the achievements are *views* of that set plus the research state: no
  per-feature state, no second source of truth.
- **Data, not code.** One table maps an item or block to (element, form): ore, oxide, carbonate, pure element, alloy, compound,
  gas, liquid. Element rows hold symbol, number, a real-world fact and what it does in this game. The card shows each form as
  *unknown / seen / made*. All 118 cells are drawn; the ones the game can reach are lit by era, the rest stay dark silhouettes
  ("not in this world yet"), so the table promises depth without promising content.
- **Achievements** are data rows with a test over the same set (make 10 elements, first steel, first liquid gas, first launch,
  a whole period). *Decided (user, 2026-11-16):* they reward nothing beyond the card and a toast, in line with the binding
  decision "no more perk layers" (recorded in `docs/DECISIONS.md`).
- **UI** follows "show, don't write": symbols, colours by form and era, progress rings; words only in the card. One module
  in `ui/`, a chapter of the menu beside Research; a core module of about 100 lines for the set and the scan.
- *Cost:* wasm +1 KB or so, no frame cost (a scan every 30 ticks over the player inventories and machine outputs). *Done when
  (design step):* the (element, form) table for every item that exists, which elements the final tree can reach, the
  achievement list, the user's answer on rewards; then two steps: the core set and its test (deterministic, saved, co-op equal
  hashes), then the table UI. Pairs with CHEM (its recipes are what lights the table) and the first-steps lessons (`L1`).
