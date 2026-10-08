use super::*;
use crate::block::{
    BELT, CONCRETE, COPPER_ORE, DIRT, GLASS, IRON_ORE, LEAVES, LOG, PLANKS, SAND, SAPLING, SLAG, STONE, TAILINGS,
};
use crate::item::{
    ItemId, COIN, COPPER_WIRE, CRUSHED_IRON, DRONE, GEAR, IRON_INGOT, IRON_PICKAXE, IRON_PLATE, IRON_ROD, STEEL_INGOT,
    STEEL_PICKAXE, STONE_PICKAXE, WASHED_IRON,
};

fn b(block: u8) -> ItemId {
    ItemId::block(block)
}

/// What a whole `item` pays, in coins (a tool: all its uses).
fn pays(item: ItemId) -> u64 {
    let uses = crate::tools::tool(item).map_or(1, |t| t.tier.uses as u64);
    (millicoins(item) as u64 * uses + MILLI as u64 / 2) / MILLI as u64
}

/// Every real item, with what it pays whole.
fn all() -> Vec<(ItemId, u64)> {
    (1..ITEM_COUNT as u16).map(ItemId).filter(|&i| i != COIN && i.is_valid()).map(|i| (i, pays(i))).collect()
}

#[test]
fn the_dearest_item_pays_exactly_the_maximum_and_nothing_pays_more() {
    let max = all().into_iter().map(|(_, c)| c).max().unwrap();
    assert_eq!(max, MAX_COINS);
}

#[test]
fn every_item_pays_a_whole_number_of_coins_and_at_least_one() {
    for (item, coins) in all() {
        assert!(coins >= 1, "{} pays nothing", crate::item::name(item));
        let uses = crate::tools::tool(item).map_or(1, |t| t.tier.uses as u64);
        if uses == 1 {
            assert_eq!(millicoins(item) % MILLI, 0, "{} pays a fraction", crate::item::name(item));
        }
    }
}

#[test]
fn raw_things_and_waste_pay_the_same() {
    let raw = pays(b(DIRT));
    for item in [SAND, LEAVES, SAPLING, LOG, STONE, IRON_ORE, COPPER_ORE, SLAG, TAILINGS] {
        assert_eq!(pays(b(item)), raw, "{}", crate::item::name(b(item)));
    }
    assert!(is_waste(b(SLAG)) && !is_waste(b(SAND)));
}

#[test]
fn processing_adds_value_along_a_chain() {
    assert!(pays(IRON_INGOT) >= pays(b(IRON_ORE)));
    assert!(pays(IRON_PLATE) >= pays(IRON_INGOT));
    assert!(pays(GEAR) >= pays(IRON_PLATE));
    assert!(pays(STEEL_INGOT) >= pays(IRON_INGOT));
    assert!(pays(IRON_ROD) >= pays(IRON_INGOT) && pays(COPPER_WIRE) >= 1);
    assert!(pays(b(CONCRETE)) >= pays(b(SAND)) && pays(b(GLASS)) >= pays(b(SAND)));
    assert!(pays(b(PLANKS)) >= 1);
}

#[test]
fn the_route_does_not_change_the_worth() {
    // Crushed and washed ore end in the same ingot, and are not worth less than the ore.
    assert!(millicoins(CRUSHED_IRON) >= millicoins(b(IRON_ORE)));
    assert!(millicoins(WASHED_IRON) >= millicoins(b(IRON_ORE)));
}

#[test]
fn a_tool_pays_by_its_uses_so_a_worn_one_pays_less() {
    assert!(pays(STEEL_PICKAXE) > pays(IRON_PICKAXE) && pays(IRON_PICKAXE) > pays(STONE_PICKAXE));
    let per_use = millicoins(STEEL_PICKAXE);
    assert!((1..MILLI).contains(&per_use), "a use pays a fraction of a coin: {per_use}");
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
fn values_keep_their_order() {
    assert!(pays(DRONE) > pays(b(BELT)), "a drone is worth more than a belt");
    assert!(pays(b(CONCRETE)) >= pays(b(DIRT)), "concrete is made");
}
/// `DUMP=table.tsv cargo test -p opencraft-engine dump_table -- --ignored` writes name and coins of every item.
#[test]
#[ignore]
fn dump_table() {
    let rows: Vec<String> = all().iter().map(|&(i, c)| format!("{}\t{}\n", crate::item::name(i), c)).collect();
    std::fs::write(std::env::var("DUMP").unwrap(), rows.concat()).unwrap();
}
