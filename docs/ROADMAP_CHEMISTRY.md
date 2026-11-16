# OpenCraft roadmap: chemistry, water and the codex (requests from play, 2026-11-16)

Unordered requests, kept apart from `docs/ROADMAP.md` and `docs/ROADMAP_REQUESTS.md` to keep each under its budget; pointed to
from `docs/DEV_PLAN.md` section 4. When one is built, move it into the plan, then drop it here. Rules for all: plan section 3.1
(small modules, content is data), section 3.4 (determinism), a placeholder layer and an `ART_HANDOVER.md` line for each new look.

## Chemistry and water (unordered)

- **CHEM: expand the chemical tech and recipe tree, and make it teach** (a design step first, then one family per session).
  Today (`research/chemistry.rs`, `docs/TECH_ERAS.md` section 6): Oil Processing → Refining → Plastics → Acids and Lubricants →
  Electrolysis, Ore Washing, Diesel Power; fluids ride belts as barrels (crude, naphtha, diesel, heavy oil, lubricant, acid,
  hydrogen, oxygen); rocket fuel is 2 hydrogen + 1 oxygen barrels. *Step 0 done, 2026-11-16:* `scripts/recipe-graph.mjs` (it
  reads the built wasm: `npm run build:wasm` first) lists every item and recipe (375 named items, 331 recipes: 176 by hand,
  155 in machines, 52 of those upgrader rows) and the loose ends. It found: (1) **Lubricant Barrel has no consumer anywhere**
  in the code (`TECH_ERAS.md` promises "Mk5 kits and gears"; no recipe takes it): the heavy-oil → lubricant branch is a dead
  end, so give it a use in the design step or drop the recipe. (2) Chemistry is thin: the chemical plant has 5 recipes, the
  refinery, cracker, electrolyser and centrifuge one each, and every product has one sink: naphtha → plastic only, acid and
  pure water → the chip fab only, hydrogen and oxygen → rocket fuel only, sulfur → acid and yellow concrete (diesel →
  generators and fuel cells → the reactor are fuels, outside the graph). (3) The pigment idea already lives in the coloured
  concretes (`recipes/decor.rs`; see GLASS in `docs/ROADMAP_REQUESTS.md`). (4) The eight extra woods and the Recycling Coin
  are unused by design. *Blind spot:* fuels, placing a building, town orders and drone fuel are not recipe inputs, so each
  "made, used nowhere" line is a question, not a verdict. New families should land on these gaps (a second sink for each
  product) before adding more stock. *Rules for every family:* it
  answers a problem the player has now (needs before techs; and no multiplier layers: cooling, fertiliser and the like solve a
  problem, they do not add a percentage); it uses what exists (iron, copper, coal, limestone, sand, quartz, bauxite, uranium,
  oil, sulfur, water, sea water, air), the one exception being natural gas fields (family 3, new worlds only); old recipes and saves are untouched (a new stream or product
  comes with a new recipe row or machine, never by changing `Distil`); fluids stay barrels; recipes are data rows. *Teaching:* a
  recipe's hover card names the real process and its conditions in one line (Haber–Bosch: N₂ + 3 H₂ → 2 NH₃, about 200 bar,
  450 °C, iron catalyst), and the codex (below) lists what the player has made. Numbers are real where they cost nothing (N₂
  boils at −196 °C, O₂ at −183 °C, H₂ at −253 °C, He at −269 °C, CO₂ sublimes at −78 °C). *Families, most fitting first (none
  decided; the user ranks them in the design step):*
  1. **Dyes and pigments** (decided, see GLASS in `docs/ROADMAP_REQUESTS.md`): a base-and-mix tree; metal oxides colour stained glass, as in life.
  2. **Air and gas capture.** An *air compressor* feeds a *distillation column* (cryogenic, Linde: compress, cool, expand, separate
     by boiling point): nitrogen, oxygen, argon barrels, and a trace of CO₂. Sinks: nitrogen and argon for the chip fab's clean
     room and for welding aluminium; oxygen for steel (blast furnace yield) and rocket fuel. *Carbon capture* beside generators and
     smelters gives CO₂ (and is the natural cleaner if pollution ever returns: parked R8). Helium is not in the air in a
     useful amount: it comes from natural gas (next).
  3. **Natural gas** (user, 2026-11-16: helium is a by-product of natural gas in the real world, so do the same). *Where the gas
     comes from:* (a) **gas fields**: a new ore row in `EXTRAS` (`worldgen/geology.rs`; its `since` is the next generator version,
     so new worlds only, as oil and uranium were at version 6), found by scanning, drilled by a *gas well* (a pumpjack variant)
     into *natural gas barrels*; (b) **associated gas** so existing worlds get it too: a *wellhead separator* takes a
     pumpjack's crude barrels and gives crude plus gas barrels, at a gas-to-oil ratio taken from a hash of the deposit. Old
     worlds get gas fields through the deposits-layer upgrade (decided: WORLDS in `docs/ROADMAP_REQUESTS.md`); (b) is then
     optional, for old worlds that do not upgrade.
     *What a gas plant does* (one machine, one recipe per step, the real processes named on the cards): sweetening (H₂S → sulfur,
     Claus: a sulfur source apart from the refinery), then separation into **methane**, **ethane**, **propane and butane
     (LPG)** and **nitrogen**, and a cryogenic *nitrogen-rejection* step that pulls out **helium** (it stays gas to 4 K, the
     rest liquefies first; needs the chiller below). Helium is rare on purpose: a field's fraction comes from its hash, most
     under 0.1 %, a few around 1–2 % (balance numbers to tune; the scanner and core drill show it). *Sinks, the point of the
     family:* methane burns in a **gas turbine** (a power source beside diesel) and is the feed for **steam methane
     reforming** (CH₄ + H₂O → CO + 3 H₂, shift CO + H₂O → CO₂ + H₂: hydrogen without the 500 kW electrolyser, CO₂ out, the
     real source of most hydrogen) and for methanol; ethane → ethylene (steam cracking) → **polyethylene**, a plastic that does
     not use naphtha; LPG for drone and vehicle fuel and gas-fired smelting; **LNG** (methane at −162 °C in insulated barrels)
     is the answer to far gas fields (pillar 5, distance), carried by train or cargo drone.
  4. **Cooling and heating** (`heat` already flows into coolant loops for datacenters; reuse it). A *chiller* (compressor,
     condenser rejecting heat to a cooling loop, evaporator) takes power and a loop and produces cold; a *heat exchanger* and
     *recuperator* return heat (steam, smelters). Cold makes: **liquid oxygen and hydrogen** (rocket fuel gets denser: fewer
     barrels per launch; an *insulated barrel*, a steel-glass-plastic flask, holds them and has no boil-off, said in the
     manual), **liquid nitrogen** (a water-free datacenter loop for deserts, still a cost, not a multiplier), **liquid helium**
     (from natural gas; 4 K: superconducting cable for lossless long-distance lines and maglev rails; confirm there is a loss to remove, else a
     longer span), **dry ice** (CO₂ at −78 °C: fire suppression for FIRE, a cold chain for town orders, fog for the look),
     **CO₂ as feedstock** (Sabatier: CO₂ + H₂ → methane; methanol) and for greenhouses (the forester's growth).
  5. **Nitrogen, Haber–Bosch and the nitrogen compounds.** Nitrogen from the column + hydrogen from the electrolyser (or from
     reforming, family 3) → ammonia
     (200 bar, 450 °C, an *iron catalyst* that wears out slowly, promoted with alumina: a use for bauxite). Ammonia → fertiliser
     (the forester's saplings, town order goods), → nitric acid (Ostwald: ammonia + oxygen) → ammonium nitrate, etching acid, and a
     hydrogen carrier or refrigerant. The first reason for the air column; it makes the electrolyser's hydrogen matter.
  6. **Salt, alkali and glass** (a machine at the sea takes sea water for salt and fresh water; brine electrolysis: chlorine, lye,
     hydrogen; Solvay: salt + limestone + ammonia → soda ash): soda-lime glass the way it is really made (sand, soda ash,
     limestone), lye for soap, chlorine for a second plastic. Needs no deposit, only the sea.
  7. **Carbon and silicon.** Coke from coal (blast-furnace yield); carbon fibre (heavy oil + plastic) for light frames and
     Milestone 14's tether, a sink for heavy oil; silicon the real way (quartz + coke → silicon → trichlorosilane → distilled →
     ultra-pure wafers) as a higher-yield path beside the existing wafer recipe, not a replacement.
  8. **Smaller:** synthetic rubber, fibreglass, a second battery chemistry (`MAT_RUBBER`, `BATTERY` models exist); only when a
     recipe asks for them.
  *Done when (the design step):* the dead-end report is in the plan; a table of accepted families, each with the problem it
  solves, its tech's place (`needs`, packs), its machines and recipes, and the one-line real process for its cards; the user's
  ranking; a decision on boil-off (none) and on whether gases need a pressure/cold container type of their own; then each
  family is a step with its own done-when.
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
  inventory or a machine's output (found by a deterministic scan every few ticks, not by hooks at every call site, so the
  core stays one module). The table, the element cards and the achievements are *views* of that set plus the research state: no
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
