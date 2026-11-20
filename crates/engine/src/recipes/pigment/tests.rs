use crate::block;
use crate::item::{self, ItemId};
use crate::recipes::{b, MACHINE_RECIPES, RECIPES};

/// Each ground pigment with the mineral it is ground from, the coloured concrete it tints and the roof it colours.
const BASES: [(ItemId, ItemId, block::BlockId, Option<block::BlockId>); 5] = [
    (item::PIGMENT_RED, item::IRON_INGOT, block::RED_CONCRETE, Some(block::RED_ROOF)),
    (item::PIGMENT_GREEN, item::COPPER_INGOT, block::GREEN_CONCRETE, Some(block::GREEN_ROOF)),
    (item::PIGMENT_BLACK, b(block::COAL_ORE), block::BLACK_CONCRETE, Some(block::SLATE_ROOF)),
    (item::PIGMENT_WHITE, item::QUICKLIME, block::WHITE_CONCRETE, None),
    (item::PIGMENT_YELLOW, item::SULFUR, block::YELLOW_CONCRETE, None),
];

fn made_from(output: ItemId) -> Vec<Vec<ItemId>> {
    let by_hand = RECIPES.iter().filter(|r| r.output == output).map(|r| r.inputs.iter().map(|x| x.0).collect());
    let by_machine = MACHINE_RECIPES.iter().filter(|r| r.outputs.iter().any(|x| x.0 == output));
    by_hand.chain(by_machine.map(|r| r.inputs.iter().map(|x| x.0).collect())).collect()
}

#[test]
fn each_pigment_is_one_mineral_ground_by_hand() {
    for (pigment, mineral, _, _) in BASES {
        assert_eq!(made_from(pigment), vec![vec![mineral]], "{:?}", pigment);
        assert!(RECIPES.iter().any(|r| r.output == pigment && r.inputs == [(mineral, 1)]));
    }
}

#[test]
fn every_colour_that_existed_is_now_dye_made() {
    let minerals = [b(block::IRON_ORE), b(block::COPPER_ORE), b(block::COAL_ORE), item::QUICKLIME, item::SULFUR];
    for (pigment, _, concrete, roof) in BASES {
        for coloured in [Some(concrete), roof].into_iter().flatten() {
            let ways = made_from(b(coloured));
            assert_eq!(ways.len(), 1, "{} has one recipe", block::def(coloured).name);
            assert!(ways[0].contains(&pigment), "{} takes its pigment", block::def(coloured).name);
            assert!(
                !ways[0].iter().any(|i| minerals.contains(i)),
                "{} takes no raw mineral",
                block::def(coloured).name
            );
        }
    }
}
