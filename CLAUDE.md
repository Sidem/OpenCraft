# OpenCraft: notes for coding agents

Browser voxel sandbox × factory builder: a Rust → wasm engine (`crates/engine`) with a thin TypeScript/WebGL2
host (`web/src`).

**Start every session by reading `docs/DEV_PLAN.md` (sections 0–4), then `docs/CODEMAP.md` (once it exists).**
The plan is the plan of record: direction, the user's settled decisions, what exists, and the exact next steps
with acceptance criteria. Keep it current: tick finished steps, update its status line, add to its change log.

## Top priority: keep the codebase small and cheap to work on

This project is written **entirely by AI coding agents**. Every session starts cold, so every line you must read
costs tokens and time, and a sprawling codebase makes every future feature slower and more expensive. **This
matters as much as any feature.** The full rules are in plan section 3.1 and are mandatory:

- **Changes stay local.** One feature means one module or folder: new files plus at most one registration line.
  No god objects. Content (blocks, items, recipes, machines) is data.
- **Size budgets.** Source files: warn at 400 lines, fail at 600 (tests excluded). If a file is over budget,
  split it before adding to it.
- **Tests live in `src/<module>/tests.rs`**, never inline.
- **Every module starts with a short header:** what it owns, invariants, how to extend. Public items first.
- **Boring, explicit code.** No speculative abstractions; add a registry when the second instance arrives.
- **Docs are maps.** Update `docs/CODEMAP.md` in the same commit as any structural change. The plan details
  only the current milestone. Future milestones live in `docs/ROADMAP.md` (read only when planning).
  Operational how-tos live in `docs/WORKFLOW.md` (read the section you need).
- **Fast, quiet feedback.** Use `npm run check` before commits. Prefer headless Rust scenario tests over the
  browser pane. Keep tool output short.
- **One scoped step per session.** Finish green and committed.

## Essentials (details in `docs/WORKFLOW.md`)

- **Pushing to `main` deploys the live site** (GitHub Pages). Ask the user before any push. Commit only when
  asked.
- Windows machine: use PowerShell and the file tools (the Bash tool fails here). For commit messages, write a
  file and use `git commit -F <file>`.
- Checks: `npm run check` (after plan step 1.0), or `cargo test --workspace --release`, `npm run typecheck`,
  `npm run build`. Browser: `preview_start` name `opencraft`, then drive `window.opencraft.game` directly.
- Performance first: state and hot loops in Rust, zero-copy data to TS, dependencies justified, wasm size
  watched.
- The deterministic core (world edits, factory, deposits, inventories) stays deterministic: fixed ticks,
  changes only through actions, no dependence on loaded chunks, camera or frame time (plan section 3.4).
