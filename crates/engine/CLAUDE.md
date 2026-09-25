# Engine conventions (crates/engine)

Rust → wasm. Owns all game state and every hot loop. Module map: `docs/CODEMAP.md`.

## Layout

- One feature, one module or folder. A module with submodules is a folder with `mod.rs`.
- Tests go in `src/<module>/tests.rs` via `#[cfg(test)] mod tests;` (`src/tests.rs` for Game-level
  scenario tests). Never inline. Test-only helpers on a type are `#[cfg(test)]` methods.
- `Game` (lib.rs) is a thin facade. JS-facing methods live in `api/*.rs` and only forward. Gameplay that
  needs several `Game` fields lives in a child module (`interaction.rs`): child modules can read `Game`'s
  private fields, and cross-module methods are `pub(crate)`.
- Header first (`//!`: owns, invariants, how to extend), then public items, then private helpers.
- Balance numbers are named constants at the top of the module that uses them.
- Formatting: `rustfmt.toml` (width 120). `npm run check` runs fmt, clippy `-D warnings` and tests.

## Testing

- `npm run check` before every commit; `cargo test --workspace -q` alone for a quick loop. Filter with
  `cargo test -q <name>`.
- Prefer headless scenario tests over the browser. Patterns to copy:
  - `src/tests.rs`: `run_until_ready(&mut g)` streams the world in; `find_outcrop_block` finds ore;
    `build_mine` places a miner, belt and box directly through `factory`.
  - `factory/tests.rs`: `run(&mut f, seconds, check)` steps a bare `Factory`; `stocked_box` fills a box.
- Worldgen is seeded; tests use fixed seeds (2024, 1337, 7). Changing generation can move the features
  such tests look for.

## Wasm size (check `npx vite build` when it could grow)

- No float formatting (`{:.1}`, `to_string()` on floats): about 25 KB. Round to integers or format in TS.
- No new instantiations of std sorts on short lists (5–9 KB each): use an insertion sort
  (`worldgen/ore.rs::sort_by_ownership`).
- No new crates without a reason worth their size. Hand-written serialisation over serde.

## Determinism (DEV_PLAN section 3.4)

- Core state (world edits, factory, deposits, inventories) advances only in `Game::run_tick`, by exactly
  `TICK`, and through actions. Never depend on frame time, loaded chunks, the camera, UI state or hash-map
  order. Per-frame work in `Game::update` is presentation only. `results_do_not_depend_on_frame_rate`
  (`src/tests.rs`) guards this; extend its snapshot when you add core state.
- Core reads and edits use `World::block_anywhere` / `set_block_anywhere`. `get_block` / `set_block` only
  see loaded chunks, and `set_block` silently fails elsewhere.
- Queries must not create state. Known violation until step 1.2: `target_detail` calls
  `Deposits::lookup`, which inserts deposit state.

## Gotchas

- A `pub` field whose type is private to the parent module triggers the `private_interfaces` lint; make
  the type `pub(crate)` (see `factory::Link`).
- `Factory::relink` runs lazily when `dirty` is set, so read links only after an `update` (the belt
  readout skips link text while dirty).
- Block ids double as item ids. `BLOCK_COUNT` sizes the lookup tables; append ids, never renumber.
