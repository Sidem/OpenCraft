//! The left hand informs: a scanner there looks for the right hand's ore, a core drill there forecasts how long a miner
//! in the right hand would last on the aimed deposit, and nothing queues a core action.

use super::*;
use crate::action::Action;
use crate::block::{self, STONE};
use crate::item::{SCANNER, SCANNER_MK2, TELESCOPE};
use crate::prospect::SCAN_FIELDS;
use crate::raycast::RayHit;
use crate::tests::{find_outcrop_block, run_until_ready};

/// Gives the local player `item` and selects it (the right hand).
fn select(g: &mut Game, item: ItemId) {
    g.act(Action::Give { item, count: 1 });
    g.run_ticks(1);
    let slot = g.inventory().slots.iter().position(|s| s.item == item).unwrap();
    g.act(Action::SelectSlot { slot: slot as u8 });
    g.run_ticks(1);
}

/// Puts `item` in the left hand.
fn hold_left(g: &mut Game, item: ItemId) {
    select(g, item);
    g.act(Action::Offhand { swap: true });
    g.run_ticks(1);
    assert_eq!(g.left_item(), item);
}

#[test]
fn g_works_the_scanner_the_core_drill_and_the_telescope() {
    assert_eq!(left_use_of(SCANNER), LEFT_SCAN);
    assert_eq!(left_use_of(SCANNER_MK2), LEFT_SCAN);
    assert_eq!(left_use_of(CORE_DRILL), LEFT_DRILL);
    assert_eq!(left_use_of(TELESCOPE), LEFT_SCOPE);
    assert_eq!(left_use_of(ItemId::block(STONE)), LEFT_NONE);
    assert_eq!(left_use_of(ItemId::NONE), LEFT_NONE);
}

#[test]
fn a_scanner_in_the_left_hand_looks_for_the_ore_in_the_right() {
    for scanner in [SCANNER, SCANNER_MK2] {
        let mut g = Game::new(2024, 2);
        run_until_ready(&mut g);
        hold_left(&mut g, scanner);
        g.run_ticks(1);
        let all = g.prospect_records();
        assert_eq!(g.scan_filter(), 0, "an empty right hand looks for every ore");
        let ore = all[SCAN_FIELDS..].chunks_exact(SCAN_FIELDS).map(|r| r[0]).find(|&o| o != all[0]).expect("two ores");
        select(&mut g, ItemId::block(ore as block::BlockId));
        assert_eq!(i32::from(g.scan_filter()), ore);
        let kept = g.prospect_records();
        assert!(!kept.is_empty() && kept.len() < all.len());
        assert!(kept.chunks_exact(SCAN_FIELDS).all(|r| r[0] == ore), "only that ore");
        assert_eq!(g.sim.pending_len(), 0);
    }
}

#[test]
fn a_core_drill_forecasts_the_miner_in_the_right_hand_on_the_aimed_deposit() {
    let mut g = Game::new(2024, 2);
    run_until_ready(&mut g);
    let (p, key) = find_outcrop_block(&mut g, 3);
    hold_left(&mut g, CORE_DRILL);
    // No miner in the right hand: no forecast.
    g.target = Some(RayHit { block: p, normal: IVec3::new(0, 1, 0), id: g.sim.world.get_block(p).unwrap() });
    g.update_drill_forecast();
    assert!(g.left_hand_label().is_none());
    select(&mut g, ItemId::block(block::MINER));
    let hash = g.sim.state_hash();
    g.target = Some(RayHit { block: p, normal: IVec3::new(0, 1, 0), id: g.sim.world.get_block(p).unwrap() });
    g.update_drill_forecast();
    let label = g.left_hand_label().expect("a forecast");
    let st = g.sim.factory.deposits.get(&key).unwrap();
    assert!(label.starts_with("Core drill\n@bar|"), "{label}");
    assert!(label.contains(&st.deposit.name()) && label.contains("Mk1 +1 /s") && label.contains("years"), "{label}");
    // An outcrop's draw cap is one unit a second: its units are the seconds a Mk1 takes.
    let year = YEAR_DAYS * DAY_TICKS / u64::from(TICK_RATE);
    let tenths = st.remaining_units() as u64 * 10 / year;
    assert!(label.contains(&format!("{}.{} years", tenths / 10, tenths % 10)), "{label}");
    // Stone holds no deposit; the forecast is a query only.
    assert!(g.drill_forecast(p + IVec3::new(0, 40, 0), 0).is_none());
    assert_eq!(g.sim.state_hash(), hash, "forecasting changed the core");
    assert_eq!(g.sim.pending_len(), 0);
}
