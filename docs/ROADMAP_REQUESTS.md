# OpenCraft roadmap: requests from play (2026-11-16)

Unordered requests, one session each, kept apart from `docs/ROADMAP.md` to keep both under their budgets. Each is pointed to
from `docs/DEV_PLAN.md` section 4; when one is built, move it into the plan, then drop it here. Rules for all: plan section 3.1
(small modules, content is data), section 3.4 (determinism), a placeholder layer and an `ART_HANDOVER.md` line for each new look.

## Logistics and power polish (user, 2026-11-16; unordered, one session each; in `docs/DEV_PLAN.md` section 4 as one line)

Two requests from play left (MERGE and LOOK built 2026-11-17, plan section 6). Rules for all: new content is data and a new
module (plan 3.1), each new look gets a placeholder layer and an `ART_HANDOVER.md` line, players get a patch note and a
`FACTORY.md` / `PLAYING.md` line.

- **XRAY: X-ray goggles** (a new worn item; it solves lost underpass and lift lines). While on, what
  the player built shows through terrain, tinted. Build it in two steps. **XRAY1 logistics:** belts, underpasses, lifts, routers,
  pipes, poles and machines, which are factory records and already drawn per frame; add one overlay pass of the same instances
  with the depth test set to *greater*, so only the hidden parts draw, capped to a radius of the player. It must remesh
  nothing and add no chunk work. **XRAY2 building blocks:** the world keeps only a per-chunk `modified` flag
  (`world/mod.rs`), so "placed by the player" means "differs from generation": measure that comparison per chunk first and cache
  it per chunk as a bit set, built over the streaming budget; if it costs too much, show only what sits within a few blocks of
  the logistics. *Done when:* XRAY1 shows a buried belt line from above in the browser pane and the frame bench is unchanged
  with the goggles off (and costs are reported with them on); XRAY2 only if its measured cost is acceptable.
  *Research (user, 2026-11-16):* a blue-science tech after Hoists: a `Tech` row with `needs: &[51]` (Hoists) and
  `packs: &[RED_PACK, GREEN_PACK, BLUE_PACK]`, appended at the bottom of `techs::TECHS` (the table's order is the saved
  index; `research/chemistry.rs` shows the pattern), unlocking the goggles; the lint
  `every_ingredient_comes_from_a_tech_before_the_tech_that_needs_it` must pass for the goggles' recipe. *Decided (user,
  2026-11-16):* the goggles are **worn in an equipment slot**, not toggled from the hotbar (the key then only matters if the
  slot needs a quick off switch); XRAY1 starts by finding how worn gear (jetpack, hover pack) is held and adds the slot there.
