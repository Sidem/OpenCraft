//! Hover words: every mark shape is explained, only what the player knows is told, nearest first.

use std::time::Instant;

use super::*;
use crate::block::{self, BELT, COAL_ORE, IRON_ORE, POLE, SMELTER};
use crate::deposits::Tier;
use crate::factory::Kind;
use crate::math::IVec3;
use crate::minimap::marks::{MARK_COVER, MARK_DEPOSIT, MARK_LAUNCH, MARK_ORE, MARK_SITE, MARK_TOWN};
use crate::minimap::MARK_FIELDS;
use crate::settlements;
use crate::tests::{find_outcrop_block, run_until_ready};
use crate::Game;

/// Every `pub const MARK_*: i32` in `marks.rs`, read from its source: Rust cannot list constants, and a mark added
/// without words must fail here.
fn shape_constants() -> Vec<(String, i32)> {
    include_str!("../marks.rs")
        .lines()
        .filter_map(|l| {
            let (name, value) = l.strip_prefix("pub const MARK_")?.split_once(": i32 = ")?;
            Some((format!("MARK_{name}"), value.trim_end_matches(';').parse().ok()?))
        })
        .collect()
}

/// One subject of every variant, and of every shape a variant makes.
fn samples() -> Vec<Subject> {
    let mut all = vec![
        Subject::Deposit { ore: IRON_ORE, tier: Tier::Lode },
        Subject::Deposit { ore: IRON_ORE, tier: Tier::Vein },
        Subject::Ore(COAL_ORE),
        Subject::Machine { block: SMELTER, tier: 2 },
        Subject::Launch,
        Subject::Pod { launcher: false, channel: 0 },
        Subject::Pod { launcher: true, channel: 5 },
        Subject::Range { reach: 100 },
    ];
    all.extend((0..4).map(|job| Subject::Site { job }));
    all.extend((1..=3).map(|strength| Subject::Guess { ore: COAL_ORE, strength }));
    all.extend((0..satellites::KINDS.len() as u8).map(|kind| Subject::Cover { kind, reach: 500 }));
    all.extend(
        (0..settlements::KINDS.len() as u8)
            .flat_map(|kind| (0..=DORMANT).map(move |state| Subject::Town { kind, state })),
    );
    all
}

#[test]
fn every_mark_shape_has_words() {
    let constants = shape_constants();
    assert!(constants.len() >= 9, "the constants are read from the source: {constants:?}");
    let described: Vec<i32> = samples().iter().map(|s| s.shape()).collect();
    for (name, value) in &constants {
        assert!(described.contains(value), "{name} ({value}) has no Subject with words in describe.rs");
    }
    for s in samples() {
        let (title, detail) = s.words();
        assert!(!title.is_empty() && !detail.is_empty(), "{s:?}");
        assert!(![&title, &detail].iter().any(|t| t.contains(['\t', '\n'])), "{s:?} breaks the line format");
        let _ = s.colour();
    }
}

/// A game with one of everything the maps mark, around (40, 40).
fn busy() -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    while g.next_event() != 0 {}
    for _ in 0..200 {
        g.minimap.atlas.refresh_some(&g.sim.world);
    }
    let mut world = crate::world::World::new(1, 2);
    let at = g.body().pos.floor();
    let f = &mut g.sim.factory;
    for (block, dx, facing) in
        [(SMELTER, 10, 0), (block::LAUNCH_SITE, 20, 0), (block::LANDING_PAD, 30, 1), (block::POD_LAUNCHER, 40, 2)]
    {
        let pos = IVec3::new(at.x + dx, 100, at.z + 12);
        f.place(&mut world, block, pos, facing, pos - IVec3::new(0, 1, 0), 0);
    }
    let depot = IVec3::new(at.x - 30, 100, at.z + 12);
    f.place(&mut world, block::TOWN_DEPOT, depot, 0, depot - IVec3::new(0, 1, 0), 0);
    g.sim.settlements.towns.push(settlements::Town::new(depot, settlements::HARBOUR));
    g.mark_site(at.x + 5, at.z - 20, at.x + 9, at.z - 18, 60, 1);
    g.launch_satellite_at(at.x, at.z);
    g.run_ticks(2);
    for _ in 0..40 {
        g.minimap_redraw();
    }
    g
}

#[test]
fn every_mark_the_maps_draw_is_described_where_it_is_drawn() {
    let g = busy();
    let at = g.body().pos.floor();
    let marks = g.world_map_marks(at.x - 900, at.z - 900, at.x + 900, at.z + 900);
    let mut shapes = std::collections::BTreeSet::new();
    for m in marks.chunks_exact(MARK_FIELDS) {
        let (x, z, colour, shape) = (m[0], m[1], m[2], m[3]);
        // A footprint is the ring, `colour` blocks from its centre.
        let (x, z) = if (MARK_COVER..MARK_COVER + 4).contains(&shape) { (x + colour, z) } else { (x, z) };
        let lines = g.world_map_describe(x, z, 2);
        let want = format!("{shape}\t{colour}\t");
        assert!(lines.lines().any(|l| l.starts_with(&want)), "mark {m:?} at {x}, {z} is not described:\n{lines}");
        shapes.insert(shape);
    }
    for shape in [MARK_DEPOSIT, MARK_ORE, MARK_SITE, MARK_COVER, MARK_LAUNCH, MARK_TOWN, 1, 16, 17] {
        assert!(shapes.contains(&shape), "the busy game shows shape {shape}: {shapes:?}");
    }
}

