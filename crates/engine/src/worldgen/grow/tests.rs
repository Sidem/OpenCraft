//! Grown trees on their own: every leaf held, trunks clear and reachable, each kind its own silhouette.

use super::kinds::*;
use super::*;
use crate::block::{is_leaves, is_log, BAMBOO, SAPLING};
use std::collections::HashMap;

type Blocks = HashMap<(i32, i32, i32), BlockId>;

const FOOT: IVec3 = IVec3 { x: 0, y: 40, z: 0 };

fn tree(kind: usize, height: i32, seed: u32) -> Blocks {
    let mut m = HashMap::new();
    grow(kind, FOOT, height, seed, |p, id, replace| {
        if replace || !m.contains_key(&(p.x, p.y, p.z)) {
            m.insert((p.x, p.y, p.z), id);
        }
    });
    m
}

/// Every kind at its least and greatest height, on a few seeds.
fn samples() -> impl Iterator<Item = (usize, i32, u32, Blocks)> {
    (0..KINDS.len()).flat_map(|kind| {
        let (least, more) = KINDS[kind].height;
        [least, least + more as i32]
            .into_iter()
            .flat_map(move |h| (1..=6).map(move |seed| (kind, h, seed, tree(kind, h, seed))))
    })
}

#[test]
fn every_leaf_is_held_by_a_log() {
    for (kind, h, seed, m) in samples() {
        let mut dist: HashMap<(i32, i32, i32), i32> =
            m.iter().filter(|(_, &b)| is_log(b)).map(|(&p, _)| (p, 0)).collect();
        let mut frontier: Vec<_> = dist.keys().copied().collect();
        for step in 1..=crate::sim::LEAF_SUPPORT_STEPS {
            let mut next = Vec::new();
            for (x, y, z) in frontier {
                for n in [(x + 1, y, z), (x - 1, y, z), (x, y + 1, z), (x, y - 1, z), (x, y, z + 1), (x, y, z - 1)] {
                    if m.get(&n).is_some_and(|&b| is_leaves(b)) && !dist.contains_key(&n) {
                        dist.insert(n, step);
                        next.push(n);
                    }
                }
            }
            frontier = next;
        }
        let loose = m.iter().filter(|(p, &b)| is_leaves(b) && !dist.contains_key(p)).count();
        let leaves = m.values().filter(|&&b| is_leaves(b)).count();
        assert_eq!(loose, 0, "kind {kind} height {h} seed {seed}: {loose} of {leaves} leaves would decay");
        assert!(leaves > 10, "kind {kind} height {h} seed {seed}: only {leaves} leaves");
    }
}

#[test]
fn nothing_reaches_past_the_reach() {
    for (kind, h, seed, m) in samples() {
        let far = m.keys().map(|p| (p.0 - FOOT.x).abs().max((p.2 - FOOT.z).abs())).max().unwrap();
        assert!(far <= REACH, "kind {kind} height {h} seed {seed} reaches {far}");
        let top = m.keys().map(|p| p.1).max().unwrap();
        assert!(top <= FOOT.y + top_of(h), "kind {kind} height {h} seed {seed}: top {top}");
    }
}

/// Whether (x, z) is in the foot of a kind's trunk (1, 2 or 3 logs square, as `shapes::Builder::trunk` lays it).
fn in_trunk(kind: usize, x: i32, z: i32) -> bool {
    let w = KINDS[kind].width;
    let lo = -(w - 1) / 2;
    (lo..lo + w).contains(&(x - FOOT.x)) && (lo..lo + w).contains(&(z - FOOT.z))
}

/// Whether the block at `target` can be seen from an eye at `eye` without a leaf in the way.
fn clear(m: &Blocks, eye: [f32; 3], target: (i32, i32, i32)) -> bool {
    let to = [target.0 as f32 + 0.5, target.1 as f32 + 0.5, target.2 as f32 + 0.5];
    let d = [to[0] - eye[0], to[1] - eye[1], to[2] - eye[2]];
    let steps = 200;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let c = (
            (eye[0] + d[0] * t).floor() as i32,
            (eye[1] + d[1] * t).floor() as i32,
            (eye[2] + d[2] * t).floor() as i32,
        );
        if c == target {
            return true;
        }
        if m.get(&c).is_some_and(|&b| is_leaves(b)) {
            return false;
        }
    }
    true
}