- **WIRES: power lines that hang from the arms** (presentation only; **built 2026-11-17, v0.8.15** in `factory/pole_hang.rs`; the
  arm's yaw from the wired neighbours, the "second step" below, is not built). *Found:* every wire of an ordinary pole leaves one point
  on its centre line (`Pole::wire_point`, `POLE_WIRE_Y`), though the model has a crossarm with an insulator at each end
  (`Pole::model`); a long-distance tower's wires leave its earth-wire peak (`pole_tower::TOP`), though it has four arm tips with
  insulators and grid-coloured sleeves (`long_accent`). *Proposal:* a pole lists its **hang points**, the tops of its insulators
  (two for an ordinary pole; four for a tower: two arms at two heights, with the peak left as the lightning wire and no longer a
  hang point), one table per model next to the model's own constants so the drawing and the wire cannot drift apart.
  A wire picks its end on each pole in two rules. (1) **Side:** the insulator on the side the other end lies towards (the dot of
  the direction to the other end with the arm's axis). (2) **Level and order:** wires on one side take the free levels in the
  order the player wired them (a tower's two between the same pair run parallel, lower arm to lower arm), and when more
  wires leave a side than it has insulators they share it, fanned a few centimetres apart along the arm so they do not overlap.
  Wires to machines and wall-mounted poles use the same two rules; cables stay bare. The arm cannot turn (a pole stores no
  facing), so a line that leaves at right angles to the arm still picks its nearest end; as a second step, the arm's yaw may be
  derived at draw time from the pole's wired neighbours (presentation reading the wires, never the core). *Where:*
  `Pole::hang_points` and one chooser used by `Factory::for_each_wire`, `preview_wire` and the selection highlight; `wire_props.rs`
  and `pole::span` (hover charging) read the old centre point and stay as they are or move to the nearest hang point.
  *Done when:* a headless test shows both ends of every wire in a built grid coincide with a hang point of their pole, two
  towers' parallel wires never cross, and the state hash is unmoved; seen in the browser pane for a Mk1 pole, a Mk4 pole and a
  tower, each with four or more wires.

## Light and colour (user, 2026-11-16; unordered, one session each; in `docs/DEV_PLAN.md` section 4 as one line)

What exists: a lamp block (`LAMP`, unpowered) and the panel light, torches; one white-ish light channel (`light.rs`: a nibble
per cell, `SOURCES` rows of strength and loss per block, never above `MARGIN` 32 blocks because the mesher reads only the 3×3×3
chunk neighbours); coloured halos per block (`halos.rs` `look`, presentation); plain `GLASS`. No dyes or pigments exist yet.

- **GLASS: stained glass and coloured lamps.** One palette table (16 hues) feeds both. *Ids (audit 2026-11-16):* only 37
  block ids are free until ROOM widens `BlockId` to `u16` (decided 2026-11-16, `docs/ROADMAP_ORDER.md`); coloured lamps still
  take the LED's per-lamp colour record, not 16 ids (one record beats 16 blocks per brightness); stained glass keeps its 16. Stained glass lets light and laser beams
  through like glass (`light.rs` class, `laser.rs`). A coloured lamp lights as a lamp does, its halo and face in its hue.
  *Method:* block ids per hue, appended in one run, textures made by one `tools/art` script that tints a base layer (no
  hand-drawn set); colour then needs no new state, saves or mesher input. *Limit, said plainly:* block light is one channel, so
  a coloured lamp tints its halo and itself, not the walls it lights; true coloured light would carry a hue with the flood (a
  second byte per cell in the mesher's field) and is a later step to measure, not part of this one. *Pigments (user, 2026-11-16):* never
  found loose; made from crushed ores, chemicals, limestone and coal, by a tech in the later chemical research (suggest after
  Acids and Lubricants, four packs), as chemical-plant recipes (`Unlock::MachineRecipe` rows, as plastic and acid are). A few
  bases, the rest mixed, with the mapping the coloured concretes already use (`recipes/machine.rs`, assembler: 4 concrete + 1 of
  the base): red from iron ore, green from copper ore, black from coal, white from quicklime, yellow from sulfur, all crushed
  where the crusher takes them; blue is the one new step, copper and acid (copper sulfate, as in life). Every other hue is a
  chemical-plant mix of two (red + yellow: orange, red + blue: purple, blue + green: cyan, anything + white: lighter, + black:
  darker), so 16 hues cost six bases and about a dozen mixing rows, all data.
  *Done when:* each hue is placed and seen with a lamp behind it, a test shows light and a laser pass stained glass, a
  headless test checks ids and recipes, and the halo colours match.
- **LED: brighter LED lamps with colour control.** *Evidence:* a lamp is already full strength (15) within 17 blocks and reaches
  31, and light must be 0 by 32 blocks (`MARGIN`) or chunk borders seam. *Measured 2026-11-16* (throwaway test, sealed tunnel):
  a row (47, 1) stays 15 to 32 blocks, then cells at distance 33 read 0 or 14 by chunk position: rejected. A row **(64, 2)** in
  the short field is seamless and full-bright to 24 blocks (twice the lamp's area), 0 at 32: use it; measure the remesh cost. Light is derived from blocks only (never core state), so the lamp's **brightness steps are block ids** (three rows),
  and its **colour is a per-lamp record** (cell to hue from the palette above, saved, set by an action from the lamp's own
  panel, as miners have `panel: true`) read by the halo and the lamp's drawn face. *Decided (user, 2026-11-16):* LEDs are
  **unpowered** like the lamp (powering them would need the light to follow the grid: not planned) and come from **a later
  tech**, not Electronics; Claude picks which when LED is planned. *Done when:* a lamp's colour and
  brightness change from its panel and survive a save and a co-op client, the cost is measured against the plan's section 2
  and reported, and a standing scene still remeshes nothing.
- **STREET: tall street lamps.** A 1×1×4 post (a `Footprint` like the tower's: the light-giving head is the top cell, the cells
  below are machine parts; breaking any cell removes the whole), with a bent arm and a lantern drawn in the top cell, a larger
  halo at that height, and a warm light that lights the ground from above (taller than a lamp on a pole, so the pool of light
  is wider where you walk). Uses the lamp's `SOURCES` row; takes the LED row once that exists. *Done when:* a headless test
  places it, breaks it from any cell and round-trips a save, the head lights the ground four blocks below it, and the
  model is seen in the browser pane by day and by night.

## Colour-blind palette and more music (user, 2026-11-16; one session each; ids CBPAL and MUSIC)

- **CBPAL: a colour-blind palette for tier colours, as a setting.** Mk1–Mk5 are told apart by colour (`Family::colour`, the
  kit cases' baked layers `tex::kit_paint`, panels, research and craft tiles); pips and "Mk" text already say the tier too. A
  Settings choice (`ui/settings-tabs.ts`) swaps in a second palette whose five colours stay apart under protan, deuter and
  tritan, checked as `web/test/controls.test.mjs` checks the control categories (CIEDE2000). Presentation only, the state hash
  does not move, and the choice is a local setting. Kit paint is baked into atlas layers, so decide when planning between
  tinting at draw time and alternate layers (an art request line either way). *Done when:* the test passes for both palettes,
  the setting survives a reload, and a kit, a panel and a research tile are seen in each palette in the browser pane.
- **MUSIC: a sea piece, an industrial piece and a space piece.** The sea piece is a place (shore and open water); the other two
  follow the **era** of the player's research, a new context for the music director beside place (`web/src/audio/`, one row a
  piece in `music-pieces.ts`; composed in the user's MusicComposer as the pipeline does, small files, 64 kbit/s Opus). Era must
  read research state from the host, never the core. *Done when:* `scripts/check-music.mjs` passes with the three pieces, a
  director test picks each in its context, and the download budget (9.3 MB of music today, plan section 2) is reported.

## Worlds: carrying old saves forward (user, 2026-11-16; analysed, not scheduled; id WORLDS)

*The question:* can an old world with a large factory move into a newly generated world, maybe with the same seed? *How a save
is built:* seed, generator version, the edited chunks (full block data) and the core state; unedited ground is regenerated, and
every record in the core (belts, machines, poles, pipes, trains, towns, sites, deposits) holds world coordinates.
*Measured* on the user's 37.6 h save (generator v7, seed 1337: 200 edited chunks in 136 chunk columns, about 1,000 × 900 blocks;
a throwaway test, removed again): the same seed read under v8, v9 or v10 keeps the ground height in 96.0 % of the columns under
the edited chunks (15 of 136 chunk columns change, worst 20 blocks); under v11 or v12 only 59.6 % keep it, 7.2 % differ by more
than 10 blocks (worst 47) and 82 of 136 chunk columns change (v11's lowland start reshapes the ground near spawn).
*Verdict.* The seed alone is not enough: seed plus version make the ground. Rewriting the version in place keeps every
coordinate and edited chunk valid, but the unedited ground between and around them changes: cliffs at the edit borders (up to 47
blocks), and `Deposits` re-resolves its saved keys through the generator (`deposit_by_key`), so a different layout can strand or
re-roll the pools miners stand on. Moving a factory to another place or seed means translating every position in the core (no
generic mechanism: each type would need its own) onto ground that will not match anyway: not recommended.
*Recommended: upgrade by layer, same coordinates.* Split the single `WorldGen::version` into layers, each recorded in the
header (a save-version bump; old saves read every layer as their one version): **ground** (landform, v8 and v11), **water**
(rivers and ponds, v8), **cover** (trees v9 and v12, ruins v9 and v10), **deposits** (ores and `EXTRAS`: oil and uranium at v6,
gas fields next). The 49 version checks in 12 worldgen files each read the layer they belong to, a mechanical edit. A world's
menu then offers each layer with its measured safety:
- **Deposits: safe.** Underground only, no seams. Conditions: gas fields are placed apart from `ore_at` (own hash and grid; a
  row in `EXTRAS` changes the biome's weight sum, and `ore_at` draws one pick over it, so every far deposit there would re-roll,
  any ore: checked 2026-11-16), every ore deposit of the bench save is pinned identical, and every saved `Deposits` key resolves. This alone
  gives the 37 h world gas fields in ground nobody has edited, which answers the natural-gas open question for old worlds.
- **Cover: cosmetic.** New trees and ruins in unedited chunks; species change at the edit borders.
- **Water: not offered near builds.** A new river course could end at the edge of a base and spill into it.
- **Ground: not offered to worlds with builds,** or only after a seam blend (unbuilt, untested: rewrite a ring of unedited
  chunks round the edits, easing the old edge height into the new height over about 4 chunks, stored as edited chunks, perhaps
  1–2 MB more).
*Decided (user, 2026-11-16):* a deposits-only upgrade is enough, and **far deposits seen once get no map pin**. *Telling players:* a world whose deposits layer is older
shows a one-time notice and a button in its menu, "Upgrade deposits: new ores and gas fields in ground you have not touched; your
base is unchanged", with one more line, "A new world has more changes: terrain, rivers, trees", and a patch-note line when the
layer ships. The upgrade is an action (core state), applied once and saved; co-op: the host decides.
*Done when (if built):* a pin test per layer for every released version; old fixtures load unchanged; a bot test on the bench
save upgrades the deposits layer only and the factory keeps its state hash while gas fields appear in unedited chunks; the plan's
open question "should old worlds opt into newer generation" is closed in `docs/DECISIONS.md`.

