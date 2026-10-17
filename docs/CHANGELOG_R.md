# The play-test response: findings and finished steps

Moved from `docs/DEV_PLAN.md` on 2026-10-16 (after R1c), to keep the plan under its budget. The open R steps stay in
the plan's section 4. Each record says what was built or agreed.

## Play-test findings (the user's save, 2026-10-16)

The user has played seed 1337 in a normal world for **31.9 hours** (day 97) and enjoys it. Read headlessly from the save (Node
loading `web/src/wasm`, `Game.load`, the tech and analytics getters):
- **78 of 84 techs done**: everything but the three endless bonus techs, Orbital Comms, Orbital Power and Drop Pods. Construction
  drones, swarms, trains, freight, cargo drones, the hover pack, Rocketry and Orbital Science are all researched.
- **Researched but never used** (the user's words): drones, trains, the planner and blueprints; no rocket built (the Launch Site
  sits in the inventory).
- **97 machines** in the analytics: 37 at full speed, **38 held back by full output**, 15 waiting for input, 7 by their deposit.
  Assemblers (18) average 10%, miners (13) 11%, the two refineries 0%. The base makes more than anything uses.
- **22 deposits** touched in 32 hours. The inventory is a store: about 800 belts, stacks of kits, logs, stone and coal.

**Reading:** research outran need. The tree can be finished while the problems its techs solve (distance, scale, repetition)
never arrive, so pillar 5 ("every tier adds a new kind of problem") does not bite in the middle and late game. Likely causes:
carrying is cheap (hauler packs up to 99 slots, the hover pack), so a far deposit is a trip, not a route; demand is small, so
nothing has to scale; deposits last, so distance never presses; every build is one-off, so blueprints don't pay.

## Finished steps

- [x] **R3 Ambient sound** (built 2026-10-17, presentation only, every world): `Game::surroundings` (`surroundings/mod.rs`, nine
  floats read from the loaded blocks: biome, sky overhead, openness, height above sea level, depth below the surface, sea, fresh
  water, trees, din of working machines via `Factory::din_near`; about 0.01 ms a call) is polled four times a second by
  `web/src/audio/ambience.ts`. `mixFor` turns it and the time of day into levels: wind (rising with height and openness, dry and
  stronger over deserts and basalt), surf (still water at sea level), rushing fresh water (rivers, falls, lakes above the sea),
  a low room tone underground, and one-shots at random by level: birds by day near trees, crickets at night on green ground,
  cave drips (`synth-ambience.ts`, all procedural). Working machines quiet everything by up to 55%. A third channel on the sound
  system with its own slider and mute on the Sound page (K mutes it too); it follows the "mute sound effects when not in focus"
  box. Tests: `surroundings/tests.rs` (sky, roofs and leaves, depth, sea against fresh water, trees, din with distance). Seen in the
  pane: spawn plains calm, the desert coast surf with dry wind, a cave hum and drips, a river's rush, pines' wind on the highlands,
  birds (7 a minute in woodland) by day and crickets (20) at night. Not heard by the agent: levels want tuning by ear.
- [x] **R2c Indicator plants** (built 2026-10-17, presentation only, every world from version 2): over each stained soil or sand (blocks
  36–43: rusty = iron, dark = coal, green = copper, pale = limestone and quartz) `mesher/cover.rs` grows that stain's own plant on 45% of
  the columns (`tex::INDICATOR_IRON..=INDICATOR_PALE`, 322–325: a rust-orange bloom, a dark shrub, a tall teal bell, pale stars), so
  they follow the stains, which follow the veins and lodes (`geology.rs`), and never grow elsewhere (`cover/tests.rs`). Pointing at one
  reads the stained soil under it ("Stains like this lie over ore veins and lodes"; the existing depth reading); the ore guide's note on
  stained soil names the plants. No new state.
