# Sources of the sound effects

Cut, faded, pitched and encoded (64 kbit/s mono Opus) outside the game; how each sound is named and levelled is in
`../effects.ts`. Every material has two sources: a footsteps recording for `step` and `land`, and an impact recording
for `dig`, `place` and `break`. Chosen by label: the source's own name says both the material and the action. Footsteps
are whole steps that die away, never pieces cut mid-sound.

## Recordings (CC0 1.0, public domain: no attribution required, given anyway)

| Material | Footsteps (`step`, `land`) | Impact (`dig`, `place`, `break`) |
|---|---|---|
| Stone | Kenney, `footstep_concrete` | Kenney, `impactMining` |
| Dirt | monte32, "Footsteps_6_Dirt_shoe" ([353799](https://freesound.org/people/monte32/sounds/353799/)) | dr19, "Shovel_dirt" ([353907](https://freesound.org/people/dr19/sounds/353907/)) |
| Grass | Kenney, `footstep_grass` | dr19, "Shovel_dirt" (dirt's impact) |
| Sand | BranndyBottle, "sandwalking_step1" and "sandwalking_step2" ([464699](https://freesound.org/people/BranndyBottle/sounds/464699/), [464695](https://freesound.org/people/BranndyBottle/sounds/464695/)) | f3bbbo, "Digging in wet course sand" (1) and (2) ([651292](https://freesound.org/people/f3bbbo/sounds/651292/), [651293](https://freesound.org/people/f3bbbo/sounds/651293/)) |
| Wood | Kenney, `footstep_wood` | Kenney, `impactWood_medium` |
| Leaves | 24srtx, "footsteps_leaves" ([62974](https://freesound.org/people/24srtx/sounds/62974/)) | kyles, "leaves dry straw brush put down slam" ([454064](https://freesound.org/people/kyles/sounds/454064/)) |
| Metal | GiocoSound, "Footstep_Metal_1", "_2" and "_3" ([421134](https://freesound.org/people/GiocoSound/sounds/421134/), [421133](https://freesound.org/people/GiocoSound/sounds/421133/), [421132](https://freesound.org/people/GiocoSound/sounds/421132/)) | Kenney, `impactMetal_medium` |

Kenney: the **Impact Sounds** pack by Kenney (www.kenney.nl), kenney.nl/assets/impact-sounds. The others are from
freesound.org, linked by sound id.

## Rendered sounds

- `drop`, `scan`, `launch`, `podup`: rendered from the sound bank of **earshot** (github.com/Leonxlnx/earshot, MIT
  licence, copyright 2026 Leon Lin): `swish/zing`, `pro_notify/ding_2045`, `impact/blast` and `whoosh_big/jet`.
- `pickup`, `click`, `splash`, `powerdown`, `powerup`, `poddown`, `denied`: the game's own former synthesised sounds,
  rendered to files (power up and down without their crackles, one variant each).
