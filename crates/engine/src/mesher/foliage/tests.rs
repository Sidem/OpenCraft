use std::collections::HashMap;

use crate::block::{BlockId, AIR, BIRCH_LEAVES, LEAVES, LOG};
use crate::chunk::Chunk;
use crate::mesher::{neighbor_index, MeshOutput, Mesher, PAD_VOLUME};

static DAYLIGHT: [u8; PAD_VOLUME] = [15; PAD_VOLUME];

/// Chunks -2..=1 on each axis, filled from `block(x, y, z)` in world blocks.
fn world(block: impl Fn(i32, i32, i32) -> BlockId) -> HashMap<[i32; 3], Chunk> {
    let mut out = HashMap::new();
    for cy in -2..=1 {
        for cz in -2..=1 {
            for cx in -2..=1 {
                let mut c = Chunk::uniform(AIR);
                for y in 0..32 {
                    for z in 0..32 {
                        for x in 0..32 {
                            let b = block(cx * 32 + x as i32, cy * 32 + y as i32, cz * 32 + z as i32);
                            if b != AIR {
                                c.set(x, y, z, b);
                            }
                        }
                    }
                }
                out.insert([cx, cy, cz], c);
            }
        }
    }
    out
}

fn mesh(m: &mut Mesher, w: &HashMap<[i32; 3], Chunk>, c: [i32; 3]) -> MeshOutput {
    let air = Chunk::uniform(AIR);
    let mut refs = [&air; 27];
    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if let Some(n) = w.get(&[c[0] + dx, c[1] + dy, c[2] + dz]) {
                    refs[neighbor_index(dx, dy, dz)] = n;
                }
            }
        }
    }
    m.set_origin(c.map(|v| v * 32));
    m.mesh(&refs, &DAYLIGHT, None)
}

/// The cards (kind 0 in the cutout list): per quad its four positions in chunk-local blocks and its normal.
fn cards(out: &MeshOutput) -> Vec<([[f32; 3]; 4], [f32; 3])> {
    let start = out.shaped_opaque as usize * 12;
    out.shaped[start..]
        .chunks(12)
        .filter(|q| q[0] >> 24 & 7 == 0)
        .map(|q| {
            let p = std::array::from_fn(|i| {
                let w = &q[i * 3..i * 3 + 3];
                [w[0] & 4095, w[0] >> 12 & 4095, w[1] & 4095].map(|v| v as f32 / 100.0 - 2.0)
            });
            (p, octahedral(q[2] & 0xFFFF))
        })
        .collect()
}

/// `shaped::octahedral` undone (as `render/shaped.ts` does).
fn octahedral(bits: u32) -> [f32; 3] {
    let e = [(bits & 255) as f32 / 255.0 * 2.0 - 1.0, (bits >> 8) as f32 / 255.0 * 2.0 - 1.0];
    let mut n = [e[0], 1.0 - e[0].abs() - e[1].abs(), e[1]];
    if n[1] < 0.0 {
        let (x, z) = (n[0], n[2]);
        n[0] = (1.0 - z.abs()) * x.signum();
        n[2] = (1.0 - x.abs()) * z.signum();
    }
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    n.map(|v| v / l)
}

/// A ball of leaves round (16, 16, 16) of radius `r`, with a trunk up into it.
fn ball(r: i32) -> impl Fn(i32, i32, i32) -> BlockId {
    move |x, y, z| {
        let (dx, dy, dz) = (x - 16, y - 16, z - 16);
        if dx == 0 && dz == 0 && (6..=16).contains(&y) {
            LOG
        } else if dx * dx + dy * dy + dz * dz <= r * r {
            if dx > 1 {
                BIRCH_LEAVES
            } else {
                LEAVES
            }
        } else {
            AIR
        }
    }
}

#[test]
fn a_lone_leaf_is_three_cards_and_a_buried_one_one() {
    let lone = world(|x, y, z| if (x, y, z) == (5, 5, 5) { LEAVES } else { AIR });
    assert_eq!(cards(&mesh(&mut Mesher::new(), &lone, [0, 0, 0])).len(), 3);
    // A 3×3×3 block of leaves: 26 that show, one inside them.
    let cube =
        world(|x, y, z| if (4..7).contains(&x) && (4..7).contains(&y) && (4..7).contains(&z) { LEAVES } else { AIR });
    assert_eq!(cards(&mesh(&mut Mesher::new(), &cube, [0, 0, 0])).len(), 26 * 3 + 1);
}

#[test]
fn a_chunk_meshes_the_same_whatever_was_meshed_before() {
    let w = world(ball(6));
    let fresh = mesh(&mut Mesher::new(), &w, [0, 0, 0]);
    let mut m = Mesher::new();
    mesh(&mut m, &w, [-1, -1, -1]);
    mesh(&mut m, &w, [0, 0, -1]);
    let again = mesh(&mut m, &w, [0, 0, 0]);
    assert!(fresh.shaped_cutout > 0);
    assert_eq!((fresh.shaped, fresh.shaped_opaque), (again.shaped, again.shaped_opaque));
}

