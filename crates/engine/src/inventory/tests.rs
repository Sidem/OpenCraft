use super::*;
use crate::item::ItemId;

#[test]
fn stacks_then_fills_empty_slots() {
    let mut inv = Inventory::default();
    assert_eq!(inv.add(ItemId(1), 10), 0);
    assert_eq!(inv.add(ItemId(1), 60), 0);
    assert_eq!(inv.slots[0], Stack { item: ItemId(1), count: 64 });
    assert_eq!(inv.slots[1], Stack { item: ItemId(1), count: 6 });
    assert_eq!(inv.add(ItemId(2), 3), 0);
    assert_eq!(inv.slots[2], Stack { item: ItemId(2), count: 3 });
}

#[test]
fn overflow_is_returned() {
    let mut inv = Inventory::default();
    assert_eq!(inv.add(ItemId(1), 64 * INVENTORY_SLOTS as u32 + 5), 5);
    assert_eq!(inv.space_for(ItemId(1)), 0);
    assert_eq!(inv.space_for(ItemId(2)), 0);
}

#[test]
fn hotbar_fills_before_backpack() {
    let mut inv = Inventory::default();
    inv.add(ItemId(1), 64 * 10);
    assert!(inv.slots[..HOTBAR_SLOTS].iter().all(|s| s.item == ItemId(1) && s.count == 64));
    assert_eq!(inv.slots[HOTBAR_SLOTS], Stack { item: ItemId(1), count: 64 });
}

#[test]
fn take_clears_empty_slot() {
    let mut inv = Inventory::default();
    inv.add(ItemId(3), 1);
    assert_eq!(inv.take_slot(inv.selected, 1), Some((ItemId(3), 1)));
    assert!(inv.slots[0].is_empty());
    assert_eq!(inv.take_slot(inv.selected, 1), None);
}

#[test]
fn scroll_wraps() {
    let mut inv = Inventory::default();
    inv.scroll(-1);
    assert_eq!(inv.selected, HOTBAR_SLOTS - 1);
    inv.scroll(1);
    assert_eq!(inv.selected, 0);
}

#[test]
fn remove_takes_from_backpack_first() {
    let mut inv = Inventory::default();
    inv.slots[0] = Stack { item: ItemId(5), count: 10 };
    inv.slots[20] = Stack { item: ItemId(5), count: 4 };
    assert!(inv.remove(ItemId(5), 6));
    assert!(inv.slots[20].is_empty());
    assert_eq!(inv.slots[0].count, 8);
    assert!(!inv.remove(ItemId(5), 9), "not enough left");
    assert_eq!(inv.count(ItemId(5)), 8);
}

#[test]
fn cursor_pick_place_merge_swap() {
    let mut inv = Inventory::default();
    inv.slots[0] = Stack { item: ItemId(1), count: 40 };
    inv.slots[1] = Stack { item: ItemId(1), count: 30 };
    inv.slots[2] = Stack { item: ItemId(2), count: 5 };
    inv.click(0);
    assert_eq!(inv.cursor, Stack { item: ItemId(1), count: 40 });
    inv.click(1);
    assert_eq!(inv.slots[1].count, 64);
    assert_eq!(inv.cursor.count, 6);
    inv.click(2);
    assert_eq!(inv.slots[2], Stack { item: ItemId(1), count: 6 });
    assert_eq!(inv.cursor, Stack { item: ItemId(2), count: 5 });
    inv.click(10);
    assert!(inv.cursor.is_empty());
    assert_eq!(inv.slots[10], Stack { item: ItemId(2), count: 5 });
}

#[test]
fn quick_move_between_sections() {
    let mut inv = Inventory::default();
    inv.slots[3] = Stack { item: ItemId(7), count: 12 };
    inv.quick_move(3);
    assert!(inv.slots[3].is_empty());
    assert_eq!(inv.slots[HOTBAR_SLOTS], Stack { item: ItemId(7), count: 12 });
    inv.quick_move(HOTBAR_SLOTS);
    assert_eq!(inv.slots[0], Stack { item: ItemId(7), count: 12 });
}
