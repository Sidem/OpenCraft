//! Minimap marks: prospected veins and lodes and machines show where they are, dry deposits drop out,
//! and the remembered list survives an export and import.

use super::*;
use crate::action::Action;
use crate::block::{BELT, IRON_ORE, SMELTER};
use crate::deposits::owner_of;
use crate::item::SCANNER;
use crate::tests::{find_outcrop_block, run_until_ready};
use crate::Game;

/// The marks of `shape` as (dx, dz, colour).
fn marks_of(g: &Game, shape: i32) -> Vec<(i32, i32, i32)> {
    let m = g.minimap_marks();
    m.chunks_exact(MARK_FIELDS).filter(|r| r[3] == shape).map(|r| (r[0], r[1], r[2])).collect()
}

fn scanned(seed: u32) -> Game {
    let mut g = Game::new(seed, 3);
    run_until_ready(&mut g);
    g.act(Action::Give { item: SCANNER, count: 1 });
    g.run_ticks(1);
    let slot = g.inventory().slots.iter().position(|s| s.item == SCANNER).unwrap();
    g.act(Action::SelectSlot { slot: slot as u8 });
    g.run_ticks(1);
    g.using = true;
    g.run_ticks(1);
    g.using = false;
    assert_eq!(g.prospect.seq, 1, "one scan");
    g.minimap_redraw();
    g
}

#[test]
fn a_scan_marks_its_veins_and_lodes_where_they_lie() {
    let g = scanned(2024);
    let at = g.body().pos.floor();
    let found: Vec<Deposit> = g.minimap.known.deposits.clone();
    assert!(found.len() >= 3, "a scan finds some veins or lodes");
    assert!(found.iter().all(|d| d.tier() != Tier::Outcrop), "outcrops show on the map by themselves");
    // Scan range (48) lies inside the image (64 each way), so every one is marked, at its centre.
    let marks = marks_of(&g, MARK_DEPOSIT);
    assert_eq!(marks.len(), found.len());
    for d in &found {
        let want = (d.center.x - at.x, d.center.z - at.z, ore_color(d.ore()));
        assert!(marks.contains(&want), "{d:?} marked");
    }
    // A second scan in the same place remembers nothing new.
    let mut g = g;
    g.run_ticks(150);
    g.using = true;
    g.run_ticks(1);
    assert_eq!(g.prospect.seq, 2);
    assert_eq!(g.minimap.known.deposits.len(), found.len());
}

#[test]
fn a_deposit_that_runs_dry_loses_its_mark() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (_, key) = find_outcrop_block(&mut g, 3);
    let d = g.sim.world.generator().deposit_by_key(key).unwrap();
    g.minimap.known.deposits.push(d); // outcrops aren't remembered; this one stands in for a vein
    g.minimap_redraw();
    assert_eq!(marks_of(&g, MARK_DEPOSIT).len(), 1);
    assert_eq!(g.known_deposits().len(), KNOWN_FIELDS);

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
    assert!(g.sim.factory.deposits.get(&key).unwrap().exhausted());
    assert!(marks_of(&g, MARK_DEPOSIT).is_empty(), "dry: no mark");
    assert!(g.known_deposits().is_empty(), "and not kept");
}

#[test]
fn remembered_deposits_survive_an_export_and_import() {
    let g = scanned(1337);
    let saved = g.known_deposits();
    assert_eq!(saved.len(), g.minimap.known.deposits.len() * KNOWN_FIELDS);

    let mut other = Game::new(1337, 2);
    let mut data = saved.clone();
    data.extend_from_slice(&[9, 0, 0, 8, 0]); // no such tier
    data.extend_from_slice(&[1, 0, 0, 8, 999]); // no such deposit
    data.extend_from_slice(&[1, 0]); // cut short
    other.set_known_deposits(&data);
    assert_eq!(other.known_deposits(), saved);
    let keys = |g: &Game| g.minimap.known.deposits.iter().map(|d| (d.key, d.center)).collect::<Vec<_>>();
    assert_eq!(keys(&other), keys(&g));
}

#[test]
fn the_oldest_deposit_is_forgotten_when_the_list_is_full() {
    let mut known = Known::default();
    let vein = |i: i32| Deposit {
        key: DepositKey { tier: Tier::Vein, cx: i, cz: 0, ore: IRON_ORE, index: 0 },
        center: IVec3::new(i * 32, 40, 0),
        radii: [3.0, 3.0, 3.0],
        seed: 0,
    };
    for i in 0..=MAX_KNOWN as i32 {
        known.remember(&vein(i));
    }
    assert_eq!(known.deposits.len(), MAX_KNOWN);
    assert_eq!(known.deposits[0].key.cx, 1);
}

#[test]
fn machines_are_marked_but_belts_are_not() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let c = g.body().pos.floor();
    let (near, far) = (c + IVec3::new(10, 0, -20), c + IVec3::new(70, 0, 0));
    for (block, p) in [(SMELTER, near), (SMELTER, far), (BELT, c + IVec3::new(3, 0, 3))] {
        g.sim.factory.place(&mut g.sim.world, block, p, 0, p);
    }
    assert!(g.minimap_marks().is_empty(), "nothing before the first redraw");
    g.minimap_redraw();
    assert_eq!(marks_of(&g, MARK_MACHINE), vec![(10, -20, machine_color(Kind::Smelter))]);
}
