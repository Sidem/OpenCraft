use super::*;
use crate::block::{STONE, WATER};
use crate::chunk::Chunk;
use crate::mesher::{neighbor_index, MeshOutput, PAD_VOLUME};

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

/// The texture layers of the plants in a mesh (one per plant), skipping the floor's own quads.
fn layers(m: &MeshOutput) -> Vec<u32> {
    let cutout = &m.verts[m.opaque_quads as usize * 4..(m.opaque_quads + m.cutout_quads) as usize * 4];
    cutout.chunks(16).map(|plant| plant[0] >> 23).collect()
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
    assert_eq!(a.verts, b.verts);
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
    assert_eq!(none.cutout_quads, 0, "versions without biomes get no cover");
    let short = Cover { x0: 0, z0: 0, biomes: &[], cleared: &[] };
    assert_eq!(Mesher::new().mesh(&refs, &DAYLIGHT, Some(&short)).cutout_quads, 0);
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
    assert_eq!(m.cutout_quads, 0);
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
