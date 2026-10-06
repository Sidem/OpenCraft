use super::series::{Series, COARSE_STEP};
use super::*;
use crate::block::{COAL_ORE, IRON_ORE, SMELTER};
use crate::factory::Factory;
use crate::item::IRON_INGOT;
use crate::math::IVec3;
use crate::sim::SimEvent;
use crate::tests::{build_mine, find_outcrop_block, run_until_ready};
use crate::world::World;
use crate::Game;

#[test]
fn a_window_averages_the_newest_values_and_is_blank_before_the_history() {
    let mut s = Series::default();
    (1..=100).for_each(|v| s.push(v as f32));
    assert_eq!(s.latest(), 100.0);

    let mut out = Vec::new();
    assert_eq!(s.window(60, 60, &mut out), 60);
    assert_eq!((out[0], out[59]), (41.0, 100.0));

    // Fewer points than values: each point is the mean of its slice (two values here).
    out.clear();
    assert_eq!(s.window(60, 30, &mut out), 30);
    assert_eq!((out[0], out[29]), (41.5, 99.5));

    // A window longer than the history starts blank, and never has more points than values.
    out.clear();
    assert_eq!(s.window(200, 500, &mut out), 200);
    assert!(out[..100].iter().all(|v| v.is_nan()));
    assert_eq!((out[100], out[199]), (1.0, 100.0));
}

#[test]
fn three_hours_come_from_the_coarse_history() {
    let mut s = Series::default();
    for v in 0..5000 {
        s.push((v / COARSE_STEP) as f32);
    }
    let mut out = Vec::new();
    assert_eq!(s.window(10_800, 4000, &mut out), 1080);
    assert_eq!(*out.last().unwrap(), 499.0, "the newest coarse value is the last full ten seconds");
    assert!(out[0].is_nan(), "the first values are before the history");
}

#[test]
fn a_late_series_reads_zero_before_it_began() {
    let mut s = Series::after(30);
    s.push(5.0);
    let mut out = Vec::new();
    s.window(60, 60, &mut out);
    assert!(out[..29].iter().all(|v| v.is_nan()));
    assert_eq!((out[29], out[58], out[59]), (0.0, 0.0, 5.0));
}

#[test]
fn production_becomes_a_rate_a_minute() {
    let mut a = Analytics::default();
    let f = Factory::default();
    for tick in 0..60 * 12 {
        if tick % 60 == 0 {
            a.produced(IRON_INGOT, 10);
        }
        a.record(&f);
    }
    assert_eq!(a.item_ids(), vec![IRON_INGOT.0 as u32]);
    // Ten a second for ten seconds on: 600 a minute.
    assert_eq!(a.latest()[POWER_ROWS], 600.0);
    let data = a.export(60, 60);
    assert_eq!((data[0], data[1]), (60.0, (POWER_ROWS + 1) as f32));
    assert_eq!(data.len(), 2 + 60 * (POWER_ROWS + 1));
}

#[test]
fn a_smelter_at_full_pace_reads_100_and_reports_what_it_made() {
    let mut f = Factory::default();
    let mut world = World::new(1, 2);
    let pos = IVec3::ZERO;
    f.place(&mut world, SMELTER, pos, 0, pos - IVec3::new(0, 1, 0), 0);
    assert_eq!(f.insert(pos, IRON_ORE.into(), 3), 3);
    assert_eq!(f.insert(pos, COAL_ORE.into(), 3), 3);

    let (mut a, mut events) = (Analytics::default(), Vec::new());
    for tick in 0..60 * 20 {
        f.update(&mut world, tick, &mut events);
        a.record(&f);
    }
    let made: u32 = events
        .iter()
        .filter_map(|e| match e {
            SimEvent::Produced { item, count } if *item == IRON_INGOT => Some(*count),
            _ => None,
        })
        .sum();
    assert_eq!(made, 3);
    // Out of ore, it now waits: its average shows what that cost.
    let line = a.machines.line(pos);
    assert!(line.contains("waiting for input"), "{line}");
}

#[test]
fn a_miner_without_power_reads_low_power_and_one_with_it_reads_full() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, _) = find_outcrop_block(&mut g, 6);
    let (m, _) = build_mine(&mut g, p);
    g.skip_time(20.0);

    let line = g.efficiency_line(m);
    assert!(line.starts_with("Efficiency"), "{line}");
    assert!(g.machine_efficiency_percent(m.x, m.y, m.z) >= 90, "{line}");
    let now = g.analytics.latest();
    assert!(now[1] > 0.0 && now[0] >= now[1], "capacity {} must cover use {}", now[0], now[1]);
    assert!(g.analytics.item_ids().len() == 1, "the miner's ore has a graph row");
    assert!(g.analytics.machines.summary()[0] >= 1);

    // Take the generator away: the average falls and names the power.
    g.sim.factory.remove(m + IVec3::new(0, 3, 0));
    g.skip_time(40.0);
    let line = g.efficiency_line(m);
    assert!(line.contains("low power"), "{line}");
    assert!(g.machine_efficiency_percent(m.x, m.y, m.z) < 20, "{line}");
}
