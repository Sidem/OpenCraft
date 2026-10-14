use super::*;
use crate::block::{DIESEL_GENERATOR, QUARRY as QUARRY_BLOCK, REACTOR, SMELTER};
use crate::item::{IRON_PLATE, SMELTER_MK3};

fn rows(item: ItemId) -> String {
    facts(item).expect("a machine").rows()
}

#[test]
fn a_burner_names_its_fuel_and_an_electric_machine_its_kw() {
    let mk1 = rows(ItemId::block(SMELTER));
    assert!(mk1.contains("size\tSize\t1×1×1") && mk1.contains("fuel\tBurns\tCoal"), "{mk1}");
    assert!(!mk1.contains("use\t"), "{mk1}");
    let mk3 = rows(SMELTER_MK3);
    assert!(mk3.contains("use\tUses\t40 kW") && !mk3.contains("fuel\t"), "{mk3}");
}

#[test]
fn generators_show_what_they_make_and_what_they_burn() {
    let coal = rows(ItemId::block(crate::block::GENERATOR));
    assert!(coal.contains("make\tMakes\tup to 60 kW") && coal.contains("fuel\tBurns\tCoal"), "{coal}");
    let reactor = rows(ItemId::block(REACTOR));
    assert!(reactor.contains("size\tSize\t3×3×3") && reactor.contains("up to 2,000 kW"), "{reactor}");
    assert!(reactor.contains("Fuel Cell"), "{reactor}");
    assert!(rows(ItemId::block(DIESEL_GENERATOR)).contains("Diesel Canister"));
}

#[test]
fn a_machine_that_needs_no_energy_says_so_and_a_quarry_sets_its_own_size() {
    let belt = rows(ItemId::block(crate::block::BELT));
    assert!(belt.contains("none\tEnergy\tNone needed") && belt.contains("stat\tSpeed"), "{belt}");
    assert!(rows(ItemId::block(QUARRY_BLOCK)).contains("Set when placed"));
}

#[test]
fn things_that_are_not_machines_have_no_card() {
    assert!(facts(IRON_PLATE).is_none());
    assert!(facts(ItemId::block(crate::block::PLANKS)).is_none());
}