#[test]
fn a_crown_is_lit_as_a_ball() {
    // Cards on the outer shell of the ball have normals pointing out of it, so it shades as one rounded volume.
    let out = mesh(&mut Mesher::new(), &world(ball(6)), [0, 0, 0]);
    let (mut outer, mut out_facing) = (0, 0);
    for (p, n) in cards(&out) {
        let c: [f32; 3] = std::array::from_fn(|k| p.iter().map(|v| v[k]).sum::<f32>() / 4.0 - 16.5);
        let d = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        if d > 4.5 {
            outer += 1;
            out_facing += usize::from(c[0] * n[0] + c[1] * n[1] + c[2] * n[2] > 0.3 * d);
        }
    }
    assert!(outer > 300 && out_facing * 100 >= outer * 90, "{out_facing} of {outer} outer cards face out");
}

#[test]
fn leaves_on_a_chunk_border_are_drawn_once() {
    // A slab of leaves across x = 0: each chunk draws the cards of its own leaves only.
    let w = world(|x, y, z| if (-2..2).contains(&x) && y == 5 && (4..8).contains(&z) { LEAVES } else { AIR });
    let left = cards(&mesh(&mut Mesher::new(), &w, [-1, 0, 0])).len();
    let right = cards(&mesh(&mut Mesher::new(), &w, [0, 0, 0])).len();
    assert_eq!((left, right), (2 * 4 * 3, 2 * 4 * 3));
}

#[test]
fn evergreens_stay_full_and_drop_no_litter_all_year() {
    use super::season::{
        species_look, SPECIES_BAMBOO, SPECIES_EUCALYPTUS, SPECIES_PALM, SPECIES_PINE, SPECIES_REDWOOD,
    };
    for sp in [SPECIES_PINE, SPECIES_REDWOOD, SPECIES_EUCALYPTUS, SPECIES_PALM, SPECIES_BAMBOO] {
        for day in 0..36 {
            let look = species_look(sp as usize, day as f32 / 36.0);
            assert!(
                (look.cover - 1.0).abs() < 1e-5,
                "evergreen {sp} dropped leaves on day {day}: cover={}",
                look.cover
            );
            assert_eq!(look.litter_cover, 0.0, "evergreen {sp} dropped litter on day {day}");
            assert!(look.mix <= 0.30, "evergreen {sp} shifted colour too strongly on day {day}: mix={}", look.mix);
        }
    }
}

#[test]
fn deciduous_and_cherry_species_follow_accurate_phenology() {
    use super::season::{species_look, SPECIES_ACACIA, SPECIES_BAOBAB, SPECIES_BIRCH, SPECIES_CHERRY, SPECIES_OAK};
    // Birch: brilliant golden autumn, bare before winter, buds early in spring.
    let birch_aut = species_look(SPECIES_BIRCH as usize, 0.62);
    assert!(birch_aut.mix > 0.85 && birch_aut.a[0] > 0.75 && birch_aut.a[1] > 0.60 && birch_aut.a[2] < 0.25);
    assert_eq!(species_look(SPECIES_BIRCH as usize, 0.80).cover, 0.0);
    assert!(species_look(SPECIES_BIRCH as usize, 0.00).cover > 0.6);

    // Oak: russet/amber autumn, holds light marcescence into early winter, bare in mid-winter.
    let oak_aut = species_look(SPECIES_OAK as usize, 0.65);
    assert!(oak_aut.mix > 0.8 && oak_aut.a[0] > oak_aut.a[1]);
    let oak_early_win = species_look(SPECIES_OAK as usize, 0.77);
    assert!(oak_early_win.cover > 0.01 && oak_early_win.cover < 0.15);
    assert_eq!(species_look(SPECIES_OAK as usize, 0.88).cover, 0.0);

    // Cherry: pink sakura blossoms in spring (mix=0), pink petal litter late spring, green summer, red autumn, bare winter.
    let cherry_spr = species_look(SPECIES_CHERRY as usize, 0.08);
    assert_eq!(cherry_spr.mix, 0.0);
    assert_eq!(cherry_spr.cover, 1.0);
    let cherry_petals = species_look(SPECIES_CHERRY as usize, 0.14);
    assert!(cherry_petals.litter_cover > 0.7 && cherry_petals.litter_col[0] < 0.0);
    let cherry_sum = species_look(SPECIES_CHERRY as usize, 0.36);
    assert!(cherry_sum.mix > 0.95 && cherry_sum.a[1] > cherry_sum.a[0] && cherry_sum.a[1] > cherry_sum.a[2]);
    let cherry_aut = species_look(SPECIES_CHERRY as usize, 0.63);
    assert!(cherry_aut.mix > 0.95 && cherry_aut.a[0] > cherry_aut.a[1] && cherry_aut.a[0] > cherry_aut.a[2]);
    assert_eq!(species_look(SPECIES_CHERRY as usize, 0.82).cover, 0.0);

    // Savanna: baobab drops bare in dry season; acacia thins (~42%) rather than going bare.
    assert_eq!(species_look(SPECIES_BAOBAB as usize, 0.70).cover, 0.0);
    assert!(species_look(SPECIES_BAOBAB as usize, 0.20).cover > 0.95);
    let acacia_dry = species_look(SPECIES_ACACIA as usize, 0.75);
    assert!(acacia_dry.cover > 0.35 && acacia_dry.cover < 0.50);
    assert_eq!(species_look(SPECIES_ACACIA as usize, 0.25).cover, 1.0);
}

