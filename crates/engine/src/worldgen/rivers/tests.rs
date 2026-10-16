//! Version 7 rivers and lakes: they exist, descend to the sea or a lake, hold still water in their channels, keep off
//! spawn, thin out in dry country and leave older versions alone.

use super::*;
use crate::block::{AIR, WATER};
use crate::chunk::Chunk;
use crate::math::IVec3;

const SEEDS: [u32; 3] = [2024, 1337, 7];

/// The (x, z) columns of the segment's axis, about every `step` blocks.
fn axis_points(s: &Seg, step: f64) -> Vec<(i32, i32)> {
    s.axis(step)
}

/// Every segment of the cells within `cells` of the origin.
fn all_segs(g: &WorldGen, cells: i32) -> Vec<Rc<Seg>> {
    let mut out = Vec::new();
    for gz in -cells..=cells {
        for gx in -cells..=cells {
            out.extend(g.node_segs(gx, gz).iter().cloned());
        }
    }
    out
}

#[test]
fn rivers_and_lakes_exist_in_every_world() {
    for seed in SEEDS {
        let g = WorldGen::new(seed);
        let segs = all_segs(&g, 8);
        let lakes = segs.iter().filter(|s| s.len == 0.0).count();
        println!("seed {seed}: {} reaches, {lakes} lakes", segs.len() - lakes);
        assert!(segs.len() - lakes >= 8, "seed {seed}: rivers");
        assert!(lakes >= 2, "seed {seed}: lakes");
    }
}

#[test]
fn water_levels_never_rise_downstream_and_match_the_nodes() {
    let g = WorldGen::new(1337);
    for gz in -8..=8 {
        for gx in -8..=8 {
            let n = g.river_node(gx, gz);
            for s in g.node_segs(gx, gz).iter().filter(|s| s.len > 0.0) {
                let d = g.river_down(&n).expect("a river has a downstream");
                assert!(s.levels.windows(2).all(|w| w[0] >= w[1]), "{:?}", s.levels);
                assert_eq!((s.levels[0], *s.levels.last().unwrap()), (n.level, d.level));
                assert!(n.level >= d.level && d.level >= SEA_LEVEL);
            }
        }
    }
}

#[test]
fn every_chain_ends_in_the_sea_or_a_lake() {
    for seed in SEEDS {
        let g = WorldGen::new(seed);
        for gz in -12..=12 {
            for gx in -12..=12 {
                let mut n = g.river_node(gx, gz);
                for _ in 0..64 {
                    let down = g.river_down(&n);
                    if !n.wet || down.is_none() {
                        // The chain's end: the sea reached, or this node (not wet or at a low point) holds a lake.
                        assert!(n.ocean || !n.wet || g.has_lake(&n, down.is_some()), "seed {seed}: node {gx},{gz}");
                    }
                    match down {
                        Some(d) if d.gx.abs() <= 40 && d.gz.abs() <= 40 => n = d,
                        _ => break,
                    }
                }
            }
        }
    }
}

#[test]
fn drier_country_has_fewer_rivers() {
    let (mut dry, mut wet) = ((0, 0), (0, 0));
    for seed in SEEDS {
        let g = WorldGen::new(seed);
        for gz in -30..=30 {
            for gx in -30..=30 {
                let n = g.river_node(gx, gz);
                if n.ocean {
                    continue;
                }
                let m = g.moisture.fbm2(n.x as f64 / 900.0, n.z as f64 / 900.0, 3);
                let bin = if m < -0.1 {
                    &mut dry
                } else if m > 0.1 {
                    &mut wet
                } else {
                    continue;
                };
                bin.0 += usize::from(n.wet);
                bin.1 += 1;
            }
        }
    }
    let share = |b: (usize, usize)| b.0 as f64 / b.1 as f64;
    println!("wet nodes: dry country {:?}, wet country {:?}", dry, wet);
    assert!(dry.1 > 50 && wet.1 > 50 && share(dry) + 0.3 < share(wet));
}

#[test]
fn spawn_is_dry_land_with_no_river_near() {
    for seed in [2024, 1337, 7, 1, 99, 4242, 5] {
        let g = WorldGen::new(seed);
        for z in (-90..=90).step_by(6) {
            for x in (-90..=90).step_by(6) {
                if x * x + z * z <= 90 * 90 {
                    assert!(
                        !g.river_near(x, z) || g.river_sample(x, z).is_some_and(|s| s.sd > 0.0),
                        "seed {seed}: {x},{z}"
                    );
                    let h = g.height_at(x, z);
                    assert!(g.river_water(x, z, h).is_none(), "seed {seed}: river water at {x},{z}");
                }
            }
        }
        let h = g.height_at(0, 0);
        assert!(h > SEA_LEVEL + 1 && g.water_top(0, 0, h) < h, "seed {seed}: spawn is dry ({h})");
    }
}

