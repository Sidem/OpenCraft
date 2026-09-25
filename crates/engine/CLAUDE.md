# Engine conventions (crates/engine)

Rust → wasm. Owns all game state and every hot loop. Module map: `docs/CODEMAP.md`.

## Layout

- One feature, one module or folder. A module with submodules is a folder with `mod.rs`.
- Tests go in `src/<module>/tests.rs` via `#[cfg(test)] mod tests;` (`src/tests.rs` for Game-level
  scenario tests). Never inline. Test-only helpers on a type are `#[cfg(test)]` methods.
- `Game` (lib.rs) is a thin facade over the core `Sim` (`sim.rs`), the authority (all bodies, loose
  items: `authority.rs`) and the local player's hands and view. `api/*.rs` methods only forward and act
  for the local player (`self.body()`, `self.inventory()`). Gameplay needing several `Game` fields lives
  in a child module (`interaction.rs`, `authority.rs`, `events.rs`) with `pub(crate)` methods.
- `Game` never mutates `Sim` state: it reads it and queues `self.act(Action::…)` (`act_as` for another
  player), applied at the next tick. Only `World`'s render cache (streaming, meshes) is touched
  directly. `Sim.players` and `Game.bodies` are both indexed by `PlayerId`.
- Header first (`//!`: owns, invariants, how to extend), then public items, then private helpers.
- Balance numbers are named constants at the top of the module that uses them.
- Formatting: `rustfmt.toml` (width 120). `npm run check` runs fmt, clippy `-D warnings` and tests.

## Testing

- `npm run check` before every commit; `cargo test --workspace -q` alone for a quick loop. Filter with
  `cargo test -q <name>`.
- Prefer headless scenario tests over the browser. Patterns to copy:
  - `src/tests.rs`: `run_until_ready(&mut g)` streams the world in; `find_outcrop_block` finds ore;
    `build_mine` places a miner, belt and box directly through `factory`. Actions: `g.act(…)` or the
    API method, then `g.run_ticks(1)`. More players: `g.add_player()`, `g.act_as(id, …)`, `g.bodies`.
  - `action/tests.rs`: a bare `Sim` with `apply` / `queue` + `step`; works without loaded chunks.
  - `factory/tests.rs`: `run(&mut f, seconds, check)` steps a bare `Factory`; `stocked_box` fills a box.
- Worldgen is seeded; tests use fixed seeds (2024, 1337, 7). Changing generation can move the features
  such tests look for.

## Wasm size (check `npx vite build` when it could grow)

- No float formatting (`{:.1}`, `to_string()` on floats): about 25 KB. Round to integers or format in TS.
- No new instantiations of std sorts (5–9 KB each): use `math::sort_small_by_key` (insertion sort).
- No new crates without a reason worth their size. Hand-written serialisation over serde.

## Determinism (DEV_PLAN section 3.4)

- Core state lives in `Sim` (`sim.rs`): world edits, factory, deposits, inventories, tick, rng. It
  changes only in `Sim::step`: the tick's queued actions (`action.rs`), then the factory by exactly
  `TICK`. Never depend on frame time, loaded chunks, the camera, `Sounds`, UI state or hash-map order;
  report outward with a `SimEvent`. Per-frame work in `Game::update` is presentation only.
- New core state must be written by its type's `write_state` (`bytes.rs`), or `Sim::state_hash` (and
  saves) miss it. `sim/tests.rs` (same actions, same hash; loaded chunks don't matter) and
  `results_do_not_depend_on_frame_rate` (`src/tests.rs`) compare hashes.
- Core reads and edits use `World::block_anywhere_or_generate` / `set_block_anywhere`. `get_block` /
  `set_block` only see loaded chunks (the render cache), and `set_block` silently fails elsewhere.
- Queries must not create core state: `target_detail` uses `deposits::owner_of` and
  `DepositState::survey`, never `Deposits::lookup` (which starts tracking).

## Gotchas

- A `pub` field whose type is private to the parent module triggers the `private_interfaces` lint; make
  the type `pub(crate)` (see `factory::Link`).
- `Factory::relink` runs lazily when `dirty` is set, so read links only after an `update` (the belt
  readout skips link text while dirty).
- Block ids double as item ids. `BLOCK_COUNT` sizes the lookup tables; append ids, never renumber.
