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
fn sorting_never_merges_tools_and_puts_empty_slots_last() {
    let pick = crate::item::STONE_PICKAXE;
    let mut slots = [Stack::default(); 6];
    slots[1] = Stack { item: pick, count: 40 };
    slots[2] = Stack { item: pick, count: 100 };
    slots[4] = Stack { item: ItemId(3), count: 1 };
    sort_stacks(&mut slots);
    assert_eq!(slots[0], Stack { item: pick, count: 100 }, "tools before blocks, the one with more uses first");
    assert_eq!(slots[1], Stack { item: pick, count: 40 }, "each tool keeps its own wear");
    assert_eq!(slots[2], Stack { item: ItemId(3), count: 1 });
    assert!(slots[3..].iter().all(Stack::is_empty));
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

#[test]
fn splitting_and_placing_one_respect_stack_limits_and_tools() {
    let (a, b) = (ItemId(1), ItemId(2));
    let max = stack_size(a);
    // One item halves to itself; an odd stack gives the larger half; a full cursor takes nothing more.
    let (mut s, mut c) = (Stack { item: a, count: 1 }, Stack::default());
    assert!(split_stack(&mut s, &mut c) && s.is_empty() && c == Stack { item: a, count: 1 });
    let (mut s, mut c) = (Stack { item: a, count: 9 }, Stack { item: a, count: max });
    assert!(!split_stack(&mut s, &mut c) && s.count == 9 && c.count == max);
    let (mut s, mut c) = (Stack { item: a, count: 9 }, Stack { item: a, count: max - 2 });
    assert!(split_stack(&mut s, &mut c) && (s.count, c.count) == (7, max), "topped up to a full stack only");
    assert!(!split_stack(&mut Stack::default(), &mut Stack::default()));
    assert!(!place_one(&mut Stack::default(), &mut Stack::default()), "nothing held, nothing placed");

    // Placing one: a full target refuses, so does another item; the last item empties the cursor.
    let (mut s, mut c) = (Stack { item: a, count: max }, Stack { item: a, count: 3 });
    assert!(!place_one(&mut s, &mut c));
    let (mut s, mut c) = (Stack { item: b, count: 1 }, Stack { item: a, count: 3 });
    assert!(!place_one(&mut s, &mut c));
    let (mut s, mut c) = (Stack::default(), Stack { item: a, count: 1 });
    assert!(place_one(&mut s, &mut c) && c == Stack::default() && s == Stack { item: a, count: 1 });

    // A tool's count is its wear: it is taken, placed and thrown whole, never halved.
    let tool = crate::item::STONE_PICKAXE;
    let (mut s, mut c) = (Stack { item: tool, count: 100 }, Stack::default());
    assert!(split_stack(&mut s, &mut c) && s.is_empty() && c.count == 100);
    let mut slot = Stack::default();
    assert!(place_one(&mut slot, &mut c) && slot.count == 100 && c.is_empty());
    let mut inv = Inventory { cursor: Stack { item: tool, count: 42 }, ..Inventory::default() };
    assert_eq!(inv.take_cursor_one(), Some(Stack { item: tool, count: 42 }));
    inv.cursor = Stack { item: a, count: 2 };
    assert_eq!(inv.take_cursor_one(), Some(Stack { item: a, count: 1 }));
    assert_eq!(inv.cursor.count, 1);
}

fn every_item() -> impl Iterator<Item = ItemId> {
    (1..crate::item::ITEM_COUNT as u16).map(ItemId).filter(|i| i.is_valid())
}

#[test]
fn every_item_sorts_into_the_kind_a_player_expects() {
    use super::kinds::*;
    use crate::{block, item};
    let expect = [
        (ItemId::block(block::MINER), BUILDINGS),
        (item::MINER_MK3, BUILDINGS),
        (ItemId::block(block::BELT), BUILDINGS),
        (item::LOCOMOTIVE, BUILDINGS),
        (item::IRON_PICKAXE, TOOLS),
        (ItemId::block(block::GLASS), BUILDING_BLOCKS),
        (item::IRON_PLATE, INGREDIENTS),
        (item::RED_PACK, INGREDIENTS),
        (ItemId::block(block::IRON_ORE), RAW),
        (ItemId::block(block::SLAG), RAW),
        (ItemId::block(block::DIRT), NATURAL),
        (ItemId::block(block::LOG), NATURAL),
    ];
    for (it, kind) in expect {
        assert_eq!(KINDS[kind_of(it) as usize], KINDS[kind as usize], "{}", item::name(it));
    }
    for it in every_item() {
        let kind = kind_of(it);
        assert!((kind as usize) < KINDS.len(), "{} has no kind", item::name(it));
        if it.0 >= 256 {
            assert!(kind != RAW && kind != NATURAL, "{} is made, not found", item::name(it));
        }
    }
    // Every tier of every machine sorts as a building (or a tool), next to its Mk1.
    for f in crate::factory::tiers::FAMILIES {
        let mk1 = sort_key(f.items[0]) >> 16;
        for &it in f.items {
            assert!(matches!(kind_of(it), BUILDINGS | TOOLS), "{}", item::name(it));
            assert_eq!(sort_key(it) >> 16, mk1, "{} sorts away from its Mk1", item::name(it));
        }
    }
}

#[test]
fn a_fuzzed_sort_keeps_every_item_and_leaves_the_hotbar_alone() {
    let items: Vec<ItemId> = every_item().collect();
    let mut seed = 0x2545_f491_4f6c_dd1du64;
    let mut next = |n: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % n as u64) as usize
    };
    for _ in 0..300 {
        let mut inv = Inventory::default();
        let cap = inv.capacity();
        for s in inv.slots[..cap].iter_mut() {
            if next(3) > 0 {
                // A few items at a time, so equal items meet often.
                let item = items[next(12) * items.len() / 12 + next(3)];
                let max = if tools::tool(item).is_some() { 100 } else { stack_size(item) };
                *s = Stack { item, count: 1 + next(max as usize) as u32 };
            }
        }
        let before = inv.slots;
        inv.sort();
        let hotbar = HOTBAR_SLOTS;
        assert_eq!(inv.slots[..hotbar], before[..hotbar], "the hotbar keeps its layout");
        let tally = |slots: &[Stack]| {
            let mut t: Vec<(u16, u32)> = Vec::new();
            for s in slots.iter().filter(|s| !s.is_empty()) {
                let key = if tools::tool(s.item).is_some() { s.count } else { 0 };
                match t.iter_mut().find(|e| e.0 == s.item.0 && (key == 0 || e.1 == key)) {
                    Some(e) if key == 0 => e.1 += s.count,
                    _ => t.push((s.item.0, if key == 0 { s.count } else { key })),
                }
            }
            t.sort_unstable();
            t
        };
        assert_eq!(tally(&inv.slots[hotbar..cap]), tally(&before[hotbar..cap]), "the sort lost or made items");
        let backpack = &inv.slots[hotbar..cap];
        let filled = backpack.iter().take_while(|s| !s.is_empty()).count();
        assert!(backpack[filled..].iter().all(Stack::is_empty), "empty slots come last");
        let keys: Vec<u64> = backpack[..filled].iter().map(|s| super::kinds::sort_key(s.item)).collect();
        assert!(keys.windows(2).all(|w| w[0] <= w[1]), "sorted by kind");
        let again = inv.slots;
        inv.sort();
        assert_eq!(inv.slots, again, "sorting twice changes nothing");
    }
}
