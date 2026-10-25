use crate::block::{self, HOUSE_COUNT, WOOD_DOOR};
use crate::item::ItemId;
use crate::recipes::{Group, RECIPES};

const FIRST: block::BlockId = WOOD_DOOR;

#[test]
fn every_placeable_house_block_has_one_hand_recipe_in_the_building_section() {
    for id in FIRST..FIRST + HOUSE_COUNT as block::BlockId {
        let def = block::def(id);
        let made: Vec<_> = RECIPES.iter().filter(|r| r.output == ItemId::block(id)).collect();
        if def.placeable {
            assert_eq!(made.len(), 1, "{} has one recipe", def.name);
            assert_eq!(made[0].group, Group::Building);
        } else {
            assert!(made.is_empty(), "{} is only ever the twin of a placed block", def.name);
        }
    }
}

#[test]
fn the_house_set_is_the_ids_after_the_coal_miner() {
    assert_eq!(FIRST, block::COAL_MINER + 1);
    assert_eq!(FIRST as usize + HOUSE_COUNT, block::BLOCK_COUNT);
}
