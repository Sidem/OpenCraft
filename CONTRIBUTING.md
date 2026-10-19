# Contributing to OpenCraft

Thanks for helping. OpenCraft is written mostly by AI coding agents, so the rules below exist to keep the code
small and cheap to read. They apply to people and agents alike.

## Before you start

- Read [CLAUDE.md](CLAUDE.md), then `docs/DEV_PLAN.md` and `docs/CODEMAP.md`. The plan says what is
  being built now and why; area conventions are in `crates/engine/CLAUDE.md` and `web/src/CLAUDE.md`.
- For anything bigger than a fix, open an issue first so it can be fitted into the plan.
- Setup, commands and Windows notes: [README](README.md#quick-start) and `docs/WORKFLOW.md`.

## Rules that matter most

- **Keep changes local.** One feature is one module or folder, plus at most one registration line. Content (blocks,
  items, recipes, machines) is data.
- **Size budgets.** Source files warn at 400 lines and fail at 600 (tests excluded). Split before you add.
- **Tests** live in `src/<module>/tests.rs`, never inline. Prefer headless Rust scenario tests over the browser.
- **Determinism.** The core (world edits, factory, deposits, inventories) changes only through queued actions at a
  fixed tick. A change to saved state bumps the save version (see `crates/engine/CLAUDE.md`).
- **Docs are maps.** Update `docs/CODEMAP.md` in the same commit as any structural change.

## Sending a change

1. Branch from `main` and keep the change to one scoped step.
2. Run `npm run build:wasm` once, then `npm run check` (format, clippy, tests, types, size budgets). CI runs the same
   checks on every pull request.
3. Open a pull request. **Merging to `main` deploys the live site**, so only the maintainer merges.

By contributing you agree that your work is licensed under the [MIT license](LICENSE).
