//! Per-layer worldgen pin tests and deposits-upgrade safety invariants.

use super::*;
use crate::chunk::CHUNK_VOLUME;
use crate::deposits::Tier;
use crate::math::IVec3;
use crate::worldgen::{Biome, WorldGen};

/// Digests heights, biomes, water, caves, trees, ruins, chunks and deposits for an arbitrary `Layers`.
fn layer_digest(layers: Layers) -> u64 {
    let mut g = WorldGen::with_layers(1337, layers);
    let mut bytes = Vec::new();
    for z in (-600..600).step_by(97) {
        for x in (-600..600).step_by(89) {
            bytes.extend_from_slice(&g.height_at(x, z).to_le_bytes());
        }
    }
    let lode = g.find_deposit(IVec3::new(0, 64, 0), Tier::Lode, 16).expect("a lode").center;
    let spawn_y = g.height_at(0, 0) >> 5;
    let mut chunks = vec![
        IVec3::new(0, spawn_y, 0),
        IVec3::new(0, spawn_y - 1, 0),
        IVec3::new(-3, 2, 5),
        IVec3::new(lode.x >> 5, lode.y >> 5, lode.z >> 5),
    ];
    if layers.ground >= 2 {
        let mut seen = Vec::new();
        for z in (-1500..1500).step_by(64) {
            for x in (-1500..1500).step_by(64) {
                let h = g.height_at(x, z);
                let biome = g.biome_at(x, z, h);
                if !seen.contains(&biome) {
                    seen.push(biome);
                    bytes.push(biome as u8);
                    chunks.push(IVec3::new(x >> 5, h >> 5, z >> 5));
                }
            }
        }
        assert_eq!(seen.len(), Biome::ALL.len(), "every biome is sampled");
    }
    if layers.deposits >= 3 {
        chunks
            .extend(g.starter_outcrops().iter().map(|d| IVec3::new(d.center.x >> 5, d.center.y >> 5, d.center.z >> 5)));
    }
    if layers.cover >= 10 {
        if let Some(door) =
            (-3..=3).flat_map(|j| (-3..=3).map(move |i| (i, j))).find_map(|(i, j)| g.ruin_at(i, j)?.vault)
        {
            chunks.push(IVec3::new(door.x >> 5, door.y >> 5, door.z >> 5));
        }
    }
    for c in chunks {
        let chunk = g.generate(c);
        for i in 0..CHUNK_VOLUME {
            bytes.extend_from_slice(&chunk.get(i & 31, i >> 10, (i >> 5) & 31).to_le_bytes());
        }
        for d in g.deposits_for_column(c.x, c.z) {
            let k = d.key;
            let words = [k.tier as i32, k.cx, k.cz, k.ore as i32, k.index as i32, d.center.x, d.center.y, d.center.z];
            words.iter().for_each(|w| bytes.extend_from_slice(&w.to_le_bytes()));
            d.radii.iter().for_each(|r| bytes.extend_from_slice(&r.to_bits().to_le_bytes()));
            bytes.extend_from_slice(&d.seed.to_le_bytes());
        }
    }
    crate::bytes::fnv1a(&bytes)
}

#[test]
fn deposits_v6_to_latest_keep_every_seeded_deposit_identical() {
    let base = Layers::uniform(7);
    let v6 = WorldGen::with_layers(1337, base.with_deposits(6));
    let (mut gas_seen, mut clay_seen) = (0usize, 0usize);
    for v in 7..=WORLDGEN_VERSION {
        let other = WorldGen::with_layers(1337, base.with_deposits(v));
        for cz in (-20..=20).step_by(3) {
            for cx in (-20..=20).step_by(3) {
                let (mut a, mut b) = (Vec::new(), Vec::new());
                v6.seed_deposits(cx, cz, &mut a);
                other.seed_deposits(cx, cz, &mut b);
                // v13 appends independent GAS_SHALE deposits without moving or changing any v6 ore deposit.
                let gas_here = b.iter().filter(|d| d.ore() == crate::block::GAS_SHALE).count();
                if v >= 13 {
                    gas_seen += gas_here;
                } else {
                    assert_eq!(gas_here, 0, "v{v} must not seed gas fields");
                }
                // v14 appends independent clay beds and kaolin pockets the same way.
                let clay_here =
                    b.iter().filter(|d| matches!(d.ore(), crate::block::CLAY | crate::block::KAOLIN)).count();
                if v >= 14 {
                    clay_seen += clay_here;
                } else {
                    assert_eq!(clay_here, 0, "v{v} must not seed clay");
                }
                b.retain(|d| !matches!(d.ore(), crate::block::GAS_SHALE | crate::block::CLAY | crate::block::KAOLIN));
                assert_eq!(a.len(), b.len(), "v{v} column ({cx},{cz}) existing ore deposit count changed");
                for (da, db) in a.iter().zip(&b) {
                    assert_eq!(da.key, db.key, "v{v} key changed");
                    assert_eq!(da.center, db.center, "v{v} center changed");
                    assert_eq!(da.radii, db.radii, "v{v} radii changed");
                    assert_eq!(da.seed, db.seed, "v{v} seed changed");
                }
            }
        }
    }
    assert!(gas_seen > 0, "v13 must seed independent gas fields");
    assert!(clay_seen > 0, "v14 must seed clay beds");
}

