use super::*;

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
