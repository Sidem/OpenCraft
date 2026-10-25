//! Tests of the left hand: swapping, clicking, a tool in it, saving it and the bytes of older saves.

use super::*;
use crate::action::Action;
use crate::block::{AIR, LOG, STONE, TORCH};
use crate::bytes::{ByteReader, ByteWriter};
use crate::item::{ItemId, IRON_AXE, STONE_AXE, STONE_PICKAXE};
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim, SimEvent};
use crate::tests::run_until_ready;
use crate::Game;

const P: PlayerId = PlayerId(0);

fn stack(item: impl Into<ItemId>, count: u32) -> Stack {
    Stack { item: item.into(), count }
}

#[test]
fn swapping_trades_the_selected_stack_with_the_left_hand_and_back() {
    let mut inv = Inventory::default();
    inv.slots[2] = stack(TORCH, 20);
    inv.select(2);
    let before = inv.version;
    inv.swap_offhand();
    assert_eq!((inv.offhand, inv.selected_stack()), (stack(TORCH, 20), Stack::default()), "the whole stack moves");
    inv.slots[2] = stack(STONE, 5);
    inv.swap_offhand();
    assert_eq!((inv.offhand, inv.selected_stack()), (stack(STONE, 5), stack(TORCH, 20)));
    assert!(inv.version > before);

    let mut empty = Inventory::default();
    empty.swap_offhand();
    assert_eq!(empty.version, 0, "two empty hands change nothing");
}

#[test]
fn the_cursor_clicks_the_left_hand_like_a_slot() {
    let mut inv = Inventory { cursor: stack(TORCH, 3), ..Default::default() };
    inv.click_offhand();
    assert_eq!((inv.offhand, inv.cursor), (stack(TORCH, 3), Stack::default()));
    inv.cursor = stack(TORCH, 4);
    inv.click_offhand();
    assert_eq!((inv.offhand.count, inv.cursor.count), (7, 0), "the same item merges");
    inv.click_offhand();
    assert_eq!((inv.offhand, inv.cursor), (Stack::default(), stack(TORCH, 7)), "an empty cursor picks it up");
}

#[test]
fn a_tool_in_the_left_hand_serves_when_the_right_hands_is_not_for_the_block() {
    let mut inv = Inventory::default();
    inv.slots[0] = stack(STONE_PICKAXE, 100);
    inv.offhand = stack(IRON_AXE, 50);
    assert_eq!((inv.tool_hand(STONE), inv.tool_hand(LOG)), (Hand::Right, Hand::Left));
    assert_eq!(inv.tool_hand(AIR), Hand::Right, "no tool for it: the right hand, as before");
    inv.offhand = stack(STONE_AXE, 1);
    assert!(inv.wear_hand(Hand::Left) && inv.offhand.is_empty(), "the last use wears it out");
    assert_eq!(inv.tool_hand(LOG), Hand::Right);
    inv.slots[0] = stack(STONE_AXE, 2);
    assert_eq!(inv.tool_hand(LOG), Hand::Right, "when both are for it, the right hand goes first");
    assert!(!inv.wear_hand(Hand::Right) && inv.selected_stack().count == 1);
}

#[test]
fn breaking_with_a_left_hand_axe_wears_it_and_tells_when_it_is_gone() {
    let mut sim = Sim::new(7, 2);
    let at = IVec3::new(0, 200, 0);
    sim.world.set_block_anywhere(at, LOG);
    sim.apply(P, Action::Give { item: STONE_AXE, count: 2 });
    sim.apply(P, Action::Offhand { swap: true });
    assert_eq!(sim.player(P).unwrap().inventory.offhand, stack(STONE_AXE, 2));
    sim.apply(P, Action::BreakBlock { pos: at });
    assert_eq!(sim.world.block_anywhere_or_generate(at), AIR);
    assert_eq!(sim.player(P).unwrap().inventory.offhand.count, 1, "one use gone, and none from the right hand");

    sim.world.set_block_anywhere(at, LOG);
    sim.events.clear();
    sim.apply(P, Action::BreakBlock { pos: at });
    assert!(sim.player(P).unwrap().inventory.offhand.is_empty());
    assert!(sim.events.iter().any(|e| matches!(e, SimEvent::ToolWornOut { item, .. } if *item == STONE_AXE)));
}

#[test]
fn a_pickaxe_in_the_left_hand_breaks_stone_faster_than_bare_hands() {
    fn ticks(g: &mut Game) -> u32 {
        let p = IVec3::new(0, 200, 0);
        g.sim.world.set_block_anywhere(p, STONE);
        g.teleport(0.5, 202.0, 0.5);
        g.set_look(0.0, -1.55);
        g.set_mining(true);
        for t in 1..600 {
            g.update(1.0 / 60.0);
            if g.sim.world.get_block(p) == Some(AIR) {
                g.set_mining(false);
                (0..30).for_each(|_| g.update(1.0 / 60.0));
                return t;
            }
        }
        panic!("never broke");
    }
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    crate::tests::fly(&mut g);
    g.select_slot(8);
    let bare = ticks(&mut g);
    g.give(STONE_PICKAXE.0, 150);
    g.update(1.0 / 60.0);
    let slot = (0..9).find(|&i| g.slot_item(i) == STONE_PICKAXE.0).expect("in the hotbar");
    g.select_slot(slot);
    g.swap_offhand();
    g.update(1.0 / 60.0);
    assert_eq!((g.offhand_item(), g.offhand_count()), (STONE_PICKAXE.0, 150));
    let left = ticks(&mut g);
    assert!(left * 2 <= bare + 2, "{left} ticks with the left hand's pickaxe, {bare} bare");
    assert_eq!(g.offhand_count(), 149);
}

#[test]
fn the_left_hand_is_saved_and_older_saves_have_it_empty() {
    let inv = Inventory { offhand: stack(TORCH, 12), ..Default::default() };
    let mut w = ByteWriter::default();
    inv.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    assert_eq!(Inventory::read_state(&mut r).map(|i| i.offhand), Some(stack(TORCH, 12)));
    assert!(r.is_done());

    // Version 59 wrote the same bytes without the stack at the end.
    let mut w = ByteWriter::default();
    Inventory::default().write_state(&mut w);
    let mut last = ByteWriter::default();
    Stack::default().write_state(&mut last);
    let old = &w.bytes[..w.bytes.len() - last.bytes.len()];
    let mut r = ByteReader::new(old);
    r.version = 59;
    assert_eq!(Inventory::read_state(&mut r).map(|i| i.offhand), Some(Stack::default()));
    assert!(r.is_done(), "all of it was read, and no more");
}

#[test]
fn the_action_travels_in_one_byte_pair_and_back() {
    for swap in [true, false] {
        let mut w = ByteWriter::default();
        Action::Offhand { swap }.write(&mut w);
        assert_eq!(w.bytes.len(), 2);
        let mut r = ByteReader::new(&w.bytes);
        assert!(matches!(Action::read(&mut r), Some(Action::Offhand { swap: s }) if s == swap) && r.is_done());
    }
}
