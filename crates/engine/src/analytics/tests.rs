use super::power::TOTALS;
use super::series::{Series, COARSE_STEP};
use super::*;
use crate::block::{BlockId, COAL_ORE, CONSTRUCTOR, IRON_ORE, LAB, MINER, SLAG, SMELTER};
use crate::factory::efficiency::CAUSES;
use crate::factory::Factory;
use crate::item::{DIESEL_CANISTER, IRON_INGOT, IRON_PLATE};
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
fn production_and_use_become_rates_a_minute() {
    let mut a = Analytics::default();
    let f = Factory::default();
    for tick in 0..60 * 12 {
        if tick % 60 == 0 {
            a.produced(IRON_INGOT, 10);
            a.consumed(IRON_INGOT, 4);
        }
        a.record(&f);
    }
    assert_eq!(a.item_ids(), vec![IRON_INGOT.0 as u32]);
    // Ten made and four used a second for ten seconds on: 600 and 240 a minute, 120 and 48 in all.
    assert_eq!(a.item_now(), vec![600.0, 240.0, 120.0, 48.0]);
    let data = a.export_items(60, 60);
    assert_eq!((data[0], data[1]), (60.0, 2.0), "a row for made and a row for used");
    assert_eq!(data.len(), 2 + 60 * 2);
    assert_eq!(data[2 + 59], 600.0);
    assert_eq!(data[2 + 60 + 59], 240.0);
}

#[test]
fn a_late_item_reads_blank_before_the_world_was_opened_and_zero_before_it_appeared() {
    let mut a = Analytics::default();
    let f = Factory::default();
    for _ in 0..60 * 30 {
        a.record(&f);
    }
    a.consumed(COAL_ORE.into(), 1);
    for _ in 0..60 * 5 {
        a.record(&f);
    }
    let data = a.export_items(60, 60);
    let used = &data[2 + 60..];
    assert!(used[..25].iter().all(|v| v.is_nan()), "before the world was opened");
    assert_eq!(used[30], 0.0);
}

#[test]
fn items_sort_into_material_groups() {
    use crate::item::{COPPER_WIRE, CRUSHED_IRON, GREEN_PACK, MOTOR, SILICON};
    let group = |item: ItemId| GROUPS[groups::group_of(item) as usize].name;
    assert_eq!(group(IRON_ORE.into()), "Raw materials");
    assert_eq!(group(COAL_ORE.into()), "Raw materials");
    assert_eq!(group(IRON_INGOT), "Ingots and bricks");
    assert_eq!(group(CRUSHED_IRON), "Ore processing");
    assert_eq!(group(COPPER_WIRE), "Basic parts");
    assert_eq!(group(MOTOR), "Components and kits");
    assert_eq!(group(GREEN_PACK), "Science packs");
    assert_eq!(group(SILICON), "Electronics");
    assert_eq!(group(DIESEL_CANISTER), "Chemicals and fuel");
    assert_eq!(group(SLAG.into()), "Raw materials", "a block the crusher gives back is bulk, not processed ore");
}

#[test]
fn a_smelter_at_full_pace_reads_100_and_reports_what_it_made_and_used() {
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
    let count = |want_made: bool, which: ItemId| -> u32 {
        events
            .iter()
            .filter_map(|e| match e {
                SimEvent::Produced { item, count } if want_made && *item == which => Some(*count),
                SimEvent::Consumed { item, count } if !want_made && *item == which => Some(*count),
                _ => None,
            })
            .sum()
    };
    assert_eq!(count(true, IRON_INGOT), 3);
    assert_eq!(count(false, IRON_ORE.into()), 3, "each ore used up as its batch starts");
    assert!(count(false, COAL_ORE.into()) >= 1, "the fire burned coal");
    // Out of ore, it now waits: its average shows what that cost.
    let line = a.machines.line(pos);
    assert!(line.contains("waiting for input"), "{line}");

    // By kind: one smelter working, short of input.
    let types = a.machines.by_type();
    let row = &types[..TYPE_FIELDS];
    assert_eq!((row[0], row[1], row[2]), (SMELTER as u32, 1, 0));
    assert_eq!(row[5 + 1], 1, "counted under waiting for input");
    assert_eq!(&a.machines.worst(5)[..4], &[0, 0, 0, SMELTER as i32]);
}

