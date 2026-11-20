use super::*;
use crate::block::{STONE, WATER};
use crate::chunk::Chunk;
use crate::mesher::shaped::{KIND_PLANT, SHAPED_OFFSET, SHAPED_SCALE};
use crate::mesher::{neighbor_index, MeshOutput, PAD_VOLUME};
use std::collections::BTreeMap;

static DAYLIGHT: [u8; PAD_VOLUME] = [15; PAD_VOLUME];
static DARK: [u8; PAD_VOLUME] = [0; PAD_VOLUME];

/// A 32 x 32 floor of `ground` at y = 3, meshed with ground cover of `biome` (or none) under `light`.
fn floor(ground: BlockId, biome: Biome, light: &[u8], x0: i32) -> MeshOutput {
    let (air, mut c) = (Chunk::uniform(AIR), Chunk::uniform(AIR));
    for z in 0..32 {
        for x in 0..32 {
            c.set(x, 3, z, ground);
        }
    }
    let mut refs = [&air; 27];
    refs[neighbor_index(0, 0, 0)] = &c;
    let biomes = vec![biome; 1024];
    Mesher::new().mesh(&refs, light, Some(&Cover { x0, z0: 0, biomes: &biomes, cleared: &[] }))
}

/// Decoded position (chunk-local blocks) of one shaped vertex.
fn pos_of(w: &[u32]) -> [f32; 3] {
    let f = |v: u32| v as f32 / SHAPED_SCALE - SHAPED_OFFSET as f32;
    [f(w[0] & 4095), f((w[0] >> 12) & 4095), f(w[1] & 4095)]
}

type PlantCells<'a> = BTreeMap<(i32, i32, i32), (u32, Vec<&'a [u32]>)>;

/// Every `KIND_PLANT` quad in `m`, grouped by its cell `(x, y, z)` into `(layer, quads)`.
fn plants(m: &MeshOutput) -> PlantCells<'_> {
    let start = m.shaped_opaque as usize * 12;
    let cutout = &m.shaped[start..start + m.shaped_cutout as usize * 12];
    let mut by_cell: PlantCells<'_> = BTreeMap::new();
    for q in cutout.chunks_exact(12) {
        if (q[0] >> 24) & 7 != KIND_PLANT {
            continue;
        }
        // Quad centroid's cell (a flower or fern frond can overhang slightly, its centroid stays in its own cell).
        let mut sum = [0.0f32; 3];
        for i in 0..4 {
            let p = pos_of(&q[i * 3..i * 3 + 3]);
            sum[0] += p[0];
            sum[1] += p[1];
            sum[2] += p[2];
        }
        let cell = ((sum[0] * 0.25).floor() as i32, (sum[1] * 0.25).floor() as i32, (sum[2] * 0.25).floor() as i32);
        let layer = q[2] >> 16;
        let entry = by_cell.entry(cell).or_insert_with(|| (layer, Vec::new()));
        assert_eq!(entry.0, layer, "all quads of a cell share one plant layer");
        entry.1.push(q);
    }
    by_cell
}

/// The texture layers of the plants in a mesh (one per plant cell).
fn layers(m: &MeshOutput) -> Vec<u32> {
    plants(m).into_values().map(|(layer, _)| layer).collect()
}

fn count(m: &MeshOutput, layer: u16) -> usize {
    layers(m).iter().filter(|&&l| l == u32::from(layer)).count()
}