#[test]
fn what_the_player_has_not_learned_is_not_told_and_no_reserve_is_named() {
    let mut g = busy();
    // An unprospected deposit the satellite did not see, far from it: its centre has no mark and no words.
    let (_, key) = find_outcrop_block(&mut g, 3);
    let d = g.sim.world.generator().deposit_by_key(key).unwrap();
    let lines = g.world_map_describe(d.center.x, d.center.z, 3);
    assert!(!lines.lines().any(|l| l.starts_with("0\t")), "an outcrop nobody prospected: {lines}");
    // Known deposits are named by ore and tier only.
    let at = g.body().pos.floor();
    let marks = g.world_map_marks(at.x - 700, at.z - 700, at.x + 700, at.z + 700);
    let deposit = marks.chunks_exact(MARK_FIELDS).find(|m| m[3] == MARK_DEPOSIT).expect("seen from orbit");
    let told = g.world_map_describe(deposit[0], deposit[1], 1);
    let line = told.lines().find(|l| l.starts_with("0\t")).expect("described");
    let words = line.split('\t').skip(2).collect::<Vec<_>>().join(" ");
    assert!(!words.contains(|c: char| c.is_ascii_digit()), "{words}");
    assert!(words.contains("vein") || words.contains("lode"), "{words}");
}

#[test]
fn the_nearest_come_first_within_reach_and_a_footprint_counts_by_its_ring() {
    let mut g = Game::new(7, 2);
    let mut world = crate::world::World::new(1, 2);
    for x in [10, 12, 20] {
        let pos = IVec3::new(x, 70, 0);
        g.sim.factory.place(&mut world, SMELTER, pos, 0, pos - IVec3::new(0, 1, 0), 0);
    }
    let count = |lines: &str| lines.lines().count();
    let near = g.world_map_describe(11, 0, 5);
    assert_eq!(count(&near), 2, "the smelters at 10 and 12, not 20:\n{near}");
    assert_eq!(count(&g.world_map_describe(10, 0, 1)), 1);
    assert_eq!(count(&g.world_map_describe(10, 0, 30)), 3);
    assert!(g.world_map_describe(10, 0, 30).lines().next().unwrap().starts_with("1\t"), "nearest first");
    assert_eq!(count(&g.world_map_describe(500, 500, 30)), 0);

    g.sim.factory.orbit.launch(crate::item::SURVEY_SATELLITE, IVec3::new(0, 70, 0), 0);
    let reach = crate::satellites::KINDS[0].reach;
    let on_ring = g.world_map_describe(reach, 0, 3);
    assert!(on_ring.contains("Survey satellite cover"), "{on_ring}");
    assert!(!g.world_map_describe(0, 0, 3).contains("cover"), "its middle is not marked");
}

#[test]
#[ignore]
fn bench_world_map_describe() {
    let mut g = Game::new(2024, 3);
    let mut world = crate::world::World::new(1, 2);
    g.sim.factory.by_hand = true;
    let (rows, per_row, y) = (100, 300, 100);
    let f = &mut g.sim.factory;
    for z in 0..rows {
        for x in 0..per_row {
            let pos = IVec3::new(x, y, z * 4);
            f.place(&mut world, BELT, pos, 1, pos - IVec3::new(0, 1, 0), 0);
        }
        for x in (0..per_row).step_by(10) {
            let pos = IVec3::new(x + 3, y, z * 4 + 2);
            f.place(&mut world, SMELTER, pos, 0, pos - IVec3::new(0, 1, 0), 0);
        }
    }
    for x in (0..per_row).step_by(10) {
        for z in (0..rows).step_by(3) {
            let pos = IVec3::new(x + 5, y, z * 4 + 1);
            f.place(&mut world, POLE, pos, 0, pos - IVec3::new(0, 1, 0), 3);
        }
    }
    for k in 0..4u64 {
        for i in 0..crate::satellites::MAX_PER_KIND as u64 {
            let item = crate::satellites::KINDS[k as usize].item;
            f.orbit.launch(item, IVec3::new(100 + 20 * i as i32, y, 200), i);
        }
    }
    for i in 0..30 {
        g.sim.settlements.towns.push(settlements::Town::new(IVec3::new(i * 20, y, 150), (i % 4) as u8));
    }
    let towns = &g.sim.settlements.towns;
    // The widest reach the hover uses (a far-out map, 16 blocks a pixel, 14 px), with the long-pole rings on, over the base.
    let (mut total, mut worst, n) = (0u128, 0u128, 2000);
    let mut lines = 0;
    for i in 0..n {
        let at = ((i * 37) % per_row, (i * 91) % (rows * 4));
        let start = Instant::now();
        lines += g.minimap.describe(&g.sim.factory, towns, at, 224, true).len();
        let us = start.elapsed().as_micros();
        (total, worst) = (total + us, worst.max(us));
    }
    let (processors, belts) = (g.sim.factory.count(Kind::Process), g.sim.factory.count(Kind::Belt));
    println!(
        "describe on {processors} processors and {belts} belts: {} µs a query, {worst} µs worst ({lines} bytes)",
        total / n as u128
    );
    if !cfg!(debug_assertions) {
        assert!(total / (n as u128) < 200, "the query takes under 0.2 ms");
    }
}
