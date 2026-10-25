use super::*;
use crate::block::{flow, WATER};
use crate::item::{GOLD_PACK, IRON_INGOT};
use crate::research::{TechState, TECHS};
use crate::Game;

fn all_techs_done(g: &Game) -> bool {
    (0..TECHS.len() as u8).all(|t| g.sim.factory.research.state(t) == TechState::Done)
}

#[test]
fn the_item_list_is_every_real_item_once() {
    let items = creative_items();
    assert!(items.iter().all(|i| i.is_valid() && !crate::item::name(*i).is_empty()));
    assert!(items.windows(2).all(|w| w[0] < w[1]), "in id order, none twice");
    assert!(items.contains(&IRON_INGOT) && items.contains(&GOLD_PACK));
    assert!(!items.contains(&WATER.into()), "water is not an item");
    assert!(!items.contains(&ItemId::block(flow(1))), "nor is flowing water");
    // Every tier of a machine can be taken, not only the first.
    assert!(items.contains(&crate::item::BLAST_FURNACE_MK5));
}

#[test]
fn a_creative_world_has_every_tech_and_gives_stacks() {
    let mut g = Game::new_creative(7, 2);
    assert!(g.is_creative() && all_techs_done(&g));
    assert_eq!(g.creative_items().len(), creative_items().len());
    g.creative_give(IRON_INGOT.0, true);
    g.creative_give(GOLD_PACK.0, false);
    g.run_ticks(1);
    assert_eq!(g.item_total(IRON_INGOT.0), crate::item::stack_size(IRON_INGOT));
    assert_eq!(g.item_total(GOLD_PACK.0), 1);
}

#[test]
fn a_normal_world_gives_nothing_and_keeps_its_research() {
    let mut g = Game::new(7, 2);
    assert!(!g.is_creative() && !all_techs_done(&g) && g.creative_items().is_empty());
    g.creative_give(IRON_INGOT.0, true);
    g.run_ticks(1);
    assert_eq!(g.item_total(IRON_INGOT.0), 0);
}

#[test]
fn the_mode_survives_a_save_and_never_changes() {
    let mut g = Game::new_creative(7, 2);
    g.run_ticks(2);
    let back = Game::from_save(&g.save(), 2).expect("loads");
    assert!(back.is_creative() && all_techs_done(&back));
    assert_eq!(back.sim.state_hash(), g.sim.state_hash());

    let mut plain = Game::new(7, 2);
    plain.run_ticks(2);
    let back = Game::from_save(&plain.save(), 2).expect("loads");
    assert!(!back.is_creative());
    assert_ne!(back.sim.state_hash(), g.sim.state_hash(), "the mode is part of the state");
}

#[test]
fn only_a_creative_world_can_fly() {
    let mut plain = Game::new(7, 2);
    plain.toggle_fly();
    plain.run_ticks(1);
    assert!(!plain.flying(), "F does nothing in a normal world");

    let mut creative = Game::new_creative(7, 2);
    creative.toggle_fly();
    creative.run_ticks(1);
    assert!(creative.flying(), "F still works in a creative world");
}

#[test]
fn a_normal_world_saved_while_flying_loses_the_flight() {
    // The playtest save that flew at 24 blocks a second with nothing worn: flying was in the saved body.
    let mut g = Game::new(7, 2);
    g.body_mut().flying = true;
    let mut back = Game::from_save(&g.save(), 2).expect("loads");
    assert!(back.flying(), "the file says flying");
    back.run_ticks(1);
    assert!(!back.flying(), "a normal world takes it back on the first tick");

    let mut creative = Game::new_creative(7, 2);
    creative.body_mut().flying = true;
    let mut back = Game::from_save(&creative.save(), 2).expect("loads");
    back.run_ticks(1);
    assert!(back.flying(), "a creative world keeps it");
}

#[test]
fn a_hover_without_a_charged_pack_does_not_last() {
    let mut g = Game::new(7, 2);
    let me = g.sim.players[0].as_mut().expect("the local player");
    assert!(!me.inventory.has_hover_pack());
    (me.helpers.hover, me.helpers.charge) = (true, 5_000);
    g.run_ticks(1);
    assert!(!g.sim.players[0].as_ref().unwrap().helpers.hover, "no pack worn: no hover, charge or not");
}

#[test]
fn a_creative_world_completes_techs_added_after_it_was_saved() {
    let mut g = Game::new_creative(7, 2);
    // Pretend the newest tech did not exist yet when this world was saved.
    let last = TECHS.len() as u8 - 1;
    g.sim.factory.research = crate::research::Research::default();
    assert_eq!(g.sim.factory.research.state(last), TechState::Locked);
    let back = Game::from_save(&g.save(), 2).expect("loads");
    assert!(all_techs_done(&back));
}
