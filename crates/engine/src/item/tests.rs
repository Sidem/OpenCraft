use super::*;
use crate::block::{BEDROCK, BELT, IRON_ORE, STONE};

#[test]
fn block_items_share_the_block_ids() {
    let stone = ItemId::block(STONE);
    assert_eq!(stone.0, STONE as u16);
    assert_eq!(name(stone), "Stone");
    assert_eq!(stone.places(), Some(STONE));
    assert_eq!(ItemId::block(BELT).places(), Some(BELT));
    assert_eq!(ItemId::block(IRON_ORE).places(), None, "ore stays in the ground");
    assert_eq!(ItemId::block(BEDROCK).places(), None);
    assert_eq!(def(stone).unwrap().size, [1.0; 3]);
}

#[test]
fn ingots_are_items_but_not_blocks() {
    for (id, label) in [(IRON_INGOT, "Iron Ingot"), (COPPER_INGOT, "Copper Ingot")] {
        assert!(id.is_valid());
        assert_eq!(name(id), label);
        assert_eq!(id.places(), None);
        assert_eq!(stack_size(id), MAX_STACK);
    }
}

#[test]
fn unknown_ids_are_not_items() {
    assert!(!ItemId::NONE.is_valid());
    assert!(!ItemId(BLOCK_COUNT as u16).is_valid(), "past the blocks");
    assert!(!ItemId(256 + EXTRA.len() as u16).is_valid(), "past the extra items");
    assert_eq!(name(ItemId(9999)), "");
}

#[test]
fn every_belt_tier_is_drawn_the_same_way() {
    // Top, side, bottom: the belt's own top texture over the plain frame (Mk1's block and Mk2's block are drawn so).
    let frame = def(ItemId::block(crate::block::FAST_BELT)).unwrap().tex[1];
    for belt in [ItemId::block(BELT), ItemId::block(crate::block::FAST_BELT), BELT_MK3, BELT_MK4] {
        let tex = def(belt).unwrap().tex;
        assert_eq!(tex[1..], [frame; 2], "{}'s sides and bottom are the frame", name(belt));
        assert_ne!(tex[0], frame, "{}'s top shows the belt", name(belt));
    }
}
