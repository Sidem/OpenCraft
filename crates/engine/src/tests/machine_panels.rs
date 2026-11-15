//! What a machine's panel says about speed: recipe times follow the machine's Mk, every recipe machine reads out items a
//! minute and a second, and a miner opens a panel with its workings instead of being emptied by a right-click.

use super::{build_mine, find_outcrop_block, run_until_ready};
use crate::block::{CONSTRUCTOR, MINER, MINER_MK2, SMELTER};
use crate::factory;
use crate::item::{IRON_INGOT, IRON_ROD};
use crate::math::IVec3;
use crate::recipes::MACHINE_RECIPES;
use crate::Game;

fn recipe_of(item: crate::item::ItemId) -> u16 {
    MACHINE_RECIPES.iter().position(|r| r.main().0 == item).unwrap() as u16
}

#[test]
fn a_higher_mk_runs_the_same_recipe_in_less_time() {
    let mut g = Game::new(2024, 3);
    let (mk1, mk3) = (IVec3::new(500, 200, -500), IVec3::new(504, 200, -500));
    g.sim.factory.place(&mut g.sim.world, SMELTER, mk1, 0, mk1, 0);
    g.sim.factory.place(&mut g.sim.world, SMELTER, mk3, 0, mk3, 2);
    let ingot = u32::from(recipe_of(IRON_INGOT));
    let base = g.machine_recipe_seconds(ingot);
    let slow = g.machine_recipe_seconds_at(mk1.x, mk1.y, mk1.z, ingot);
    let fast = g.machine_recipe_seconds_at(mk3.x, mk3.y, mk3.z, ingot);
    assert!((slow - base).abs() < 0.02, "a Mk1 smelter takes the listed {base} s, not {slow}");
    assert!(fast < slow * 0.75, "a Mk3 smelter takes {fast} s against {slow} s");
    assert_eq!(g.machine_recipe_seconds_at(0, 0, 0, ingot), base, "no machine there: the listed time");
}

#[test]
fn a_recipe_machine_reads_out_a_minute_and_a_second() {
    let mut g = Game::new(2024, 3);
    let at = IVec3::new(500, 200, -500);
    g.sim.factory.place(&mut g.sim.world, CONSTRUCTOR, at, 0, at, 0);
    assert!(g.sim.factory.set_recipe(at, Some(recipe_of(IRON_ROD))).is_some());
    // One row per ingredient (0) and product (1), in tenths of an item a minute: the table the panel draws.
    let rows = g.machine_rate_rows(at.x, at.y, at.z);
    let rod = u32::from(IRON_ROD.0);
    assert_eq!(rows.len() % 3, 0);
    let (ins, outs): (Vec<_>, Vec<_>) = rows.chunks(3).partition(|r| r[0] == 0);
    assert_eq!((ins.len(), outs.len()), (1, 1), "an ingot in, a rod out: {rows:?}");
    assert_eq!((outs[0][1], ins[0][1]), (rod, u32::from(IRON_INGOT.0)));
    assert!(outs[0][2] > 0 && ins[0][2] > 0, "{rows:?}");
    assert_eq!(g.machine_rate_rows(0, 0, 0), Vec::<u32>::new(), "nothing there");
}

#[test]
fn a_miner_opens_a_panel_with_its_workings_and_keeps_its_ore_until_taken() {
    assert!(factory::machine(MINER).is_some_and(|m| m.panel), "right-click opens it, as it does a smelter");
    assert!(factory::machine(MINER_MK2).is_some_and(|m| m.panel));
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, _) = find_outcrop_block(&mut g, 2);
    let (m, _) = build_mine(&mut g, p);
    g.skip_time(20.0);
    let panel = g.machine_panel(m.x, m.y, m.z);
    assert_eq!(panel.len(), 6 + 3, "an output slot and nothing to put in: {panel:?}");
    let rows = g.machine_rate_rows(m.x, m.y, m.z);
    assert_eq!((rows[0], rows[2]), (1, 360), "one row: a Mk1 makes 36 ore a minute, 360 tenths: {rows:?}");
    let notes = g.machine_rate_notes(m.x, m.y, m.z);
    assert!(notes.contains("Keeps 60%") && notes.contains("blocks left"), "the deposit is on the panel: {notes}");
    assert!(!g.machine_status(m.x, m.y, m.z).is_empty());
    // Its output sits in the panel's only slot; taking that slot empties the miner.
    let miner = g.sim.factory.miner_at_mut(m);
    miner.out.add(miner.ore, 7);
    g.take_machine_slot(m.x, m.y, m.z, 0);
    g.run_ticks(1);
    assert_eq!(g.sim.factory.miner_at(m).out.total(), 0);
}