#[test]
fn upgrading_deposits_never_moves_ground_water_or_cover() {
    let base = Layers::uniform(7);
    let mut old = WorldGen::with_layers(1337, base);
    let mut up = WorldGen::with_layers(1337, base.with_deposits(WORLDGEN_VERSION));
    for z in (-400..=400).step_by(31) {
        for x in (-400..=400).step_by(29) {
            let h = old.height_at(x, z);
            assert_eq!(up.height_at(x, z), h, "height at ({x},{z})");
            assert_eq!(up.open_water(x, z), old.open_water(x, z), "water at ({x},{z})");
            assert_eq!(up.local_biome_at(x, z, h), old.local_biome_at(x, z, h), "biome at ({x},{z})");
        }
    }
    for cz in -3..=3 {
        for cx in -3..=3 {
            assert_eq!(up.biomes(cx, cz), old.biomes(cx, cz));
            let (x0, z0) = (cx * 32, cz * 32);
            let (ta, tb) = (old.trees_near(x0, z0), up.trees_near(x0, z0));
            assert_eq!(ta.len(), tb.len());
            for (a, b) in ta.iter().zip(&tb) {
                assert_eq!((a.x, a.z, a.ground, a.trunk, a.species), (b.x, b.z, b.ground, b.trunk, b.species));
            }
            let (ra, rb) = (old.ruins_near(x0, z0), up.ruins_near(x0, z0));
            assert_eq!(ra.len(), rb.len());
        }
    }
    // Surface stains (v8) differ on at least one column, proving the upgrade actually took effect.
    assert_ne!(layer_digest(base), layer_digest(base.with_deposits(WORLDGEN_VERSION)));
}

/// Per-layer pin test for every released version `1..=12`: holding the other three layers at v7 and
/// varying one layer at a time pins each released layer behaviour so a change cannot silently leak across layers.
#[test]
fn released_layer_versions_never_change() {
    let base = Layers::uniform(7);
    const GROUND: [u64; 12] = [
        0xfe86_9025_50bf_3e6e, // v1: classic terrain, no biomes
        0x30d8_590e_133b_8aae, // v2: biomes & rock provinces
        0x30d8_590e_133b_8aae, // v3
        0x30d8_590e_133b_8aae, // v4
        0x30d8_590e_133b_8aae, // v5
        0x5e47_3cc6_b633_b81c, // v6: cave zones & thinner tunnels
        0xba30_7644_a319_0263, // v7: landforms (continents, ranges, mesas)
        0x0a46_c560_32b3_c944, // v8: bent spawn ring
        0x0a46_c560_32b3_c944, // v9
        0x0a46_c560_32b3_c944, // v10
        0xccf5_7b74_59d9_d2fe, // v11: lowland start
        0xccf5_7b74_59d9_d2fe, // v12
    ];
    const WATER: [u64; 12] = [
        0xe420_dc2d_6f55_3de1, // v1: dry
        0xe420_dc2d_6f55_3de1, // v2
        0xcf99_f221_f947_0ff4, // v3: sea & round ponds
        0xcf99_f221_f947_0ff4, // v4
        0xcf99_f221_f947_0ff4, // v5
        0xcf99_f221_f947_0ff4, // v6
        0xba30_7644_a319_0263, // v7: straight rivers & round lakes
        0xafc2_ee5d_3480_6f4c, // v8: winding rivers, falls, irregular shores
        0xafc2_ee5d_3480_6f4c, // v9
        0xafc2_ee5d_3480_6f4c, // v10
        0xafc2_ee5d_3480_6f4c, // v11
        0xafc2_ee5d_3480_6f4c, // v12
    ];
    const COVER: [u64; 12] = [
        0xba30_7644_a319_0263, // v1..v8: oaks, no ruins
        0xba30_7644_a319_0263,
        0xba30_7644_a319_0263,
        0xba30_7644_a319_0263,
        0xba30_7644_a319_0263,
        0xba30_7644_a319_0263,
        0xba30_7644_a319_0263,
        0xba30_7644_a319_0263,
        0x304c_cfc5_c544_c441, // v9: biome tree species & ruins
        0x90cd_2ed8_d4c7_941a, // v10: ruin vaults
        0x90cd_2ed8_d4c7_941a, // v11
        0x405d_122a_9ff0_44b6, // v12: grown trees & forest biomes
    ];
    const DEPOSITS: [u64; 12] = [
        0x7812_5da2_1011_a319, // v1: classic coal/iron/copper
        0xd622_a4d9_c088_7c39, // v2: geology-driven ores & stains
        0x3fe7_92eb_a0f6_60da, // v3: depth bands & starter set
        0x7e7c_3ff6_4992_bdd1, // v4: shallower bands & 6 starters
        0x3d10_b4a1_834c_5fe7, // v5: bauxite (600+)
        0xba30_7644_a319_0263, // v6: bauxite (300+), oil sand, uranium
        0xba30_7644_a319_0263, // v7
        0x40cc_43a0_6064_b98a, // v8: irregular stain shapes (ore deposits identical to v6)
        0x40cc_43a0_6064_b98a, // v9
        0x40cc_43a0_6064_b98a, // v10
        0x40cc_43a0_6064_b98a, // v11
        0x40cc_43a0_6064_b98a, // v12
    ];
    for v in 1..=12u32 {
        let i = (v - 1) as usize;
        assert_eq!(layer_digest(Layers { ground: v, ..base }), GROUND[i], "ground v{v}");
        assert_eq!(layer_digest(Layers { water: v, ..base }), WATER[i], "water v{v}");
        assert_eq!(layer_digest(Layers { cover: v, ..base }), COVER[i], "cover v{v}");
        assert_eq!(layer_digest(Layers { deposits: v, ..base }), DEPOSITS[i], "deposits v{v}");
    }
}