- [x] **R2b Tree species (generator version 9, built 2026-10-17, new worlds only):** `worldgen/trees.rs` rows: oak (4–6) and birch (5–7,
  white bark) on plains (70/30), broad low trees (3–4 trunk, canopy 3 out) in lowlands (60%, rest oak/birch), pines (6–8, a cone of
  needles) on highlands (90%), cacti on desert sand and dead snags on basalt (flat 7% and 5% of cells; the others follow the forest noise). Blocks 117–123
  (`block/forest.rs`; `tex` 314–321): birch log and leaves, pine needles, cactus, dead log, birch and pine saplings. Leaves drop their own
  species' sapling (`sapling_of`), saplings grow their species (`species_of_sapling`), decay, terraform surveys and town plots treat every
  species alike (`is_log`, `is_leaves`, `is_tree`). Generated logs and leaves drop the oak's item, so every log saws into the same planks;
  the broad tree has no sapling of its own (it shares the oak's leaves). Not built: heather on basalt (left for R2c). Version 9
  is pinned when released.
- [x] **R2a Ground cover** (built 2026-10-16, presentation only, every world from generator version 2): `mesher/cover.rs` draws
  grass tufts, daisies, poppies, bluebells, ferns, reeds, dry brush and lichen (`tex` 306–313, `textures/ground_cover.rs`, plain
  placeholders) as plants on top of natural ground (grass and stained grass, dirt, sand and stained sand, basalt, granite) in empty
  cells with sky light at least 12, chosen per column by a hash and the biome (`WorldGen::biomes`, kept in the column cache; plains
  tufts and meadow flowers, lowlands ferns, highlands tufts, ferns and bluebells, deserts dry brush on sand, basalt fields lichen
  and dry brush; reeds beside water in every biome). Flowers cluster in 4 × 4 meadows. Never blocks, never saved, never in the state
  hash; a remesh redraws it, so it follows edits. `Mesher::mesh` takes `Option<&Cover>`. `bench_meshing`: 0.54 to 0.56 ms a chunk.
  Tests in `mesher/cover/tests.rs`.

- [x] **R1 Why the middle systems go unused (talked 2026-10-16).** Measured from the engine's tables: the whole tree asks for
  about 45,000 packs (10,725 red, 10,635 green, 10,290 blue, 9,420 violet, 4,180 gold; 126 lab-hours at Mk1 speed), and its
  only far material, bauxite, comes to about 2,800 ore (gold packs via batteries), some 44 stacks: **one trip with a Mk5 hauler
  pack** (99 slots). Demand ends with the tree; builds never repeat; deposits last. **The user picked (section 1):** settlements
  as growing demand (A, designed in R6, which comes next) and a blueprint prompt (D). Not picked: ore stacks of 16 (B), far
  bulk inputs in late recipes (C), accepting drones and trains as optional (E); B and C stay fallbacks if A is not enough.
- [x] **R1a Save inspector** (built 2026-10-16): `scripts/inspect-save.mjs <file.ocworld>` loads a save in Node through
  `web/src/wasm` and prints play time, techs done and available, the tree's pack demand, machines by kind with what holds them
  back, deposits touched and the inventory (`docs/WORKFLOW.md` section 1). It printed the findings above for the user's save.
- [x] **R1b Blueprint prompt** (built 2026-10-16, presentation only, never saved): `blueprint/repeat.rs`. When the local player
  places the same working machine (`Factory::works_at`: a processor, miner, generator or lab; same block and tier, since a recipe
  is chosen after placing) a third time within 32 blocks this session, `blueprint_prompt` gives a tip ("Building the same again"),
  shown ahead of the other tips by `ui/hints.ts` for 90 s (H dismisses it), and the blueprint selection is preset to a box around
  the previous copy, so B then Enter copies it. Once per kind a session; never in creative worlds or once the world has a
  blueprint. Tests: `blueprint/tests.rs` (fires on the third smelter, not for belts, once per kind, not with a blueprint, ends by
  itself). Seen in the browser pane. Also fixed: garbled "·" and "×" in five files (blueprint label, quarry readout, research
  blurbs, debug overlay). 846 tests.
- [x] **R6 Settlements and the megaproject (talked 2026-10-16, moved up).** Decisions in section 1; the build is Milestone 13
  below, the elevator Milestone 14 (`docs/ROADMAP.md`).
- [x] **R1c Natural rivers and lakes (asked and built 2026-10-16, generator version 8, new worlds only).** The user found
  rivers too straight, their steps a row of dams, and wants no perfect line or circle in generated land. Version 7 is
  pinned (`released_versions_never_change`, `version_7_rivers_never_change`); the user's world keeps it.
  - **Winding:** a reach follows a `Course` (`rivers/course.rs`): points pushed sideways by a `meander` noise (bends
    about 64 blocks, up to 30 either way, damped on steep reaches) plus a bow of 6–14% of its length, fading to the nodes.
    Widths and shores wobble by the column. Measured: every reach bends 8+ blocks, under 2% of 48-block stretches stay
    within a block of their chord.
  - **Flowing down:** no weirs. A pool ends in a drop wherever the ground needs one (at least 7 blocks apart, so steep
    ground saves its drops into a waterfall); the bed rises to a lip before it and deepens into a plunge pool after.
    `worldgen::falls` stamps falling water (`FLOW_7`) wherever water stands beside higher water: exactly what
    `sim/water.rs` settles on (`generated_falls_are_settled_water`: 679 fall cells, 3,047 checks, nothing moves). Banks
    rise above any water near, so two rivers at different levels side by side hold. Seen in the pane: a winding river
    with a 5-block waterfall over a lip at (-631, 79–84, -517), seed 1337.
  - **Shores and the audit:** lakes, ponds and ore stains over veins and lodes follow a `Shore` (`shore.rs`: three
    hashed harmonics, radius 0.4–1 of full; tests: every one's spread at least 25%); the spawn lift and the mesas' fade
    bend their distance by noise (`BENT`), so neither ends in a ring. Left as they are: mesa terraces (they follow the
    noise's contours) and the one tree shape (R2b). The lip is the ground's own block, not stone.
  - **Cost:** `bench_generation`, 432 chunks: version 8 is 10–15% over version 7, and no slower than version 7 was
    before this step's per-column sample cache (which serves both). 871 tests.
