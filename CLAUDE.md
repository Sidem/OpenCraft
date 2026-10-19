# OpenCraft: notes for coding agents

Browser voxel sandbox × factory builder: a Rust → wasm engine (`crates/engine`) with a thin TypeScript/WebGL2
host (`web/src`).

**Start every session by reading `docs/DEV_PLAN.md`, then `docs/CODEMAP.md`** (together under 48 KB, checked).
Area conventions live in `crates/engine/CLAUDE.md` and `web/src/CLAUDE.md`. The plan is the plan of record: status,
the decisions that bind new work, the rules, and the next steps with acceptance criteria. Keep it current: tick
finished steps, update its status line, add to its change log. Read the other docs only when a task needs them:
`docs/DECISIONS.md` (every settled choice), `docs/ROADMAP.md` (future work), `docs/TECH_TREE.md` and `TECH_ERAS.md`
(tech tree), `docs/EXTENDING.md` (how to add content), `docs/WORKFLOW.md` (commands, browser, deploying),
`docs/ART_HANDOVER.md` (who owns which art files), `docs/history/` (finished work).

## Top priority: keep the codebase small and cheap to work on

This project is written **entirely by AI coding agents**. Every session starts cold, so every line you must read
costs tokens and time. **This matters as much as any feature.** The full rules are plan section 3.1, mandatory:

- **Changes stay local.** One feature means one module or folder: new files plus at most one registration line.
  No god objects. Content (blocks, items, recipes, machines) is data.
- **Size budgets.** Source files: warn at 400 lines, fail at 600 (tests excluded); docs are budgeted in bytes. If a
  file is over budget, split it before adding to it.
- **Tests live in `src/<module>/tests.rs`**, never inline.
- **Every module starts with a short header:** what it owns, invariants, how to extend. Public items first.
- **Boring, explicit code.** No speculative abstractions; add a registry when the second instance arrives.
- **Docs are maps, not history.** Update `docs/CODEMAP.md` in the same commit as any structural change; never copy a
  number from code into a doc, point to the constant.
- **Fast, quiet feedback.** Use `npm run check` before commits. Prefer headless Rust scenario tests over the
  browser pane. Keep tool output short.
- **One scoped step per session.** Finish green and committed.

## Essentials (details in `docs/WORKFLOW.md`)

- **Pushing to `main` deploys the live site** (GitHub Pages). Ask the user before any push. Commit only when
  asked.
- Windows machine: use PowerShell and the file tools (the Bash tool fails here). For commit messages, write a
  file and use `git commit -F <file>`.
- Checks: `npm run check` (fmt, clippy, tests, typecheck, budgets, code map; needs `npm run build:wasm` once).
  Browser: `preview_start` name `opencraft`, then drive `window.opencraft.game` directly.
- Performance first: state and hot loops in Rust, zero-copy data to TS, dependencies justified, wasm size
  watched.
- The deterministic core (world edits, factory, deposits, inventories) stays deterministic: fixed ticks,
  changes only through actions, no dependence on loaded chunks, camera or frame time (plan section 3.4).
