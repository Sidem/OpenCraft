# OpenCraft roadmap (after Milestone 8 is planned)

Read this only when a milestone ends and the next one is being planned. `docs/DEV_PLAN.md` details the
**current** milestone only. At each milestone's cleanup step, move the next milestone from here into the
plan and detail it there (steps with where, how and done-when).

**Agreed order (2026-10-16, after Claude's whole-game review and the user's play-test):** the play-test response, steps
R1–R8 in DEV_PLAN section 4 (why the middle systems go unused, plants, ambient sound, music by context, the ruins,
settlements and the megaproject theme, a decor set, pressure without combat), then the feel-and-feedback interlude, the
carry's era pieces (trimmed), the megaproject. The designs the R steps need are in the section "The play-test response" below.
**Updated after R1 and R6 (2026-10-16):** settlements became Milestone 13 (DEV_PLAN section 4) and come next; then the rest of
R2–R8, the interlude, the carry; the megaproject is Milestone 14, a space elevator.

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
  and a setting turns it off. **Limits (2026-10-16):** only the few nearest families sound at once, a hard distance
  cap, and the music (R4) quiets while it plays, so a big base never turns into noise. Lives in `web/src/audio/` from the existing sound events. Later, maybe: a
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

## The play-test response: designs for steps R1–R8 (agreed 2026-10-16)

The steps, their order and done-when are in DEV_PLAN section 4; the findings that led here are there too. Each design below
is a sketch to settle with the user when its step comes up.

- **A living world (R2–R4).** The world is the reason to go outward (pillar 2), so it must be worth looking at and listening
  to, not only mined. Plants per biome and tree species, indicator plants over ore, procedural ambience, music by context with
  silences. Plants are looks first, data second: ground cover is presentation only; species and indicator plants are
  worldgen rows. Later, with pollution (R8), plants thin around a dirty base and come back when it is cleaned.
- **Settlements as customers (R6; decided 2026-10-16, now Milestone 13 in DEV_PLAN).** The user's picks: the player founds
  each town with a depot at a site that fits its kind (no worldgen towns, so existing worlds get them), a perk per town kind,
  and a stalled town (upkeep unmet) pauses its perk but never shrinks. The sketch as written before the picks: the missing
  "what for": today output goes only into packs and kits. A settlement is a
  site (placed by worldgen near spawn and in far biomes, or founded by the player) with a list of orders (bricks, glass,
  lamps, steel, concrete; later plastics, chips) delivered by belt, train, cargo drone or drop pod into its depot. Supplied, it
  grows in stages: each stage is a blueprint its own drones build, so it reuses the blueprint and drone systems and shows the
  player what they can do. Bigger stages order more, so demand grows with the town and pulls trains and drones in (R1). What a
  town gives back: a perk per town kind (a market that trades bulk for rare items, a workshop that crafts kits, a station that
  extends drone range), never combat. No walking townspeople (lights, smoke and building is enough life), no diplomacy or war.
  Core state: orders and stage per settlement, deterministic. The megaproject may be the settlements' joint project (a colony
  ship, a space elevator they build up); decide with the user in R6.
- **Decor building set (R7; built 2026-10-17 as 14 machine-made cubes, see `docs/CHANGELOG_R.md`; slabs, stairs and beams remain).** Factory players love making a base beautiful, and the world is the material (pillar 1).
  Machine-made rows in a Decor group: concrete in about eight colours (dyes from minerals already mined), brick kinds, framed
  glass, metal panels, catwalks and railings (cutout), light fixtures in a few shapes. Later: slabs, stairs and beams need
  non-cube meshing and hit boxes, a step of their own. Pure content, no new mechanics.
- **Pressure without combat (R8).** What enemies give Factorio is pressure: the factory can't be left alone. Peaceful options,
  all deterministic and built on existing systems: **subsidence** (hollowed lodes collapse unless backfilled or supported;
  already in "Later / maybe"), **groundwater** (deep mines below the water table seep and need pumps), **storms** (rare, from
  the core tick; lines in the open stop or wear unless roofed), **pollution** (smelters and generators spread a haze value
  per region that thins plants and dims solar; scrubbers clean it). Pick one or two; none may destroy the player's work.

## Ruins of the last factory (step R5; chosen by the user 2026-10-08, moved up 2026-10-16)

