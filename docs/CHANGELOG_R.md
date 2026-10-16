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
