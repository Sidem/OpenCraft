//! Tests of the left hand: swapping, clicking, saving it and the bytes of older saves.

use super::*;
use crate::action::Action;
use crate::block::{STONE, TORCH};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::ItemId;

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
fn the_left_hand_is_saved_and_older_saves_have_it_empty() {
    let inv = Inventory { offhand: stack(TORCH, 12), ..Default::default() };
    let mut w = ByteWriter::default();
    inv.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    assert_eq!(Inventory::read_state(&mut r).map(|i| i.offhand), Some(stack(TORCH, 12)));
    assert!(r.is_done());

    // Version 59 wrote the same bytes without the stack at the end (and without the head slot, version 78's).
    let before_head = Inventory::default().state_before_head_slot();
    let mut last = ByteWriter::default();
    Stack::default().write_state(&mut last);
    let old = &before_head[..before_head.len() - last.bytes.len()];
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