Placed before the megaproject so its logbooks can foreshadow it (whose theme R6 decides); it needs only the recycler and a
new generator version (shared with R2b's if that is not released yet).

- **Ruins (M).** Rare worldgen sites (new worlds only, a generator version bump): rusted machines, belts
  and poles of builders who came before and ran their ground dry, over collapsed shafts down to nearly
  exhausted lodes. That tells the finite-ground story (pillar 2) without words. Rusted blocks are their
  own ids; breaking them or feeding them to the recycler gives scrap metal and the odd circuit.
- **Logbooks** (an item with a reading panel) give real help: a bearing to an ore's biome, a recipe or tip
  ahead of time, a pin on the map. Near ruins hold early hints, far ones fragments about the late tech
  tree and the megaproject. The first ruin is 150–300 blocks from spawn, then about one per 512×512
  region. Placement is a pure function of the seed, like deposits.
- **One sealed vault** from the thread below (the carry), as an easter egg with modest loot. The ruins serve every
  player: story and loot first (trimmed 2026-10-16).

## Thread: the carry (p-adic numbers and fractals, by discovery; chosen 2026-10-11, trimmed 2026-10-16)

Mechanisms that behave exactly like p-adic numbers, so players find the ideas by playing. **Trimmed on 2026-10-16** (the
user agreed with Claude's review: the puzzles cost a lot of design for a small audience, and "names only, never explained"
would leave most players at a locked door): two cheap era pieces that come out of normal play, and one vault in the ruins
as an easter egg. Dropped: the 3-door, the 10-door, the Cantor sieve, the Hanoi crane and the layer printer. Parked, only if
a later step wants more depth for datacenters or comms: fractal radiators with the analyzer, and antenna bands.

- **Pure discovery, names only** (user's choice). When a player does the thing, a discovery shows only its name ("3-adic
  address"); digits are shown right to left with a leading "…" ("…0110"), so carries and endless digits are visible.
- **The thread, for agents** (never shown in game): a binary counter's carry length is the ruler sequence 1 2 1 3 1 2 1 4;
  a carry that never ends gives …1111 = −1, so 1 + 2 + 4 + … = −1 in the 2-adics. Two numbers are p-adically close when
  their last digits agree, which is a tree.
- **Discoveries (S).** One module: a list of named entries, each a pure check on state, a toast with the name, a page in the
  menu. Decide at planning whether it is core state (shared per world in co-op, a save bump) or kept beside the save.
- **Splitter addresses (S, primes 2 and 3, presentation only).** A splitter round-robins its outputs (`factory/router.rs`),
  so a cascade sends item n to leaf n mod 2^k (two outputs) or n mod 3^k (three). Looking at a belt fed by a tree at least 3
  deep shows its address, first split rightmost ("…0110", "…210"). Discoveries "2-adic integers", "3-adic integers".
- **The tally (S, Electronics retrofit, prime 2, core state).** A logic block that counts the box or belt behind it: +1 for
  each item in, −1 for each item out, shown on 16 binary lamps. Below zero it rolls to all lamps lit (…1111). A sensor rule
  can fire on one bit, so it doubles as a divider ("every 8th item"). Discoveries "2-adic −1", "two's complement".
- **The vault (ruins, the one easter egg, prime 5).** A door of rings, each a dial of 5 digits, showing …4444 (−1 in base
  5). Ring k opens when the number on rings 1..k, squared, ends in the target's last k digits: the first ring is found by
  trial (2 or 3), then each ring has exactly one right digit (2 → 12 → 212 → …). Modest loot; rewards stop after about 6
  rings. Discoveries "5-adic integers", "√−1".

**Order:** discoveries, splitter addresses and the tally after the feel-and-feedback interlude; the vault with the ruins (R5).

## Milestone 14: The megaproject, a space elevator (era 10; optional, non-ending; theme chosen 2026-10-16)

Milestone 13 (Settlements) took the number 13. Optional, built in phases, physically in the world, needing enormous
resources; completing it **unlocks** something rather than ending the game. Play continues.
- **Where:** it rises from the player's biggest town (a town stage unlocks its foundation), so the towns are its workforce
  and their supply routes its supply routes.
- **Phases** (a first sketch): the anchor and foundation, the tether climbing in sections (each a large order delivered to
  its base: steel, aluminium, plastics, chips, rocket fuel for the climbers), a counterweight station sent up by rocket,
  the climber cars. Each finished section shows: the tower grows visibly into the sky, seen from far away, lit at night.
- **What finishing unlocks:** asteroid mining. Climbers carry mining craft up; asteroid ore returns by drop pod to landing
  pads, resources beyond the finite ground (pillar 2's answer at the end).
- **Engine notes for planning:** a very tall structure must stay cheap to draw (boxes or a dedicated mesh, not 4,000 blocks),
  and its sections are core state like a machine's; the sky already draws satellites (`render/sky.ts`).

## Later / maybe

- A tower-defence mode as a world setting (the user's idea; explicitly not now; 2026-10-16: still peaceful, no combat).
- Subsidence: hollowed-out lodes collapse the ground above unless backfilled or supported (an R8 candidate).
- Slabs, stairs and beams for building (after R7's decor set; needs non-cube meshing).
- A world settings screen: ore richness, deposit size, peaceful toggle.
- Catch-up on return: factories run forward for the time the game was closed, capped.
- Weather, mod support (the data-driven content tables are the first step towards mods).
- A dedicated server: the engine crate compiles natively (feature-gate wasm-bindgen) and speaks WebSocket,
  with `libm` on both sides for identical math. Co-op itself is player-hosted (DEV_PLAN section 2).