/// Water blocks in the river columns around a reach (every `step` blocks) with air beside or below them.
fn leaks_along(g: &mut WorldGen, s: &Seg, step: f64) -> (usize, usize) {
    let mut chunks: rustc_hash::FxHashMap<IVec3, Chunk> = Default::default();
    let mut get = |g: &mut WorldGen, p: IVec3| {
        let c = IVec3::new(p.x >> 5, p.y >> 5, p.z >> 5);
        chunks.entry(c).or_insert_with(|| g.generate(c)).get(
            (p.x & 31) as usize,
            (p.y & 31) as usize,
            (p.z & 31) as usize,
        )
    };
    let (mut water, mut leaks) = (0, 0);
    for (cx, cz) in axis_points(s, step) {
        for z in cz - 10..=cz + 10 {
            for x in cx - 10..=cx + 10 {
                let h = g.height_at(x, z);
                for y in (h + 1).max(1)..=g.water_top(x, z, h) {
                    water += 1;
                    let p = IVec3::new(x, y, z);
                    assert_eq!(get(g, p), WATER, "water expected at {p:?}");
                    let beside = [
                        IVec3::new(1, 0, 0),
                        IVec3::new(-1, 0, 0),
                        IVec3::new(0, 0, 1),
                        IVec3::new(0, 0, -1),
                        IVec3::new(0, -1, 0),
                    ];
                    leaks += beside.iter().filter(|&&d| get(g, p + d) == AIR).count();
                }
            }
        }
    }
    (water, leaks)
}

#[test]
fn river_channels_hold_their_water() {
    for seed in SEEDS {
        let mut g = WorldGen::new(seed);
        let segs = all_segs(&g, 6);
        let (mut water, mut leaks) = (0, 0);
        for s in segs.iter().filter(|s| s.len > 0.0).take(6) {
            let (w, l) = leaks_along(&mut g, s, 24.0);
            (water, leaks) = (water + w, leaks + l);
        }
        println!("seed {seed}: {water} water blocks, {leaks} leaking faces");
        assert!(water > 500, "seed {seed}: rivers hold water");
        assert!(leaks * 100 <= water, "seed {seed}: {leaks} leaks in {water} water blocks");
    }
}

#[test]
fn lakes_hold_their_water() {
    for seed in SEEDS {
        let mut g = WorldGen::new(seed);
        let segs = all_segs(&g, 6);
        let lake = segs.iter().find(|s| s.len == 0.0).expect("a lake");
        let (water, leaks) = leaks_along(&mut g, lake, 100.0);
        println!("seed {seed}: lake of {water} blocks, {leaks} leaks");
        assert!(water > 200 && leaks * 100 <= water, "seed {seed}: {leaks} leaks in {water}");
    }
}

/// The farthest any point of `pts` lies from the line through its first and last.
fn bulge(pts: &[(i32, i32)]) -> f64 {
    let ((ax, az), (bx, bz)) = (pts[0], pts[pts.len() - 1]);
    let (dx, dz) = ((bx - ax) as f64, (bz - az) as f64);
    let len = (dx * dx + dz * dz).sqrt().max(1.0);
    pts.iter().map(|&(x, z)| (((x - ax) as f64 * dz - (z - az) as f64 * dx) / len).abs()).fold(0.0, f64::max)
}

#[test]
fn version_8_lakes_are_not_round() {
    for seed in SEEDS {
        let g = WorldGen::new(seed);
        let lakes: Vec<_> = all_segs(&g, 6).into_iter().filter(|s| s.len == 0.0).collect();
        assert!(lakes.len() >= 5, "seed {seed}: {} lakes", lakes.len());
        for l in lakes {
            let (x, z) = (l.a.0 as i32, l.a.1 as i32);
            let (hi, lo) = crate::worldgen::shore::spread(x, z, |px, pz| l.sample(px as f64, pz as f64, 0.0).sd < 0.0);
            assert!(hi >= 1.25 * lo, "seed {seed}: lake at {x},{z}: {lo} to {hi}");
        }
    }
}
#[test]
fn version_8_reaches_wind() {
    for seed in SEEDS {
        let g = WorldGen::new(seed);
        let (mut windows, mut straight, mut reaches, mut bent) = (0, 0, 0, 0);
        for s in all_segs(&g, 6).iter().filter(|s| s.len > 0.0) {
            let pts = s.axis(6.0);
            // Windows of about 48 blocks (8 points 6 apart) that stay within a block of their chord.
            for w in pts.windows(8) {
                windows += 1;
                straight += usize::from(bulge(w) < 1.0);
            }
            reaches += 1;
            bent += usize::from(bulge(&pts) >= 8.0);
        }
        println!("seed {seed}: {straight} of {windows} windows straight, {bent} of {reaches} reaches bent 8+");
        assert!(straight * 25 <= windows, "seed {seed}: {straight} of {windows} stretches are straight");
        assert_eq!(bent, reaches, "seed {seed}: every reach bends");
    }
}

