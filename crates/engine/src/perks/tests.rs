use super::*;
use crate::equipment::GLOVES_TIERS;
use crate::inventory::Stack;
use crate::item::{CRAFT_GLOVES, WORK_GLOVES};

#[test]
fn work_gloves_have_five_tiers_adding_a_quarter_each() {
    let r = Research::default();
    assert_eq!(GLOVES_TIERS.len(), 5);
    for (i, &gloves) in GLOVES_TIERS.iter().enumerate() {
        let mut inv = Inventory::default();
        inv.add(gloves, 1);
        assert_eq!(player_bonus(&r, &inv, Stat::Crafting), 1000, "in the pack they do nothing");
        assert!(inv.wear_from(0));
        assert_eq!(player_bonus(&r, &inv, Stat::Crafting), 1000 + 250 * (i as u32 + 1));
    }
}

#[test]
fn taking_off_worn_gloves_restores_base_speed() {
    let mut inv = Inventory::default();
    let r = Research::default();
    inv.add(WORK_GLOVES, 1);
    assert!(inv.wear_from(0));
    assert_eq!(player_bonus(&r, &inv, Stat::Crafting), 1250);
    inv.click_gear(5, true);
    assert_eq!((inv.cursor, inv.count(WORK_GLOVES)), (Stack::default(), 1));
    assert_eq!(player_bonus(&r, &inv, Stat::Crafting), 1000);
    inv.add(CRAFT_GLOVES, 1);
    let slot = (0..inv.slots.len()).find(|&s| inv.slots[s].item == CRAFT_GLOVES).unwrap();
    assert!(inv.wear_from(slot));
    assert_eq!(player_bonus(&r, &inv, Stat::Crafting), 2250);
}