#[test]
fn a_trunk_can_be_seen_and_reached_from_the_ground() {
    for (kind, h, seed, m) in samples() {
        let w = KINDS[kind].width;
        // The trunk's logs a player on the ground could reach: low ones near the foot.
        let trunk =
            m.iter().filter(|(p, &b)| is_log(b) && p.1 > FOOT.y && p.1 <= FOOT.y + 4 && in_trunk(kind, p.0, p.2));
        for (&p, _) in trunk {
            let spots = [(3, 0), (-3, 0), (0, 3), (0, -3), (2, 2), (-2, 2), (2, -2), (-2, -2)];
            let seen = spots.iter().any(|&(dx, dz)| {
                let (sx, sz) = (FOOT.x + dx + dx.signum() * w / 2, FOOT.z + dz + dz.signum() * w / 2);
                let eye = [sx as f32 + 0.5, (FOOT.y + 1) as f32 + 1.6, sz as f32 + 0.5];
                let far = ((p.0 - sx) as f32).hypot((p.2 - sz) as f32).hypot(p.1 as f32 + 0.5 - eye[1]);
                far <= 5.0 && clear(&m, eye, p)
            });
            assert!(seen, "kind {kind} height {h} seed {seed}: trunk log {p:?} out of reach or hidden by leaves");
        }
    }
}

#[test]
fn below_the_crown_the_trunk_stands_clear() {
    for (kind, h, seed, m) in samples() {
        let lowest_limb =
            m.iter().filter(|(p, &b)| is_log(b) && p.1 > FOOT.y && !in_trunk(kind, p.0, p.2)).map(|(p, _)| p.1).min();
        let Some(base) = lowest_limb else { continue };
        let w = KINDS[kind].width;
        let near = m.iter().filter(|(p, &b)| {
            is_leaves(b) && p.1 < base - 1 && p.1 > FOOT.y && (p.0 - FOOT.x).abs() <= w && (p.2 - FOOT.z).abs() <= w
        });
        assert_eq!(near.count(), 0, "kind {kind} height {h} seed {seed}: leaves by the bole under {base}");
    }
}

/// (width across, height, lowest leaf, highest leaf) of a tree's leaves, relative to the foot.
fn crown(m: &Blocks) -> (i32, i32, i32, i32) {
    let leaves: Vec<_> = m.iter().filter(|(_, &b)| is_leaves(b)).map(|(p, _)| *p).collect();
    let (lo_x, hi_x) = (leaves.iter().map(|p| p.0).min().unwrap(), leaves.iter().map(|p| p.0).max().unwrap());
    let (lo_y, hi_y) = (leaves.iter().map(|p| p.1).min().unwrap(), leaves.iter().map(|p| p.1).max().unwrap());
    (hi_x - lo_x + 1, hi_y - lo_y + 1, lo_y - FOOT.y, hi_y - FOOT.y)
}

#[test]
fn each_kind_has_its_own_silhouette() {
    let avg = |kind: usize| {
        let (least, more) = KINDS[kind].height;
        let h = least + more as i32 / 2;
        let all: Vec<_> = (1..=8).map(|s| crown(&tree(kind, h, s))).collect();
        let mean = |f: fn(&(i32, i32, i32, i32)) -> i32| all.iter().map(f).sum::<i32>() as f32 / all.len() as f32;
        (mean(|c| c.0), mean(|c| c.1), mean(|c| c.2), mean(|c| c.3))
    };
    let oak = avg(OAK);
    let birch = avg(BIRCH);
    let acacia = avg(ACACIA);
    let redwood = avg(REDWOOD);
    let palm = avg(PALM);
    assert!(oak.0 >= oak.1 * 0.8, "an oak's crown is about as wide as tall: {oak:?}");
    assert!(birch.0 / birch.1 < oak.0 / oak.1, "a birch's is narrower: {birch:?} {oak:?}");
    assert!(acacia.1 <= 4.0 && acacia.0 >= 9.0, "an acacia's is flat and wide: {acacia:?}");
    assert!(redwood.3 >= 28.0 && redwood.2 >= 10.0, "a redwood is tall with a long bare trunk: {redwood:?}");
    assert!(palm.2 >= KINDS[PALM].height.0 as f32 - 2.0, "a palm's leaves are all at its top: {palm:?}");
}

