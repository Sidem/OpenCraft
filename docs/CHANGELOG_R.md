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

- [x] **S1 The real night sky** (built 2026-10-17; user request "an actual night sky, real constellations, the Milky Way"). Presentation only:
  no engine, save or generator change. `scripts/bake-sky/bake-sky.py` bakes `web/src/render/sky/stars.bin` (HYG v4.4, 8,920 stars to
  magnitude 6.5, 6 bytes each) and `milkyway.jpg` (NASA SVS Deep Star Map 2020, 2048 x 1024, 320 KB) plus 89 hand-drawn line segments for 18
  constellations (`constellations.txt`); it refuses a line naming a missing star or spanning under 0.8 or over 40 degrees, and checks the
  picture's orientation at the galactic centre, the anticentre and the Large Magellanic Cloud. Sources and licences (CC BY-SA 4.0
  for the star file): `web/src/render/sky/CREDITS.md`. `sky/astro.ts` is the tested maths: the observer is the game's sun model (an
  equator observer with the sun at declination -19 degrees), `skyFrame(days)` gives the turn, the moon (phase cycle `MOON_DAYS` = 8 game days,
  a full moon at midnight) and the sun; the stars come round once per `YEAR_DAYS` = 96 game days. `sky/stars.ts` draws the stars in one
  `POINTS` draw and the lines in one `LINES` draw (no CPU work per star); the sky shader samples the Milky Way with a fixed mip. Settings
  (`sky/settings.ts`, `ui/sky.ts`, menu Settings > Sky): Classic / Stars / Stars and Milky Way, constellation lines, twinkling.
  **Checked:** `web/test/astro.test.mjs` (rotation, the sun equals `skyAt`, rise/transit/set, Orion not mirrored, moon phases, picture
  orientation, the star file); in the pane the galactic centre, the Great Rift and a crescent moon with earthshine show where they should;
  Classic is byte-identical to the old sky (FNV hash of 20 frames at 640 x 480: five times of day, four directions, satellites on).
  **Cost** (pane, 1383 x 1222, hardware GL, sky pass alone, `finish`-timed): Classic 0.11 ms, Stars 0.14, Stars and Milky Way 0.19 (up to 0.46 while
  the GPU clocks ramp). **Left:** moon phases do not change the terrain's moonlight (still full-strength moon colour); lines cover 18 constellations.
- [x] **R5c The vault** (built 2026-10-17, generator version 10). About one ruin in five (`VAULT_ONE_IN`, never the spawn's first) has a
  sealed 3 × 3 × 2 room east of the shaft's chamber, behind a two-block Vault Door (block 128, unbreakable, `worldgen/ruins.rs`). Right-click
  opens a panel (`ui/vault.ts`): six dials of digits 0 to 4, the first ring on the right, the dials' number times itself (the
  engine's `vault_probe`) set against the door's `…444444` (-1 in base 5). A ring lights when its digits and those before agree, so the first
  ring takes 2 or 3 and then each has one right digit (2, 12, 212, …; two numbers open it, one the negative of the other, tested over all 15,625).
  `Action::OpenVault {pos, dials}` (codec tag 59) is validated by the engine (`sim/vault.rs`: door at `pos`, `lit_rings == RINGS`), clears both
  door blocks and drops the loot (5 circuits, 4 steel plates, a motor, 8 scrap). The vault is no core state. **Hint:** a far-ruin logbook
  fragment says doors of rings exist and the rings must multiply by themselves to match the fours. **Checks:** `sim/vault/tests.rs`,
  `worldgen/ruins/tests.rs` (a sealed room of exactly 18 cells of air; frequency 10 to 30% on four seeds; none at version 9), the codec
  test. **Seen in the pane** (seed 1337, door at x -119, y 76, z -497; `where_the_ruins_stand` prints the vaults): solved ring by ring, the
  loot reached the hotbar. **Not built:** discoveries with names (no such module yet). The door's look is a placeholder (`ART_HANDOVER.md`).

- [x] **R5b Logbooks** (built 2026-10-17, generator version 9). Block 127, unbreakable, stands on every ruin's slab; right-click opens a
  panel (`ui/logbook.ts`, through `take_panel_request` and `Game::logbook`). A page is a pure function of the seed and the ruin's spot
  (`worldgen/logbook.rs`), never saved. **Pages:** a bearing (the nearest ground that holds an ore, from `bearing_to`, as "about N
  blocks north-east" with a Pin button that adds an ore pin), a tip naming real recipes (stained soil, hand mining, scrap, concrete,
  motor, bricks and glass), and from ruins 700 or more blocks from spawn story fragments (the tower to the sky, spent rock, the
  reactor) and bearings to bauxite, uranium and oil sand. **Finding:** the common ores lie under every biome (nearest iron, coal and
  limestone were 0 blocks away from every spot tried, copper up to 107, quartz 48 to 173), so near pages point mostly at quartz and
  otherwise give a tip; the roadmap's "bearing to an ore's biome" is only useful for the rare ores. **Deviation:** a block, not an item.
  **Checks:** `worldgen/logbook/tests.rs` (a logbook on every test seed's first ruin and nowhere else; near pages never tell the late
  story; far ones do; a bearing points at ground that holds the ore and the text's distance and compass point are right; pages depend
  on seed and spot only; compass), `api/ruins/tests.rs`. **Seen in the pane:** right-click opened the panel; Pin put an ore-34 pin at
  the page's ground. Old R5a ruins in a saved test world lack the logbook (the chunk was edited before it existed; worlds on the live
  site never saw that version).