#[test]
fn every_biome_has_its_own_cover() {
    let plains = floor(GRASS, Biome::Plains, &DAYLIGHT, 0);
    let low = floor(GRASS, Biome::Lowlands, &DAYLIGHT, 0);
    let desert = floor(SAND, Biome::Desert, &DAYLIGHT, 0);
    let basalt = floor(BASALT, Biome::BasaltFields, &DAYLIGHT, 0);
    let hills = floor(GRANITE, Biome::Highlands, &DAYLIGHT, 0);
    assert!(count(&plains, tex::GRASS_TUFT) > 80 && count(&plains, tex::FERN) == 0, "plains: tufts");
    assert!(count(&low, tex::FERN) > 40, "lowlands: ferns");
    assert!(count(&desert, tex::DRY_BRUSH) > 20 && count(&desert, tex::GRASS_TUFT) == 0, "desert: dry brush");
    assert!(count(&basalt, tex::LICHEN) > 30, "basalt fields: lichen");
    assert!(count(&hills, tex::LICHEN) > 20, "highlands: lichen on granite");
    assert_eq!(layers(&desert).len(), count(&desert, tex::DRY_BRUSH), "nothing else grows on sand in a desert");
}

#[test]
fn flowers_grow_in_meadow_patches() {
    let m = floor(GRASS, Biome::Plains, &DAYLIGHT, 0);
    let flowers: usize = [tex::DAISY, tex::POPPY, tex::BLUEBELL].iter().map(|&l| count(&m, l)).sum();
    let tufts = count(&m, tex::GRASS_TUFT);
    assert!(flowers > 0 && flowers < tufts, "{flowers} flowers among {tufts} tufts");
}

#[test]
fn cover_is_a_pure_function_of_the_ground_and_position() {
    let (a, b) = (floor(GRASS, Biome::Plains, &DAYLIGHT, 64), floor(GRASS, Biome::Plains, &DAYLIGHT, 64));
    assert_eq!(a.shaped, b.shaped);
    assert_ne!(layers(&a), layers(&floor(GRASS, Biome::Plains, &DAYLIGHT, 96)), "another place, another meadow");
}

#[test]
fn no_cover_without_sky_without_biomes_or_off_natural_ground() {
    assert!(layers(&floor(GRASS, Biome::Plains, &DARK, 0)).is_empty(), "indoors and in caves stays bare");
    assert!(layers(&floor(STONE, Biome::Plains, &DAYLIGHT, 0)).is_empty(), "plain stone grows nothing");
    let (air, mut c) = (Chunk::uniform(AIR), Chunk::uniform(AIR));
    c.set(4, 3, 4, GRASS);
    let mut refs = [&air; 27];
    refs[neighbor_index(0, 0, 0)] = &c;
    let none = Mesher::new().mesh(&refs, &DAYLIGHT, None);
    assert_eq!(none.shaped_cutout, 0, "versions without biomes get no cover");
    let short = Cover { x0: 0, z0: 0, biomes: &[], cleared: &[] };
    assert_eq!(Mesher::new().mesh(&refs, &DAYLIGHT, Some(&short)).shaped_cutout, 0);
}

#[test]
fn cover_stays_off_cells_that_are_not_empty() {
    let (air, mut c) = (Chunk::uniform(AIR), Chunk::uniform(AIR));
    for z in 0..32 {
        for x in 0..32 {
            c.set(x, 3, z, GRASS);
            c.set(x, 4, z, STONE);
        }
    }
    let mut refs = [&air; 27];
    refs[neighbor_index(0, 0, 0)] = &c;
    let biomes = vec![Biome::Plains; 1024];
    let m = Mesher::new().mesh(&refs, &DAYLIGHT, Some(&Cover { x0: 0, z0: 0, biomes: &biomes, cleared: &[] }));
    assert_eq!(m.shaped_cutout, 0);
}

#[test]
fn reeds_stand_on_the_bank() {
    let (air, mut c) = (Chunk::uniform(AIR), Chunk::uniform(AIR));
    for z in 0..32 {
        for x in 0..32 {
            // Sand land with a lake in the left half, whose water surface lies a block below the bank.
            if x < 16 {
                c.set(x, 2, z, SAND);
                c.set(x, 3, z, WATER);
            } else {
                c.set(x, 3, z, SAND);
            }
        }
    }
    let mut refs = [&air; 27];
    refs[neighbor_index(0, 0, 0)] = &c;
    let biomes = vec![Biome::Plains; 1024];
    let m = Mesher::new().mesh(&refs, &DAYLIGHT, Some(&Cover { x0: 0, z0: 0, biomes: &biomes, cleared: &[] }));
    let reeds = count(&m, tex::REED);
    assert!(reeds > 5 && reeds < 32, "reeds only along the shore line: {reeds}");
}