#[test]
fn bamboo_culms_stand_apart() {
    for seed in 1..=8 {
        let m = tree(BAMBOO_CLUMP, 12, seed);
        let feet: Vec<_> = m.iter().filter(|(p, &b)| b == BAMBOO && p.1 == FOOT.y + 1).map(|(p, _)| *p).collect();
        assert!(feet.len() >= 4, "seed {seed}: {} culms", feet.len());
        for a in &feet {
            for b in &feet {
                let apart = (a.0 - b.0).abs().max((a.2 - b.2).abs());
                assert!(a == b || apart >= 2, "seed {seed}: culms {a:?} and {b:?} touch");
            }
        }
    }
}

#[test]
fn every_kind_grows_somewhere_and_each_where_it_should() {
    use crate::block::{GRASS, SAND};
    use crate::worldgen::Biome::*;
    let mut seen = [false; KINDS.len()];
    let all = [Plains, Desert, Highlands, Lowlands, BasaltFields, Savanna, Boreal, Woodland, BambooForest, Redwoods];
    for biome in all {
        for top in [GRASS, SAND] {
            for t in [-0.3, -0.12, 0.0, 0.04, 0.08, 0.2] {
                for (shore, grove) in [(false, false), (false, true), (true, false)] {
                    for h in (0..20).map(|r| r << 24) {
                        if let Some(k) = pick(biome, top, t, shore, grove, h) {
                            seen[k] = true;
                            assert!(k != PALM || shore, "palms only by water");
                            assert!(k != REDWOOD || biome == Redwoods, "redwoods only in redwood forests");
                            assert!(!matches!(k, ACACIA | BAOBAB) || biome == Savanna, "acacia and baobab: savanna");
                            assert!(k != BAMBOO_CLUMP || biome == BambooForest, "bamboo only in bamboo forests");
                            assert!(k != CHERRY || biome == Plains, "cherry groves on plains");
                            assert!(biome != Desert || k == PALM, "a desert's only new tree is the palm");
                        }
                    }
                }
            }
        }
    }
    assert!(seen.iter().all(|&s| s), "never picked: {:?}", (0..KINDS.len()).filter(|&k| !seen[k]).collect::<Vec<_>>());
}

/// The forest biomes lie inside their broad ones, and only from generator 12.
#[test]
fn forest_biomes_split_the_broad_ones_from_version_12() {
    use crate::worldgen::WorldGen;
    let (old, new) = (WorldGen::with_version(1337, 11), WorldGen::with_version(1337, 12));
    let mut split = 0;
    for i in 0..4000 {
        let (x, z) = ((i % 64) * 41 - 1300, (i / 64) * 41 - 1300);
        let h = new.height_at(x, z);
        let (broad, local) = (new.biome_at(x, z, h), new.local_biome_at(x, z, h));
        assert_eq!(local.base(), broad, "({x}, {z})");
        split += usize::from(local != broad);
        let h = old.height_at(x, z);
        assert_eq!(old.local_biome_at(x, z, h), old.biome_at(x, z, h), "version 11 has no forest biomes");
    }
    assert!(split > 400, "forest biomes cover a good share of the land: {split} of 4000");
}
#[test]
fn saplings_grow_their_own_kind() {
    assert_eq!(kind_of_sapling(SAPLING), OAK);
    for (i, k) in KINDS.iter().enumerate().filter(|&(i, _)| i != BROAD) {
        assert_eq!(kind_of_sapling(k.sapling), i, "kind {i}");
    }
}

/// Where each kind grows in a new world (`cargo test -q species_census -- --ignored --nocapture`): how many stand
/// within `R` columns of spawn, and the nearest of each, for screenshots and placement tuning.
#[test]
#[ignore]
fn species_census() {
    const R: i32 = 48;
    for seed in [1337, 2024, 7] {
        let mut g = crate::worldgen::WorldGen::new(seed);
        let mut count = [0u32; KINDS.len()];
        let mut near: [Option<(i32, i32, i32, i32)>; KINDS.len()] = [None; KINDS.len()];
        for cz in -R..R {
            for cx in -R..R {
                let col = g.column(cx, cz);
                for t in col.trees.iter().filter(|t| !t.grown.is_empty()) {
                    // Each tree is listed by every column it reaches: count it in the one holding its foot.
                    if t.x.div_euclid(32) != cx || t.z.div_euclid(32) != cz {
                        continue;
                    }
                    count[t.species] += 1;
                    let d = t.x * t.x + t.z * t.z;
                    if near[t.species].is_none_or(|n| d < n.3) {
                        near[t.species] = Some((t.x, t.ground, t.z, d));
                    }
                }
            }
        }
        let total: u32 = count.iter().sum();
        println!("seed {seed}: {total} trees within {} blocks", R * 32);
        for k in 0..KINDS.len() {
            let n = near[k].map(|(x, y, z, _)| format!("{x} {y} {z}")).unwrap_or_default();
            println!(
                "  kind {k:2}: {:6} ({:4.1}%)  nearest {n}",
                count[k],
                100.0 * count[k] as f64 / total.max(1) as f64
            );
        }
    }
}

