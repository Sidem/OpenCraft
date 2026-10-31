//! Minimap marks: prospected veins and lodes and machines show where they are, worked-out deposits and their miners
//! are marked so,
//! and the remembered list survives an export and import.

use super::*;
use crate::action::Action;
use crate::block::{BELT, IRON_ORE, SMELTER};
use crate::deposits::owner_of;
use crate::item::SCANNER;
use crate::tests::{build_mine, find_outcrop_block, run_until_ready};
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
fn a_survey_satellite_marks_its_footprint_and_the_deposits_under_it() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g.minimap_redraw();
    assert!(marks_of(&g, MARK_COVER).is_empty() && marks_of(&g, MARK_DEPOSIT).is_empty(), "nothing before a launch");
    let at = g.body().pos.floor();
    g.launch_satellite_at(at.x, at.z);
    g.run_ticks(1);
    for _ in 0..40 {
        g.minimap_redraw();
    }
    // The player stands under it: the ring is centred on the image, its reach (in blocks) in the colour field.
    let rings = marks_of(&g, MARK_COVER);
    assert_eq!(rings.len(), 1);
    assert!(rings[0].0.abs() <= 1 && rings[0].1.abs() <= 1, "{rings:?}");
    assert_eq!(rings[0].2, crate::satellites::KINDS[0].reach);
    // The whole world map's window shows many deposits that nobody prospected.
    let (x, z) = (at.x, at.z);
    let marks = g.world_map_marks(x - 700, z - 700, x + 700, z + 700);
    let deposits = marks.chunks_exact(MARK_FIELDS).filter(|r| r[3] == MARK_DEPOSIT).count();
    assert!(deposits > 20 && g.minimap.known.deposits.is_empty(), "{deposits} seen from orbit");
}

#[test]
fn a_worked_out_deposit_and_its_miner_are_marked_so() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, key) = find_outcrop_block(&mut g, 3);
    let (miner, _) = build_mine(&mut g, p);
    g.run_ticks(1);
    g.minimap_redraw();
    assert_eq!(marks_of(&g, MARK_DEPOSIT).len(), 1, "the deposit under a miner is marked, scanned or not");
    assert!(marks_of(&g, MARK_DRY).is_empty() && marks_of(&g, MARK_DRY_MINER).is_empty());
    let d = g.sim.world.generator().deposit_by_key(key).unwrap();
    g.minimap.known.deposits.push(d); // outcrops aren't remembered; this one stands in for a vein
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
    g.run_ticks(2);
    g.minimap_redraw();
    assert!(marks_of(&g, MARK_DEPOSIT).is_empty(), "no longer a deposit with ore");
    assert_eq!(marks_of(&g, MARK_DRY).len(), 1, "but marked worked out");
    let at = g.minimap.centre.unwrap();
    let stopped = marks_of(&g, MARK_DRY_MINER);
    assert_eq!(
        stopped.iter().map(|m| (m.0 + at.0, m.1 + at.1)).collect::<Vec<_>>(),
        [(miner.x, miner.z)],
        "the miner too"
    );
    assert_eq!(g.known_deposits().len(), KNOWN_FIELDS, "kept with the world's record, so the mark stays");
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
        g.sim.factory.place(&mut g.sim.world, block, p, 0, p, 0);
    }
    assert!(g.minimap_marks().is_empty(), "nothing before the first redraw");
    g.minimap_redraw();
    assert_eq!(marks_of(&g, MARK_MACHINE), vec![(10, -20, machine_color(SMELTER))]);
}

#[test]
fn a_site_shows_as_one_mark_at_its_centre_in_its_job_colour() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g.minimap_redraw();
    let at = g.body().pos.floor();
    let (x, z) = (at.x + 10, at.z + 6);
    g.mark_site(x, z, x + 4, z + 2, 60, 1);
    g.mark_tunnel(&[x - 30, 50, z, x - 20, 50, z], 0);
    g.run_ticks(2);
    g.minimap_redraw();
    let sites = marks_of(&g, MARK_SITE);
    assert_eq!(sites.len(), 2);
    assert!(sites.contains(&(10 + 2, 6 + 1, crate::site_hands::SITE_COLOURS[1])), "the fill, at its middle: {sites:?}");
    assert!(sites.contains(&(-15, 6, crate::site_hands::SITE_COLOURS[3])), "the tunnel, at its middle: {sites:?}");
}