/// An ASCII map of the land round spawn (run with `--ignored --nocapture`): `~` sea, `=` river or lake, `:` pond,
/// `^` high ground (digits would be better, but this fits a terminal).
#[test]
#[ignore]
fn map_of_the_land() {
    for version in [6, 7] {
        let g = WorldGen::with_version(1337, version);
        let mut bands = [0u32; 5];
        for z in (-3000..3000).step_by(30) {
            for x in (-3000..3000).step_by(30) {
                let h = g.height_at(x, z);
                bands[[SEA_LEVEL, 100, 150, 200].iter().filter(|&&b| h > b).count()] += 1;
            }
        }
        println!("version {version}: share by height (sea, 62-100, 100-150, 150-200, 200+) {bands:?}");
    }
    let g = WorldGen::new(1337);
    for row in -40..40 {
        let line: String = (-80..80)
            .map(|col| {
                let (x, z) = (col * 24, row * 24);
                let h = g.height_at(x, z);
                let top = g.water_top(x, z, h);
                match () {
                    _ if top > h && h < SEA_LEVEL => '~',
                    _ if top > h && g.river_near(x, z) => '=',
                    _ if top > h => ':',
                    _ if h > 150 => '^',
                    _ if h > 100 => 'n',
                    _ => '.',
                }
            })
            .collect();
        println!("{line}");
    }
    // A close-up, 3 blocks a character, round the middle of the first long reach.
    let s = all_segs(&g, 4).into_iter().find(|s| s.len > 220.0).expect("a long reach");
    let (mx, mz) = s.axis(6.0)[s.axis(6.0).len() / 2];
    for row in -25..25 {
        let line: String = (-60..60)
            .map(|col| {
                let (x, z) = (mx + col * 3, mz + row * 3);
                let h = g.height_at(x, z);
                if g.water_top(x, z, h) > h {
                    char::from(b'0' + (g.water_top(x, z, h) % 10) as u8)
                } else {
                    '.'
                }
            })
            .collect();
        println!("{line}");
    }
}

/// Generation cost of versions 6, 7 and 8 round spawn and in river country 1,500 blocks out (run with `--release
/// --ignored --nocapture --test-threads=1`).
#[test]
#[ignore]
fn bench_generation() {
    for (name, (x0, z0)) in [("spawn", (0, 0)), ("far", (47, 47))] {
        for version in [6, 7, 8, 7, 8] {
            let mut g = WorldGen::with_version(1337, version);
            let start = std::time::Instant::now();
            for cz in z0 - 6..z0 + 6 {
                for cx in x0 - 6..x0 + 6 {
                    for cy in 1..4 {
                        g.generate(IVec3::new(cx, cy, cz));
                    }
                }
            }
            println!("{name}, version {version}: {:?} for 432 chunks", start.elapsed());
        }
    }
}

/// Version 7's rivers as released (recorded before version 8): heights, water and the chunk blocks along reaches
/// and lakes round spawn, since `released_versions_never_change` seldom samples a river.
#[test]
fn version_7_rivers_never_change() {
    let mut g = WorldGen::with_version(1337, 7);
    let mut bytes = Vec::new();
    let segs = all_segs(&g, 5);
    for s in segs.iter().take(12) {
        for (x, z) in axis_points(s, 9.0) {
            for (dx, dz) in [(0, 0), (4, 3), (-7, 5), (11, -9)] {
                let h = g.height_at(x + dx, z + dz);
                bytes.extend_from_slice(&h.to_le_bytes());
                bytes.extend_from_slice(&g.water_top(x + dx, z + dz, h).to_le_bytes());
            }
        }
        let (x, z) = axis_points(s, 9.0)[1];
        let chunk = g.generate(IVec3::new(x >> 5, g.height_at(x, z) >> 5, z >> 5));
        bytes.extend((0..crate::chunk::CHUNK_VOLUME).map(|i| chunk.get(i & 31, i >> 10, (i >> 5) & 31)));
    }
    assert_eq!(crate::bytes::fnv1a(&bytes), 0x119e_c1c9_a2d5_08b9, "version 7 rivers");
}

#[test]
fn older_versions_have_no_rivers() {
    let g = WorldGen::with_version(1337, 6);
    assert!(g.rivers.nodes.borrow().is_empty());
    for z in (-600..600).step_by(40) {
        for x in (-600..600).step_by(40) {
            g.height_at(x, z);
        }
    }
    assert!(g.rivers.nodes.borrow().is_empty() && g.rivers.segs.borrow().is_empty());
}
