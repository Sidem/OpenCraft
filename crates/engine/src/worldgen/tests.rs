use super::*;
use crate::deposits::Tier;

#[test]
fn generation_is_deterministic() {
    let mut a = WorldGen::new(7);
    let mut b = WorldGen::new(7);
    for p in [IVec3::new(0, 1, 0), IVec3::new(-3, 2, 5), IVec3::new(10, 0, -10)] {
        let (ca, cb) = (a.generate(p), b.generate(p));
        for i in 0..CHUNK_VOLUME {
            let (x, y, z) = (i & 31, i >> 10, (i >> 5) & 31);
            assert_eq!(ca.get(x, y, z), cb.get(x, y, z));
        }
    }
}

/// Blocks, heights and deposits that `version` generates for seed 1337 around spawn, a cave and a lode,
/// and from version 2 on a surface chunk of the first column found in each biome.
fn digest(version: u32) -> u64 {
    let mut g = WorldGen::with_version(1337, version);
    let mut bytes = Vec::new();
    for z in (-600..600).step_by(97) {
        for x in (-600..600).step_by(89) {
            bytes.extend_from_slice(&g.height_at(x, z).to_le_bytes());
        }
    }
    let lode = g.find_deposit(IVec3::new(0, 64, 0), Tier::Lode, 16).expect("a lode").center;
    let lode_chunk = IVec3::new(lode.x >> 5, lode.y >> 5, lode.z >> 5);
    let spawn_y = g.height_at(0, 0) >> 5;
    let mut chunks = vec![IVec3::new(0, spawn_y, 0), IVec3::new(0, spawn_y - 1, 0), IVec3::new(-3, 2, 5), lode_chunk];
    if version >= 2 {
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
    if version >= 3 {
        chunks
            .extend(g.starter_outcrops().iter().map(|d| IVec3::new(d.center.x >> 5, d.center.y >> 5, d.center.z >> 5)));
    }
    for c in chunks {
        let chunk = g.generate(c);
        for i in 0..CHUNK_VOLUME {
            bytes.push(chunk.get(i & 31, i >> 10, (i >> 5) & 31));
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

/// Worlds regenerate their untouched terrain from their own version, so released output is frozen.
/// Version 1 was recorded before Milestone 4, version 2 before Milestone 5, version 3 before version 4,
/// versions 4 and 5 before version 6; if one fails, a change leaked into a released version.
#[test]
fn released_versions_never_change() {
    assert_eq!(digest(1), 0xefee_9cc6_179e_584f, "version 1");
    assert_eq!(digest(2), 0x24f4_7dcb_aceb_e0c5, "version 2");
    assert_eq!(digest(3), 0x25cd_1e52_1d22_b1f1, "version 3");
    assert_eq!(digest(4), 0xeabc_f56c_e953_ede9, "version 4");
    assert_eq!(digest(5), 0x742c_d9cc_c3ab_3f75, "version 5");
    assert_eq!(digest(6), 0x08cc_c363_a4e3_5bee, "version 6");
}

#[test]
fn bottom_is_bedrock_and_sky_is_empty() {
    let mut g = WorldGen::new(1);
    let bottom = g.generate(IVec3::new(0, 0, 0));
    assert_eq!(bottom.get(5, 0, 5), BEDROCK);
    assert_eq!(g.generate(IVec3::new(0, WORLD_HEIGHT_CHUNKS - 1, 0)).as_uniform(), Some(AIR));
}

#[test]
fn spawn_column_is_solid_below_surface() {
    let mut g = WorldGen::new(99);
    let h = g.height_at(0, 0);
    let c = g.generate(IVec3::new(0, h >> 5, 0));
    assert_ne!(c.get(0, (h & 31) as usize, 0), AIR);
    assert!(h > 4 && h < WORLD_HEIGHT - 16);
}

#[test]
fn ore_is_owned_and_some_is_visible_near_spawn() {
    let mut g = WorldGen::new(1337);
    let (mut ore, mut exposed) = (0, 0);
    let mut tiers = [0u32; 3];
    for cz in -2..=2 {
        for cx in -2..=2 {
            for cy in 0..WORLD_HEIGHT_CHUNKS {
                let c = g.generate(IVec3::new(cx, cy, cz));
                for i in 0..CHUNK_VOLUME {
                    let (x, y, z) = (i & 31, i >> 10, (i >> 5) & 31);
                    let b = c.get(x, y, z);
                    if !is_ore(b) {
                        continue;
                    }
                    ore += 1;
                    let p = IVec3::new(cx * 32 + x as i32, cy * 32 + y as i32, cz * 32 + z as i32);
                    let d = g.deposit_at(p).expect("every ore block belongs to a deposit");
                    assert_eq!(d.ore(), b);
                    tiers[d.tier() as usize] += 1;
                    if y < 31 && c.get(x, y + 1, z) == AIR {
                        exposed += 1;
                    }
                }
            }
        }
    }
    println!("ore blocks {ore}, exposed {exposed}, by tier (lode, vein, outcrop) {tiers:?}");
    assert!(tiers[Tier::Outcrop as usize] > 0 && tiers[Tier::Vein as usize] > 0);
    assert!(exposed > 20, "only {exposed} ore blocks see the sky or a cave");
}

#[test]
fn lodes_exist_within_prospecting_range() {
    let g = WorldGen::new(1337);
    let lode = g.find_deposit(IVec3::new(0, 64, 0), Tier::Lode, 16).expect("a lode within ~500 blocks");
    assert!(lode.center.y < 32, "lodes sit near bedrock");
}

#[test]
fn terrain_height_distribution_is_reasonable() {
    let g = WorldGen::new(1337);
    let (mut lo, mut hi, mut sum, mut n) = (i32::MAX, i32::MIN, 0i64, 0i64);
    for z in (-4000..4000).step_by(37) {
        for x in (-4000..4000).step_by(37) {
            let h = g.height_at(x, z);
            lo = lo.min(h);
            hi = hi.max(h);
            sum += h as i64;
            n += 1;
        }
    }
    let mean = sum / n;
    println!("height min {lo} max {hi} mean {mean}");
    assert!(lo >= 4 && hi <= WORLD_HEIGHT - 16);
    assert!(hi - lo > 60, "terrain too flat: {lo}..{hi}");
}