#[test]
fn leaf_cards_encode_species_and_consistent_drop_hash() {
    use super::season::species_of;
    use crate::block::{
        ACACIA_LEAVES, BAMBOO_LEAVES, BAOBAB_LEAVES, CHERRY_BLOSSOM, EUCALYPTUS_LEAVES, FACE_TEX, PALM_FRONDS,
        PINE_LEAVES, REDWOOD_NEEDLES,
    };
    let all = [
        LEAVES,
        BIRCH_LEAVES,
        PINE_LEAVES,
        CHERRY_BLOSSOM,
        PALM_FRONDS,
        BAOBAB_LEAVES,
        ACACIA_LEAVES,
        REDWOOD_NEEDLES,
        EUCALYPTUS_LEAVES,
        BAMBOO_LEAVES,
    ];
    for b in all {
        let w = world(|x, y, z| if (x, y, z) == (5, 5, 5) { b } else { AIR });
        let out = mesh(&mut Mesher::new(), &w, [0, 0, 0]);
        let start = out.shaped_opaque as usize * 12;
        let quads: Vec<&[u32]> = out.shaped[start..].chunks(12).filter(|q| q[0] >> 24 & 7 == 0).collect();
        assert!(!quads.is_empty(), "species block {b} emitted no leaf cards");
        let want_sp = u32::from(species_of(b));
        let base_layer = u32::from(FACE_TEX[b as usize][0]);
        for q in quads {
            let drop0 = ((q[0] >> 30) << 4) | ((q[1] >> 15) & 15);
            for i in 0..4 {
                let v = &q[i * 3..i * 3 + 3];
                let layer = (v[2] >> 16) & 2047;
                let sp = (v[2] >> 27) & 15;
                let is_litter = v[2] >> 31;
                let drop_i = ((v[0] >> 30) << 4) | ((v[1] >> 15) & 15);
                assert_eq!(sp, want_sp, "wrong species index for leaf block {b}");
                assert_eq!(is_litter, 0, "crown card must not be marked as litter");
                assert_eq!(drop_i, drop0, "all four corners of a card must share its drop hash");
                assert!(layer == base_layer || b == LEAVES, "layer mismatch for {b}: {layer}");
            }
        }
    }
}

#[test]
fn season_layers_and_ground_respect_grass_flowers_and_ore_indicators() {
    use super::season::{pack_ground, season_layers, ROLE_FLOWER, ROLE_GRASS, ROLE_NONE, ROLE_WOODLAND};
    use crate::block::tex;
    let mask = season_layers();
    let role = |l: u16| (mask[usize::from(l) >> 4] >> ((usize::from(l) & 15) * 2)) & 3;
    assert_eq!(role(tex::GRASS_TOP), ROLE_GRASS);
    assert_eq!(role(tex::GRASS_TUFT), ROLE_GRASS);
    assert_eq!(role(tex::FERN), ROLE_WOODLAND);
    assert_eq!(role(tex::REED), ROLE_WOODLAND);
    assert_eq!(role(tex::DAISY), ROLE_FLOWER);
    assert_eq!(role(tex::POPPY), ROLE_FLOWER);
    assert_eq!(role(tex::BLUEBELL), ROLE_FLOWER);
    for l in [tex::INDICATOR_IRON, tex::INDICATOR_COAL, tex::INDICATOR_COPPER, tex::INDICATOR_PALE, tex::STONE] {
        assert_eq!(role(l), ROLE_NONE, "layer {l} must stay unshifted year-round");
    }
    let spring = pack_ground(0.12);
    let autumn = pack_ground(0.64);
    let winter = pack_ground(0.82);
    assert!(spring[1] > spring[0], "spring grass is lush green");
    assert!(autumn[0] > autumn[2], "autumn grass warms toward yellow-ochre");
    assert_eq!(spring[9], 1.0, "meadows bloom in spring");
    assert_eq!(winter[9], 0.0, "wildflowers go dormant in winter");
    assert!(spring[10] > 1.0 && spring[11] > 0.95, "spring 3D grass is tall and dense");
    assert!(autumn[10] < spring[10] && autumn[10] > winter[10], "autumn 3D grass shortens toward winter");
    assert!(winter[10] < 0.5 && winter[11] < 0.5, "winter 3D grass is short and sparse");
}
