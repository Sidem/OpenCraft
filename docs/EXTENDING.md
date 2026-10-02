# Extending OpenCraft

Module ownership is mapped in CODEMAP.md. These recipes describe how to add content, APIs and UI.

## How to add…

**A block.** `block.rs`: append an id constant (never renumber), bump `BLOCK_COUNT`, add a `DEFS` row
(`cube`, `ore` or `machine` helper). New texture: a `tex` constant plus its arm in `textures::pixel`.
Placeable blocks work at once; worldgen use goes in `worldgen/`.

**An item or recipe.** A block is already an item. Any other item: an id constant (from 256, append only)
and a row in `item.rs` `EXTRA`, with a texture layer in `block::tex` and its pattern in `textures::pixel`
if it needs a new look; the HUD icon and the loose and belt models follow from the row. A recipe is a row
in `recipes/mod.rs` (its `group` picks the build-menu section); the build menu shows every row. To lock it
behind research, list `Unlock::Recipe(item)` in a tech's `unlocks` (`research/techs.rs`). A machine recipe is a
row in `recipes/machine.rs` with its category (lock: `Unlock::MachineRecipe(index)`). The content lint
(`recipes/tests.rs`, `research/tests.rs`) says what is missing: a source, a use, a machine.

**A tech.** A row appended to `TECHS` in `research/techs.rs` (saves store progress by index): name, blurb,
prerequisites by index, packs per unit, units, seconds, what it unlocks. The research screen and the build
menu follow. A new science pack: an item, a hand recipe and an entry in `PACKS` (labs get a slot for it).

**A processor** (anything that turns inputs into outputs: assembler, furnace, crusher): a `ProcessSpec` row
in `factory/process/specs.rs` (categories, energy, pick, buffers, tiers, footprint and ports, model parts), a `MACHINES` row of
`Kind::Process` for its block, the block (`machine(...)` in `block/mod.rs`), a hand recipe, and its family
in `factory/tiers.rs` if it has tiers. No new code unless it needs new behaviour (then in `process/`: the
boiler, turbine and silo are `Energy::Boiler`, `Energy::Turbine` and `Pick::Store` rows with their code in
`process/steam.rs`; the crusher is a plain 1×1 row with `Category::Crushing`).

**A machine** with behaviour of its own. (1) `factory/<machine>.rs`: the struct (a `Buffer` if it holds
items), `new`, `step`, `impl Machine` (bytes, contents, readout, model), its tuning constants. (2)
`factory/mod.rs`: a `Kind` and a `Slot` variant, a `MACHINES` row (block, kind, slots), a `Vec` field; then
follow the compiler through the `match`es on `Kind` and `Slot` (`place`, `remove`, `take_contents`,
`describe`), and add its list to `write_state` / `read_state` (`state.rs`), `update`, `write_instances`, and
its outputs to `links.rs`. Power: a demand in `Power::balance`, its pole in `Power::rebuild`, a `speed`
argument to `step`, its arm in `wiring.rs` (`powered_cells`, `power_slots`). Fed by belts: an arm in `Slot::is_sink` and a field in `Sinks`. A panel: `panel: true`
in its row, a `panel()` method and its arms in `panel.rs` (the host needs nothing). (3) Its block
(`machine(...)` for a model, `cube(...)` if meshed) and a hand recipe. Tests in `factory/tests.rs` (`run`,
`stocked_box`; test-only accessors live there). A second block with the same behaviour (splitter/filter) is
an extra `MACHINES` row after the Kind-ordered ones, not a new kind.
**A wasm API method.** Put it in the `api/*.rs` file for its area and keep it a thin forwarder; logic goes
in a module. Run `npm run build:wasm`, then call it from TS (types come from `web/src/wasm/engine.d.ts`).
Never mirror an engine constant in TS; expose a getter as `api/content.rs` does.

**A UI panel.** `web/src/ui/<panel>.ts` with a header comment, plus `<panel>.css` imported at the top of
that file. Build DOM with `h()` / `button()` from `ui/dom.ts`; reuse `.secondary-btn` and `.close-btn`.
Construct it in `main.ts`. To open it with a key: an `Action` in `input.ts` and a branch in main.ts's
action loop.

**Core state, a way to change it, or a reaction to it.** State: a field on `Sim` (`sim.rs`) or the type
that owns it (per-player state in `PlayerCore`), plus its bytes in that type's `write_state` and
`read_state`, which the state hash and saves use (a save test fails if the two disagree). A change: an
`Action` variant, its arm in `Sim::apply_to_player` (`action.rs`) and its bytes in `action/codec.rs`; `Game` queues it with `act` (local
player) or `act_as`. A reaction (sound, toast, item spawn): a `SimEvent` variant, pushed by the core, and
its arm in `Game::handle_sim_events` (`events.rs`). Per-player state the authority keeps (body-related)
goes in `authority.rs`, indexed by `PlayerId` like `Sim.players`.

**Something saved.** Core state: as above. Bodies and loose items: `Player` / `Items` `write_state` and
`read_state`, called from `save.rs`. Any change to the bytes bumps `SAVE_VERSION`; keep older saves
loading when a read can follow the old layout cheaply (branch on `ByteReader::version`, add a fixture
test), else raise `OLDEST_VERSION`. A change to world generation goes into a new generator version (`worldgen/mod.rs` header); released versions never change. The browser
side (`web/src/save/`) only stores bytes and never needs to change.

**A sound material.** Engine: a constant in `block::sound` and point blocks' `DEFS` rows at it. Web: append
the name to `MATERIALS` (same order as the engine), add a `MATERIAL_LABELS` entry and a
`DEFAULT_DESIGN.materials` entry in `audio/settings.ts`. The designer gets a tab automatically.

**A sound event kind.** A constant in `sound.rs`, its entry in `EVENT_ACTIONS` (`audio/sound.ts`) and an
action in `audio/settings.ts` (`ACTIONS`, `ACTION_INFO`, `DEFAULT_DESIGN.actions`).
