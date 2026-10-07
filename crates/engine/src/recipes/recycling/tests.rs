use super::*;
use crate::block::{
    BELT, CONCRETE, COPPER_ORE, DIRT, GLASS, IRON_ORE, LEAVES, LOG, PLANKS, SAND, SAPLING, SLAG, STONE, TAILINGS,
};
use crate::item::{
    ItemId, COIN, COPPER_WIRE, CRUSHED_IRON, DRONE, GEAR, IRON_INGOT, IRON_PLATE, IRON_ROD, SCREW, STEEL_INGOT,
    STEEL_PICKAXE, STICK, WASHED_IRON,
};

fn b(block: u8) -> ItemId {
    ItemId::block(block)
}

/// What `item` pays, in whole coins and a thousandth of one.
fn pays(item: ItemId) -> f64 {
    millicoins(item) as f64 / MILLI as f64
}

#[test]
fn raw_things_pay_one() {
    for item in [DIRT, SAND, LEAVES, SAPLING, LOG, STONE, IRON_ORE, COPPER_ORE] {
        assert_eq!(coins(b(item)), 1, "{}", crate::item::name(b(item)));
    }
}

#[test]
fn each_step_of_processing_doubles_it() {
    assert_eq!(coins(IRON_INGOT), 2, "smelted");
    assert_eq!(coins(IRON_ROD), 4, "smelted, then pressed");
    assert_eq!(coins(COPPER_WIRE), 2, "two wires from an ingot (2), the batch doubled to 4, shared out");
    assert_eq!(coins(IRON_PLATE), 8, "two ingots pressed");
    assert_eq!(coins(SCREW), 2, "four from a rod");
    assert_eq!(coins(GEAR), 16, "a plate pressed again");
    assert_eq!(coins(STEEL_INGOT), 10, "two ore, a coal and a quicklime (smelted limestone) blasted");
}

#[test]
fn small_shares_pay_a_fraction_that_adds_up() {
    assert_eq!(pays(b(PLANKS)), 0.5, "four from a log: the batch's 2 coins shared out");
    assert_eq!(pays(STICK), 0.5);
    // A steel pickaxe is 1500 uses made from 68 coins of parts: its batch of 136 coins is spread over its uses.
    let batch = millicoins(STEEL_PICKAXE) as u64 * crate::tools::STEEL_TIER.uses as u64;
    assert!((136_000..136_000 + 1500).contains(&batch), "{batch}");
}

#[test]
fn the_route_does_not_change_the_worth() {
    // Raw, crushed and washed ore all end in the same ingot, and the crushed and washed ones are not worth less
    // than the ore (a better route cannot lower what an ingot is worth, nor print coins).
    assert_eq!(coins(IRON_INGOT), 2);
    assert!(millicoins(CRUSHED_IRON) >= millicoins(b(IRON_ORE)));
    assert!(millicoins(WASHED_IRON) >= millicoins(b(IRON_ORE)));
}

#[test]
fn waste_pays_one_and_so_does_what_is_crushed_from_it() {
    assert_eq!(coins(b(SLAG)), 1);
    assert_eq!(coins(b(TAILINGS)), 1);
    assert_eq!(coins(b(SAND)), 1, "sand crushed from slag is still 1");
    assert_eq!(coins(b(GLASS)), 1, "glass is cheapest from quartz: 2 a smelted ore, so the batch's 2 is shared out");
    assert!(is_waste(b(SLAG)) && !is_waste(b(SAND)));
}

#[test]
fn the_coin_is_worth_nothing_and_every_other_item_something() {
    assert_eq!(millicoins(COIN), 0);
    assert_eq!(millicoins(ItemId(60000)), 0);
    for id in 1..ITEM_COUNT as u16 {
        let item = ItemId(id);
        if item != COIN && item.is_valid() {
            assert!(millicoins(item) >= 1, "{} has no value", crate::item::name(item));
        }
    }
}

#[test]
fn no_recipe_could_lower_a_value() {
    // The settled table is a fixpoint: a batch of any recipe is never worth less than what it makes, so a
    // cheaper route would have been found. (Salvage from waste alone pays 1.)
    let check = |inputs: &[(ItemId, u32)], outputs: &[(ItemId, u32)]| {
        let shares: u64 = outputs.iter().filter(|o| !is_waste(o.0)).map(|o| o.1 as u64).sum();
        if shares == 0 {
            return;
        }
        let all_waste = inputs.iter().all(|i| is_waste(i.0));
        let total: u64 = inputs.iter().map(|i| millicoins(i.0) as u64 * i.1 as u64).sum();
        let each = if all_waste { MILLI as u64 } else { (STEP * total).div_ceil(shares) };
        for o in outputs.iter().filter(|o| !is_waste(o.0) && o.0 != COIN) {
            assert!(millicoins(o.0) as u64 <= each, "{} is worth more than a recipe makes it", crate::item::name(o.0));
        }
    };
    MACHINE_RECIPES.iter().for_each(|r| check(r.inputs, r.outputs));
    RECIPES.iter().for_each(|r| check(r.inputs, &[(r.output, r.count)]));
}

#[test]
fn values_are_sane_for_a_machine_to_pay() {
    let max = (1..ITEM_COUNT as u16).map(|i| millicoins(ItemId(i))).max().unwrap();
    assert!(max < 400_000_000, "the dearest item pays {} coins", max / MILLI);
    assert!(coins(DRONE) > coins(b(BELT)), "a drone is worth more than a belt");
    assert!(coins(b(CONCRETE)) > 1, "concrete is made");
}
