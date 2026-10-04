//! Prospecting: scans match a brute-force search, core samples match the deposit figures, and
//! neither touches the core.

use super::*;
use crate::action::Action;
use crate::block;
use crate::deposits::{owner_of, DepositKey, Tier};
use crate::item::{ItemId, CORE_DRILL, SCANNER, SCANNER_MK2};
use crate::tests::{find_outcrop_block, run_until_ready};
use crate::worldgen::WorldGen;

/// Every deposit seeded anywhere near `at` whose centre is within `range`, the slow way.
fn brute_force(seed: u32, version: u32, at: IVec3, range: i32) -> Vec<DepositKey> {
    let g = WorldGen::with_version(seed, version);
    let mut all = Vec::new();
    let reach = range / 32 + 2;
    for cz in (at.z >> 5) - reach..=(at.z >> 5) + reach {
        for cx in (at.x >> 5) - reach..=(at.x >> 5) + reach {
            g.seed_deposits(cx, cz, &mut all);
        }
    }
    let mut keys: Vec<DepositKey> =
        all.iter().filter(|d| flat_dist2(d.center, at) <= range * range).map(|d| d.key).collect();
    sort_small_by_key(&mut keys, |k| *k);
    keys
}

#[test]
fn scans_list_exactly_the_deposits_in_range() {
    for (seed, version) in [(2024, 2), (1337, 2), (7, 1)] {
        let mut g = Game::with_generator(WorldGen::with_version(seed, version), 2);
        for at in [IVec3::new(0, 70, 0), IVec3::new(130, 80, -70), IVec3::new(-400, 60, 333)] {
            for (_, range, _) in SCANNERS {
                let found = g.scan(at, range);
                let mut keys: Vec<DepositKey> = found.iter().map(|f| f.deposit.key).collect();
                sort_small_by_key(&mut keys, |k| *k);
                assert_eq!(keys, brute_force(seed, version, at, range), "seed {seed} at {at:?} range {range}");
                assert!(keys.len() > 20, "a scan finds plenty");
                // Lodes first, then veins, then outcrops, each nearest first.
                let order: Vec<(Tier, i32)> =
                    found.iter().map(|f| (f.deposit.tier(), flat_dist2(f.deposit.center, at))).collect();
                assert!(order.windows(2).all(|w| w[0] <= w[1]), "sorted");
            }
        }
    }
}

#[test]
fn the_mk2_scanner_reaches_further_and_shows_quartz() {
    assert_eq!(scan_range_of(SCANNER), Some(48));
    assert_eq!(scan_range_of(SCANNER_MK2), Some(96));
    assert_eq!(scan_range_of(CORE_DRILL), None);
    assert_eq!(tools::device(SCANNER_MK2), Some(ToolKind::Scanner));
    let mut g = Game::with_generator(WorldGen::with_version(2024, 4), 2);
    let at = IVec3::new(0, 70, 0);
    let (near, far) = (g.scan(at, 48), g.scan(at, 96));
    assert!(far.len() > near.len() * 2, "four times the area finds many more: {} vs {}", far.len(), near.len());
    assert!(near.iter().all(|n| far.iter().any(|f| f.deposit.key == n.deposit.key)), "Mk2 finds all Mk1 does");
    // Quartz lies in highlands, deserts and basalt fields, not around the plains spawn: look further out.
    let quartz_seen = |g: &mut Game, range: i32| {
        (-6..=6)
            .flat_map(|i| (-6..=6).map(move |j| IVec3::new(i * 140, 70, j * 140)))
            .filter(|&at| g.scan(at, range).iter().any(|f| f.deposit.ore() == block::QUARTZ_ORE))
            .count()
    };
    let (mk1, mk2) = (quartz_seen(&mut g, 48), quartz_seen(&mut g, 96));
    assert!(mk1 > 0, "quartz deposits are listed by every scanner");
    assert!(mk2 > mk1, "the Mk2 finds quartz from more places: {mk2} vs {mk1}");
}

#[test]
fn the_mk2_filters_by_ore_and_reports_reserves() {
    assert!(is_advanced(SCANNER_MK2) && !is_advanced(SCANNER) && !is_advanced(CORE_DRILL));
    let mut g = Game::with_generator(WorldGen::with_version(2024, 4), 2);
    run_until_ready(&mut g);
    // The Mk1 has no filter: R does nothing for it.
    hold(&mut g, SCANNER);
    assert!(!g.rotate_target() && g.prospect.filter.is_none());
    hold(&mut g, SCANNER_MK2);
    g.using = true;
    g.run_tick();
    g.using = false;
    let all = g.prospect_records();
    assert_eq!(all.len() % SCAN_FIELDS, 0);
    assert!(!all.is_empty());
    // R steps through coal, iron, copper, limestone, quartz, bauxite and back to every ore.
    let mut seen = vec![g.scan_filter()];
    for _ in 0..FILTER_ORES.len() + 1 {
        assert!(g.rotate_target());
        seen.push(g.scan_filter());
        let kept = g.prospect_records();
        let ore = g.prospect.filter;
        assert!(kept.chunks_exact(SCAN_FIELDS).all(|r| ore.is_none_or(|o| r[0] == o as i32)));
        assert!(ore.is_some() || kept == all, "no filter keeps everything");
    }
    assert_eq!(seen, [0, 7, 8, 9, 33, 34, 76, 0]);
    // Reserves: a tracked deposit's exact units, an untouched one's estimate, and the minutes a full-speed mine
    // takes (units over the tier's draw cap).
    for r in all.chunks_exact(SCAN_FIELDS) {
        let tier = Tier::from_u8(r[1] as u8).unwrap();
        assert!(r[6] > 0, "a deposit that is listed holds ore");
        let minutes = r[6] as f64 / (tier.draw_cap() * 60.0);
        assert!(
            (r[7] as f64 - minutes).abs() <= 0.5,
            "{} units, {} minutes at {} a second",
            r[6],
            r[7],
            tier.draw_cap()
        );
    }
    // The estimate matches an actual survey of a vein to within a fifth.
    let vein = g.sim.world.generator().find_deposit(g.body().pos.floor(), Tier::Vein, 4).expect("a vein");
    let blocks = DepositState::survey(&mut g.sim.world, vein).initial_blocks as f64;
    let estimate = estimated_units(&vein) as f64 / vein.tier().grade() as f64;
    assert!((estimate / blocks - 1.0).abs() < 0.2, "estimated {estimate} blocks, found {blocks}");
}