/// Each kind drawn from the side and from above (`cargo test -q sketch_kinds -- --ignored --nocapture`; `KIND=7` for
/// one): what you see from the front and from above, `#` log, `*` leaves, with each kind's log and leaf counts.
#[test]
#[ignore]
fn sketch_kinds() {
    let only: Option<usize> = std::env::var("KIND").ok().and_then(|v| v.parse().ok());
    for kind in (0..KINDS.len()).filter(|k| only.is_none_or(|o| o == *k)) {
        let (least, more) = KINDS[kind].height;
        for seed in 1..=2 {
            let m = tree(kind, least + more as i32 / 2, seed);
            let top = m.keys().map(|k| k.1).max().unwrap();
            println!("kind {kind} seed {seed}: side (x across, y up), then top (x, z)");
            let logs = m.values().filter(|b| is_log(**b)).count();
            println!("  {logs} logs, {} leaves", m.values().filter(|b| is_leaves(**b)).count());
            let mark = |a: Option<&BlockId>, _: bool| match a {
                Some(b) if is_log(*b) => '#',
                Some(_) => '*',
                _ => '.',
            };
            for y in (FOOT.y - 1..=top).rev() {
                let row: String = (-REACH..=REACH)
                    .map(|x| {
                        let col: Vec<_> = (-REACH..=REACH).filter_map(|z| m.get(&(x, y, z))).collect();
                        mark(col.first().copied(), col.iter().any(|b| is_log(**b)))
                    })
                    .collect();
                println!("{y:3} {row}");
            }
            for z in -REACH..=REACH {
                let row: String = (-REACH..=REACH)
                    .map(|x| {
                        let col: Vec<_> = (FOOT.y + 1..=top).rev().filter_map(|y| m.get(&(x, y, z))).collect();
                        mark(col.first().copied(), col.iter().any(|b| is_log(**b)))
                    })
                    .collect();
                println!("    {row}");
            }
        }
    }
}
/// What finding and growing the trees costs a column, against version 11
/// (`cargo test -q --release bench_tree_generation -- --ignored --nocapture`).
#[test]
#[ignore]
fn bench_tree_generation() {
    use crate::worldgen::WorldGen;
    use std::time::Instant;
    let cols: Vec<(i32, i32)> = (-6..6).flat_map(|z| (-6..6).map(move |x| (x * 32 - 160, z * 32 - 40))).collect();
    for version in [11, 12] {
        let g = WorldGen::with_version(1337, version);
        let start = Instant::now();
        let mut trees = Vec::new();
        for &(x, z) in &cols {
            trees.extend(g.trees_near(x, z));
        }
        let all = start.elapsed();
        let start = Instant::now();
        let mut cells = 0;
        for t in trees.iter().filter(|t| !t.grown.is_empty()) {
            grow(t.species, IVec3::new(t.x, t.ground, t.z), t.trunk, 1337, |_, _, _| cells += 1);
        }
        let grown = start.elapsed();
        let start = Instant::now();
        let mut full = WorldGen::with_version(1337, version);
        for &(x, z) in &cols {
            full.column(x / 32, z / 32);
        }
        let columns = start.elapsed();
        for &(x, z) in &cols {
            for cy in 1..5 {
                full.generate(IVec3::new(x / 32, cy, z / 32));
            }
        }
        println!(
            "version {version}: {} trees for {} columns: finding {:?} per column, growing {:?} of it; column {:?}, with 4 chunks {:?} ({cells} cells)",
            trees.len(),
            cols.len(),
            all / cols.len() as u32,
            grown / cols.len() as u32,
            columns / cols.len() as u32,
            start.elapsed() / cols.len() as u32
        );
    }
}
