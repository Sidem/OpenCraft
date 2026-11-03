# OpenCraft on itch.io

This folder exists only on the `itch` branch. The deploy workflow is `.github/workflows/itch.yml` (on `main`); how
the two branches fit together is in `docs/WORKFLOW.md` section 4.

## One-time setup (the user)

1. **Project page:** itch.io → Upload new project. Kind of project: **HTML**. Save as a draft first; the page must
   exist before the first push.
2. **API key:** itch.io → Settings → API keys → Generate. Follow `itch/butler-key.example.txt`: it sets the secret
   `BUTLER_API_KEY` and the variable `ITCH_TARGET` (`hexlife/opencraft`, the page at hexlife.itch.io/opencraft).
3. **Worker:** redeploy the signalling Worker so it accepts itch (`cd signal; npx wrangler deploy`).
4. **First push:** push `main` and `itch` to GitHub. The workflow uploads the `html5` channel.
5. **Page settings** (Edit game): the `html5` upload is ticked "This file will be played in the browser"; viewport
   1280 x 720 (or whatever fits the page); enable **Fullscreen button**; enable "Automatically start on page load"
   only if you want it; leave **SharedArrayBuffer support off** (the game doesn't use it). Mobile friendly: off.
   Then set Visibility to Public when the page is ready.

## Manual upload (no CI)

```
npm run build:wasm
npm run build
node itch/pack.mjs          # checks dist/ against itch's rules, writes itch/build/opencraft-html5.zip
```

Upload the zip on the Edit game page, or `butler push dist <user>/<game>:html5`.

## Things that differ from the GitHub Pages site

- Saves live in the browser's IndexedDB for itch's game host, so they do not carry over from the Pages site. Use the
  world export/import to move a save.
- Co-op: the Worker allows `https://*.itch.zone`. Players join by pasting the room code (the invite link points at
  itch's iframe host, which may not load on its own).
- itch shows the game in an iframe. The bundle boots and renders framed on a sub-path and another origin (tested
  locally), but pointer lock and fullscreen inside itch's own iframe are untested: check them on the live page.

## Page copy (draft)

**Tagline:** A voxel sandbox where you build the factory that builds the world.

**Short description:** Mine, craft and automate in a browser-based voxel world: dig deposits, lay conveyors, build
machines and grow from a hand-built camp into a whole production chain. Play solo or co-op with friends, no install.

**Tags:** voxel, sandbox, factory, automation, crafting, building, co-op, singleplayer, procedural, wasm

**Needed images:** cover 630 x 500 (itch recommends 315 x 250 minimum), 3 to 5 screenshots. The local `trailer/`
rig can produce a GIF or video.