/// Breaks every block of the deposit `key` by hand (actions, like a player would).
fn work_out(g: &mut Game, key: DepositKey) {
    let d = g.sim.world.generator().deposit_by_key(key).unwrap();
    let (lo, hi) = d.bounds();
    for y in (lo.y..=hi.y).rev() {
        for z in lo.z..=hi.z {
            for x in lo.x..=hi.x {
                let p = IVec3::new(x, y, z);
                if owner_of(&mut g.sim.world, p).is_some_and(|o| o.key == key) {
                    g.act(Action::BreakBlock { pos: p });
                    g.run_ticks(1);
                }
            }
        }
    }
}

#[test]
fn worked_out_deposits_drop_out_of_scans() {
    let mut g = Game::new(2024, 2);
    run_until_ready(&mut g);
    let (_, key) = find_outcrop_block(&mut g, 3);
    let at = g.body().pos.floor();
    assert!(g.scan(at, 48).iter().any(|f| f.deposit.key == key));
    work_out(&mut g, key);
    assert!(g.sim.factory.deposits.get(&key).unwrap().exhausted());
    assert!(!g.scan(at, 48).iter().any(|f| f.deposit.key == key), "a worked-out deposit is left out");
}

#[test]
fn core_samples_match_the_deposit_figures() {
    let mut g = Game::new(2024, 2);
    run_until_ready(&mut g);
    // A tracked outcrop with one block mined by hand.
    let (p, key) = find_outcrop_block(&mut g, 4);
    g.act(Action::BreakBlock { pos: p });
    g.run_ticks(1);
    let samples = g.core_sample(p + IVec3::new(0, 3, 0));
    let st = g.sim.factory.deposits.get(&key).unwrap();
    let s = samples.iter().find(|s| s.deposit.key == key).expect("the outcrop is in the sample");
    assert_eq!((s.remaining_blocks, s.initial_blocks), (st.remaining_blocks, st.initial_blocks));
    assert_eq!(s.remaining_blocks + 1, s.initial_blocks);
    assert_eq!(s.remaining_units, st.remaining_units() as u32);
    assert!(s.bottom <= p.y && p.y <= s.top);

    // A deep vein nobody has touched: the figures of a fresh survey, drilled from the surface.
    let vein = g.sim.world.generator().find_deposit(p, Tier::Vein, 4).expect("a vein");
    let top = IVec3::new(vein.center.x, g.sim.world.generator().height_at(vein.center.x, vein.center.z), vein.center.z);
    let samples = g.core_sample(top);
    let s = samples.iter().find(|s| s.deposit.key == vein.key).expect("the vein is in the sample");
    let survey = DepositState::survey(&mut g.sim.world, vein);
    assert!(g.sim.factory.deposits.get(&vein.key).is_none(), "drilling tracks nothing");
    assert_eq!((s.remaining_blocks, s.initial_blocks), (survey.initial_blocks, survey.initial_blocks));
    assert_eq!((s.bottom, s.top), survey.y_span());
    assert!(s.top < top.y, "the vein lies below the surface");
}

/// Gives the local player `item`, selects it, and looks straight down.
fn hold(g: &mut Game, item: ItemId) {
    g.act(Action::Give { item, count: 1 });
    g.run_ticks(1);
    let slot = g.inventory().slots.iter().position(|s| s.item == item).unwrap();
    g.act(Action::SelectSlot { slot: slot as u8 });
    g.run_ticks(1);
    g.set_look(0.0, -1.55);
}

#[test]
fn prospecting_leaves_the_core_alone() {
    for device in [SCANNER, SCANNER_MK2, CORE_DRILL] {
        let (mut a, mut b) = (Game::new(2024, 2), Game::new(2024, 2));
        for g in [&mut a, &mut b] {
            run_until_ready(g);
            hold(g, device);
        }
        a.using = true;
        for _ in 0..(4 * 60) {
            a.run_tick();
            b.run_tick();
            assert_eq!(a.sim.state_hash(), b.sim.state_hash(), "prospecting changed the core");
        }
        assert_eq!(a.sim.factory.deposits.tracked(), b.sim.factory.deposits.tracked());
        assert_eq!(b.prospect.seq, 0);
        if device != CORE_DRILL {
            // Once at once, then every two seconds while held.
            assert_eq!((a.prospect.kind, a.prospect.seq), (READING_SCAN, 2));
            assert_eq!(Some(a.prospect.range), scan_range_of(device));
            assert!(!a.prospect.records.is_empty());
        } else {
            // One sample after three seconds, which releases the button.
            assert_eq!((a.prospect.kind, a.prospect.seq, a.using), (READING_DRILL, 1, false));
            assert_eq!(a.prospect.origin, a.body().pos.floor() - IVec3::new(0, 1, 0));
        }
    }
}