/// A row of machines: smelters that run and then wait for ore, constructors with a recipe and nothing to make it
/// from, and one constructor whose recipe is taken away after it has worked (idle).
fn yard() -> (Factory, Analytics, IVec3) {
    let (mut f, mut world) = (Factory::default(), World::new(1, 2));
    let recipe = crate::recipes::MACHINE_RECIPES.iter().position(|r| r.main().0 == IRON_PLATE).unwrap() as u16;
    let at = |i: i32| IVec3::new(i * 4, 0, 0);
    for i in 0..3 {
        f.place(&mut world, SMELTER, at(i), 0, at(i) - IVec3::new(0, 1, 0), 0);
        f.insert(at(i), IRON_ORE.into(), 2 + i as u32);
        f.insert(at(i), COAL_ORE.into(), 3);
    }
    for i in 3..6 {
        f.place(&mut world, CONSTRUCTOR, at(i), 0, at(i) - IVec3::new(0, 1, 0), 0);
        assert!(f.set_recipe(at(i), Some(recipe)).is_some(), "a constructor makes plates");
    }
    let (mut a, mut events) = (Analytics::default(), Vec::new());
    for tick in 0..60 * 20 {
        if tick == 60 * 5 {
            drop(f.set_recipe(at(5), None));
        }
        f.update(&mut world, tick, &mut events);
        a.record(&f);
    }
    (f, a, at(5))
}

#[test]
fn every_kind_lists_the_machines_its_row_counts_where_the_map_marks_them() {
    let (f, a, idle) = yard();
    let mut marked = Vec::new();
    f.map_machines((i32::MIN, i32::MIN), (i32::MAX, i32::MAX), &mut |b, p| marked.push((b, p)));
    let types = a.machines.by_type();
    assert!(types.len() >= 2 * TYPE_FIELDS, "smelters and constructors both counted");
    for row in types.chunks(TYPE_FIELDS) {
        let block = row[0] as BlockId;
        let list = a.machines.of_kind(block);
        let machines: Vec<&[i32]> = list.chunks(KIND_FIELDS).collect();
        assert_eq!(machines.len() as u32, row[1] + row[4], "every machine the row counts, block {block}");
        assert_eq!(machines.iter().filter(|m| m[3] < 0).count() as u32, row[4], "idle ones");
        assert_eq!(machines.iter().filter(|m| m[3] >= 0 && m[4] < 0).count() as u32, row[2], "full-speed ones");
        for c in 0..CAUSES {
            assert_eq!(machines.iter().filter(|m| m[4] == c as i32).count() as u32, row[5 + c], "cause {c}");
        }
        for m in &machines {
            let p = IVec3::new(m[0], m[1], m[2]);
            assert!(marked.contains(&(block, p)), "{p:?} is a {block} mark on the map");
        }
    }
    let constructors = a.machines.of_kind(CONSTRUCTOR);
    let gone = constructors.chunks(KIND_FIELDS).find(|m| IVec3::new(m[0], m[1], m[2]) == idle).expect("listed");
    assert_eq!((gone[3], gone[4]), (-1, -1), "the constructor without a recipe reads idle");
    assert!(a.machines.of_kind(LAB).is_empty());
}

/// `cargo test --release -q bench_machines_of_kind -- --ignored --nocapture`: one kind picked out of 3,025 machines.
#[test]
#[ignore]
fn bench_machines_of_kind() {
    let (mut f, mut world) = (Factory::default(), World::new(1, 2));
    for x in 0..55 {
        for z in 0..55 {
            let pos = IVec3::new(x * 3, 100, z * 3);
            f.place(
                &mut world,
                if (x + z) % 2 == 0 { SMELTER } else { CONSTRUCTOR },
                pos,
                0,
                pos - IVec3::new(0, 1, 0),
                0,
            );
        }
    }
    let mut a = Analytics::default();
    a.machines.sample(&f);
    let start = std::time::Instant::now();
    let mut n = 0;
    for _ in 0..1000 {
        n += a.machines.of_kind(SMELTER).len() / KIND_FIELDS;
    }
    println!("of_kind: {} smelters of {} machines in {} µs", n / 1000, 55 * 55, start.elapsed().as_micros() / 1000);
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
    let now = g.analytics.power_now();
    assert!(now[1] > 0.0 && now[0] >= now[1], "capacity {} must cover use {}", now[0], now[1]);
    let items = g.analytics.item_now();
    assert!(items.chunks(4).any(|c| c[0] > 0.0), "the miner's ore has a graph row and was made");
    assert!(items.chunks(4).any(|c| c[1] > 0.0 || c[3] > 0.0), "the generator burned some fuel");
    assert!(g.analytics.machines.summary()[0] >= 1);

    // The generator supplied it all, and the miner is what asked for it.
    let sources = TOTALS..TOTALS + SOURCE_LIST.len();
    assert!(now[TOTALS] > 0.0, "the generator gave power");
    assert!((now[1] - now[sources].iter().sum::<f32>()).abs() < 0.5, "the sources add up to the supply");
    let users = g.analytics.power_users();
    assert_eq!(users, vec![MINER as u32]);
    assert!((now[2] - now[TOTALS + SOURCE_LIST.len()]).abs() < 0.5, "the miner asked for all of it");
    assert!(g.analytics.power_could()[0] > 0);

    // Take the generator away: the average falls and names the power.
    g.sim.factory.remove(m + IVec3::new(0, 3, 0));
    g.skip_time(40.0);
    let line = g.efficiency_line(m);
    assert!(line.contains("low power"), "{line}");
    assert!(g.machine_efficiency_percent(m.x, m.y, m.z) < 20, "{line}");
}