#[test]
fn indicator_plants_grow_only_over_their_own_stain() {
    use crate::block::{DARK_SAND, DARK_SOIL, GREEN_SAND, GREEN_SOIL, PALE_SOIL, RUSTY_SAND, RUSTY_SOIL};
    let all = [tex::INDICATOR_IRON, tex::INDICATOR_COAL, tex::INDICATOR_COPPER, tex::INDICATOR_PALE];
    let stains = [
        (RUSTY_SOIL, tex::INDICATOR_IRON),
        (DARK_SOIL, tex::INDICATOR_COAL),
        (GREEN_SOIL, tex::INDICATOR_COPPER),
        (PALE_SOIL, tex::INDICATOR_PALE),
        (RUSTY_SAND, tex::INDICATOR_IRON),
        (DARK_SAND, tex::INDICATOR_COAL),
        (GREEN_SAND, tex::INDICATOR_COPPER),
    ];
    for (stain, own) in stains {
        let m = floor(stain, Biome::Plains, &DAYLIGHT, 0);
        let mine = count(&m, own);
        assert!(mine > 300, "stain {stain}: {mine} of 1024 columns carry its plant");
        for other in all.into_iter().filter(|&o| o != own) {
            assert_eq!(count(&m, other), 0, "stain {stain} grows another stain's plant");
        }
    }
    for ground in [GRASS, SAND, DIRT, BASALT] {
        let m = floor(ground, Biome::Plains, &DAYLIGHT, 0);
        assert!(all.iter().all(|&o| count(&m, o) == 0), "unstained ground {ground} grows an indicator");
    }
}

#[test]
fn meadow_plants_stay_compact_and_flowers_face_upward() {
    let plains = floor(GRASS, Biome::Plains, &DAYLIGHT, 0);
    let mut daisies = 0;
    let mut flat_heads = 0;
    for (layer, quads) in plants(&plains).into_values() {
        let max_y =
            quads.iter().flat_map(|q| (0..4).map(|i| pos_of(&q[i * 3..i * 3 + 3])[1] - 4.0)).fold(0.0f32, f32::max);
        assert!(max_y <= 0.50, "layer {layer} reaches {max_y:.2} blocks high and would block the view");
        if layer == u32::from(tex::DAISY) {
            daisies += 1;
            for q in quads {
                let ys: [f32; 4] = std::array::from_fn(|i| pos_of(&q[i * 3..i * 3 + 3])[1]);
                let span_y = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max)
                    - ys.iter().copied().fold(f32::INFINITY, f32::min);
                // Upward-facing blossom heads sit elevated off the ground and have a small vertical span (< 0.12).
                if ys[0] > 4.10 && span_y < 0.12 {
                    flat_heads += 1;
                }
            }
        }
    }
    assert!(
        daisies > 0 && flat_heads >= daisies * 3,
        "{daisies} daisy clusters should emit 3+ upward heads ({flat_heads})"
    );

    // Lichen hugs rock flat instead of standing up as vertical crossed quads.
    let basalt = floor(BASALT, Biome::BasaltFields, &DAYLIGHT, 0);
    for (layer, quads) in plants(&basalt).into_values() {
        if layer == u32::from(tex::LICHEN) {
            let max_y =
                quads.iter().flat_map(|q| (0..4).map(|i| pos_of(&q[i * 3..i * 3 + 3])[1] - 4.0)).fold(0.0f32, f32::max);
            assert!(max_y < 0.08, "lichen should hug rock flat, reached {max_y:.3}");
        }
    }
}
