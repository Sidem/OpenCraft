use super::*;
use crate::block::{FLOW_7, ICE, STONE};
use crate::daytime::{DAY_TICKS, START, YEAR_DAYS};

#[test]
fn only_the_top_layer_of_still_water_freezes_and_sea_needs_deeper_cold() {
    let gen = WorldGen::new(2024);
    let mut air = Atmosphere::default();
    let winter = (YEAR_DAYS + 33) * DAY_TICKS + DAY_TICKS / 2 - START;
    air.advance(&gen, winter);
    let lake_y = SEA_LEVEL + 80;
    assert!(should_freeze_at(&air, &gen, WATER, AIR, 0, lake_y, 0));
    // Never freezes beneath ice, beneath another water block, or beneath solid ground.
    assert!(!should_freeze_at(&air, &gen, WATER, ICE, 0, lake_y, 0), "water under ice stays liquid");
    assert!(!should_freeze_at(&air, &gen, WATER, WATER, 0, lake_y, 0), "deeper lake layers stay liquid");
    assert!(!should_freeze_at(&air, &gen, WATER, STONE, 0, lake_y, 0), "covered water stays liquid");
    // Slightly lowered ponds (`FLOW_7`) can refreeze by borrowing 1/8 from a neighbour; thinner flow cannot.
    assert!(should_freeze_at(&air, &gen, FLOW_7, AIR, 0, lake_y, 0));
    assert!(!should_freeze_at(&air, &gen, crate::block::flow(6), AIR, 0, lake_y, 0));
    // Sea stays open in mild frost and freezes only in deep cold.
    assert!(!can_freeze(FREEZE_TEMP, SEA_LEVEL));
    assert!(can_freeze(SEA_FREEZE_TEMP, SEA_LEVEL));
}

#[test]
fn freeze_and_melt_thresholds_never_overlap() {
    for temp in -2000..=2000 {
        for y in [SEA_LEVEL - 5, SEA_LEVEL, SEA_LEVEL + 1, SEA_LEVEL + 50] {
            assert!(!(can_freeze(temp, y) && can_melt(temp)), "temp {temp} y {y} both freezes and melts");
        }
    }
    assert!(!can_melt(0));
    assert!(can_melt(1));
}

#[test]
fn inventory_melting_follows_exponential_half_life() {
    let mut rng = Rng::new(42);
    let mut left = 1000u32;
    let half = INVENTORY_MELT_HALF_LIFE as u32;
    for _ in 0..half {
        left -= melt_inventory_count(left, INVENTORY_MELT_HALF_LIFE, 1.0, &mut rng);
    }
    assert!((440..=560).contains(&left), "after one half-life ~half should remain, got {left}");
    for _ in 0..half {
        left -= melt_inventory_count(left, INVENTORY_MELT_HALF_LIFE, 1.0, &mut rng);
    }
    assert!((200..=300).contains(&left), "after two half-lives ~a quarter should remain, got {left}");
}