- [x] **R5a Ruins of the last factory: sites, rusted blocks, scrap** (built 2026-10-17, generator version 9 so new worlds only; v9 is
  pinned when pushed with R5b). **Where:** `worldgen/ruins.rs`: 512 × 512 regions centred on spawn, each with at most one
  ruin from up to 8 hashed tries (flat within 3 blocks over nine samples, dry, above sea level + 1); the spawn's region rings spawn at
  150 to 297 blocks, so the first ruin is a short walk; a pure function of the seed, cached per region, so chunk order never matters.
  **What:** a cracked concrete slab (30 to 55% cracks), three machine husks (sometimes a second block or a plating cap), two girder
  poles, a broken plating belt run, a 2 × 2 shaft 9 to 13 blocks deep with rubble, ending in a spent-rock chamber with one more husk;
  trees keep off. Blocks 124–126 (`block/ruins.rs`, generated, never held), Scrap Metal (item 396), a smelter row (135): 3 scrap to
  an iron ingot. Breaking a rusted block gives scrap (1; girder 1 to 2; husk 2 to 4) and a husk a 25% circuit (`sim/scrap.rs`, core
  `rng`). **Deviation from the roadmap:** no "feed rusted blocks to the recycler" (they cannot be held); scrap is the loop instead.
  **Checks:** `worldgen/ruins/tests.rs` (first ruin 150 to 300 blocks out for 200 seeds; candidates stay in their region; parts and
  reach; slab follows slopes; every test seed has a first ruin on flat dry land; at least 20 of 49 regions have one; every block reads
  back from a second generator asked in another order; no trunk on the slab; none before version 9), `sim/scrap/tests.rs`, the
  recipe lint (`GATHERED`). **Seen in the pane** (seed 1337, ruin at x -174, z 30; `where_the_ruins_stand`, an ignored test, prints the spots): the slab,
  girders, plating run and shaft; breaking a husk by hand gave 4 Scrap Metal in the hotbar. **Not built:** logbooks (R5b), the vault (R5c).
  Looks are placeholders (`docs/ART_HANDOVER.md`).

- [x] **R4 Music by context** (built 2026-10-17, presentation only; user: "the music shouldn't be that big", compose the pieces in the
  MusicComposer project). **Code:** `audio/surroundings.ts` (the float layout, `daylight`, `contextFor`: underground beats night beats
  biome beats sea), `audio/music-director.ts` (pure: a place holds `HOLD` 20 s before music follows, first piece after 10 to 30 s, then
  2 to 5 minutes of silence between pieces in By place and 4 to 12 s in the "always" modes, never the piece that just played, a
  playing piece that does not fit the new place or mode is faded into one that does, a piece that fails to load is never chosen again,
  machines lower the music up to 30%), `audio/music-pieces.ts` (rows: file, mood, places, `'any'` fallback, gain), `audio/music.ts`
  (`MODES`; streams each piece through a media element into the AudioContext, crossfades, pauses while hushed, no decoding: a decoded
  loop was 67 and 82 MB). Saved mode: the old "adventurous" track becomes Always adventurous, "ambient" (the old default) becomes By
  place. **Pieces** (composed with LilyPond and `render_midi.py`, `MusicComposer/opencraft/music-by-context`, all about -18 LUFS):
  Lantern Hours (night, A minor pad, celesta, harp, flute), Deep Seam (caves, C phrygian drone, low clarinet, drips), Salt Road (desert,
  D phrygian dominant, frame drums, oboe, accordion), High Pasture (highlands, D mixolydian, horn, flute, strings); the two Open Ground
  versions are the day pieces and the fallback for the sea. 64 kbit/s Opus (first shipped at 24, which measured clearly worse than the old Vorbis loops: spectral error 4.5 and 7.9 dB against 2.6 and 3.2): 1.3 to 2.1 MB a piece, 9.3 MB for all six (two Vorbis loops were 5.8 MB). **Checks:** `web/test/music-director.test.mjs` (13 tests on scripted walks with a fake clock; breaking the hold makes two
  fail), `scripts/check-music.mjs` (rows and files agree, at most 2.5 MB a file, loudness within 2 LU), both in `npm run check`. Seen
  in the pane: the first piece after 10 s, the cave piece fades in 20 s after going underground, only the pieces played are fetched
  (range requests), no `decodeAudioData`. **Not heard by the agent:** how the pieces sound; levels and tunes want the user's ear (`gain`
  per row trims a piece). Not built: the era as a context, a sea piece.
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
