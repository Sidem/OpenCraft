use super::*;
use crate::action::Action;
use crate::block::{AIR, STONE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::MAX_SLOTS;
use crate::math::Vec3;
use crate::player::Player;
use crate::save::SAVE_VERSION;
use crate::sim::{PlayerId, Sim};

const P: PlayerId = PlayerId(0);

fn inv(sim: &Sim) -> &Inventory {
    &sim.player(P).unwrap().inventory
}

fn fill(inv: &mut Inventory, upto: usize) {
    for s in inv.slots[..upto].iter_mut() {
        *s = Stack { item: STONE.into(), count: 1 };
    }
}

#[test]
fn every_piece_of_gear_is_a_single_item_for_one_slot() {
    for g in &GEAR {
        assert_eq!(crate::item::stack_size(g.item), 1, "{}", crate::item::name(g.item));
        assert!(g.slot < SLOTS && !g.blurb.is_empty());
        assert_eq!(GEAR.iter().filter(|o| o.item == g.item).count(), 1);
    }
    assert_eq!(GEAR.iter().map(|g| g.effect.pack).max(), Some(crate::inventory::MAX_SLOTS - INVENTORY_SLOTS));
}

#[test]
fn a_hauler_pack_opens_rows_and_stays_on_while_they_hold_things() {
    let mut inv = Inventory::default();
    assert_eq!(inv.capacity(), INVENTORY_SLOTS);
    inv.slots[0] = Stack { item: HAULER_PACK, count: 1 };
    assert!(inv.wear_from(0));
    assert_eq!((inv.worn[0], inv.capacity(), inv.slots[0].is_empty()), (HAULER_PACK, INVENTORY_SLOTS + 27, true));

    // The new rows take items, so the pack cannot come off.
    fill(&mut inv, INVENTORY_SLOTS + 1);
    assert_eq!(inv.add(STONE.into(), 64), 0, "room in the new row");
    inv.click_gear(0, true);
    assert_eq!(inv.worn[0], HAULER_PACK, "stays on while a stack sits in the rows it opens");
    // Emptying the rows lets it come off, into the first free slot.
    inv.slots[INVENTORY_SLOTS..].iter_mut().for_each(|s| *s = Stack::default());
    inv.slots[0] = Stack::default();
    inv.click_gear(0, true);
    assert_eq!((inv.worn[0], inv.slots[0].item, inv.capacity()), (ItemId::NONE, HAULER_PACK, INVENTORY_SLOTS));
}

#[test]
fn replacing_a_pack_puts_the_old_one_in_the_inventory() {
    let mut inv = Inventory::default();
    inv.worn[0] = HAULER_PACK;
    inv.slots[3] = Stack { item: HAULER_PACK_MK2, count: 1 };
    assert!(inv.wear_from(3));
    assert_eq!((inv.worn[0], inv.count(HAULER_PACK), inv.capacity()), (HAULER_PACK_MK2, 1, INVENTORY_SLOTS + 45));
    inv.slots.iter_mut().for_each(|s| *s = Stack::default());

    // Back to the smaller pack is refused while the big one's rows are in use, and nothing changes.
    inv.slots[INVENTORY_SLOTS + 30] = Stack { item: STONE.into(), count: 5 };
    inv.slots[3] = Stack { item: HAULER_PACK, count: 1 };
    assert!(!inv.wear_from(3));
    assert_eq!((inv.worn[0], inv.slots[3].item), (HAULER_PACK_MK2, HAULER_PACK));
    assert_eq!(inv.slots[INVENTORY_SLOTS + 30].count, 5);
}

#[test]
fn every_pack_tier_opens_eighteen_more_slots_than_the_one_below() {
    let slots = |pack: ItemId| gear(pack).unwrap().effect.pack;
    assert_eq!(PACKS.map(slots), [27, 45, 63, 81, 99]);
    assert_eq!(MAX_SLOTS, INVENTORY_SLOTS + slots(PACKS[4]));
    for w in PACKS.windows(2) {
        assert_eq!(slots(w[1]) - slots(w[0]), 18);
    }
}

#[test]
fn a_full_pack_is_raised_in_place_with_kits_and_only_when_unlocked() {
    use crate::item::{BLUE_KIT, GREEN_KIT};
    let mut inv = Inventory::default();
    inv.worn[0] = HAULER_PACK;
    fill(&mut inv, INVENTORY_SLOTS + 27);
    // Not even a free slot for the kits: take stacks out first, as a player would.
    inv.slots[0] = Stack { item: GREEN_KIT, count: PACK_KITS };
    assert!(!inv.upgrade_pack(|_| false), "the tech has not unlocked it");
    assert_eq!(inv.worn[0], HAULER_PACK);
    assert!(inv.upgrade_pack(|_| true));
    assert_eq!((inv.worn[0], inv.capacity(), inv.slots[0].is_empty()), (HAULER_PACK_MK2, INVENTORY_SLOTS + 45, true));
    assert!(inv.slots[INVENTORY_SLOTS..INVENTORY_SLOTS + 27].iter().all(|s| !s.is_empty()), "nothing moved");

    // The next step wants kits of the next colour, and as many as the recipe says.
    inv.slots[0] = Stack { item: GREEN_KIT, count: PACK_KITS };
    assert!(!inv.upgrade_pack(|_| true), "green kits do not make a Mk3");
    inv.slots[0] = Stack { item: BLUE_KIT, count: PACK_KITS - 1 };
    assert!(!inv.upgrade_pack(|_| true), "one kit short");
    inv.slots[0].count = PACK_KITS;
    assert!(inv.upgrade_pack(|_| true));
    assert_eq!(inv.worn[0], HAULER_PACK_MK3);

    // The last tier, and no pack at all, have nothing to raise.
    inv.worn[0] = HAULER_PACK_MK5;
    assert!(!inv.upgrade_pack(|_| true));
    inv.worn[0] = ItemId::NONE;
    assert!(!inv.upgrade_pack(|_| true));
}

#[test]
fn pack_recipes_are_the_pack_below_plus_the_kits_of_the_upgrade() {
    for w in PACKS.windows(2) {
        let (next, kit, kits) = next_pack(w[0]).unwrap();
        assert_eq!(next, w[1]);
        let recipe = crate::recipes::RECIPES.iter().find(|r| r.output == next).expect("every pack is craftable");
        assert_eq!((recipe.count, recipe.inputs), (1, &[(w[0], 1), (kit, kits)][..]), "{}", crate::item::name(next));
    }
    assert_eq!(next_pack(PACKS[4]), None);
}

#[test]
fn saves_from_before_the_bigger_packs_read_their_eighteen_rows() {
    let mut w = ByteWriter::default();
    let none = Stack::default();
    let stone = Stack { item: STONE.into(), count: 9 };
    (0..INVENTORY_SLOTS).for_each(|i| if i == 5 { stone } else { none }.write_state(&mut w));
    none.write_state(&mut w);
    w.u8(0);
    (0..18).for_each(|i| if i == 17 { stone } else { none }.write_state(&mut w));
    (0..LEGACY_SLOTS).for_each(|i| w.item(if i == 0 { HAULER_PACK_MK2 } else { ItemId::NONE }));
    let mut r = ByteReader::new(&w.bytes);
    r.version = 41;
    let back = Inventory::read_state(&mut r).expect("reads");
    assert_eq!((back.slots[5].count, back.slots[INVENTORY_SLOTS + 17].count), (9, 9));
    assert_eq!((back.worn[0], r.u8().is_none()), (HAULER_PACK_MK2, true), "all of it was read, and no more");
}

#[test]
fn the_jetpack_works_only_while_worn_and_an_old_pack_jetpack_goes_on_when_a_save_loads() {
    let mut inv = Inventory::default();
    inv.add(JETPACK, 1);
    assert!(!inv.has_jetpack(), "in the pack it does nothing");
    assert!(inv.wear_from(0) && inv.has_jetpack() && inv.count(JETPACK) == 0);
    inv.click_gear(4, true);
    assert!(!inv.has_jetpack() && inv.count(JETPACK) == 1, "Shift takes it off into the pack");

    let mut w = ByteWriter::default();
    let none = Stack::default();
    (0..INVENTORY_SLOTS)
        .for_each(|i| if i == 2 { Stack { item: JETPACK, count: 1 } } else { none }.write_state(&mut w));
    none.write_state(&mut w);
    w.u8(0);
    (0..18).for_each(|_| none.write_state(&mut w));
    (0..LEGACY_SLOTS).for_each(|_| w.item(ItemId::NONE));
    let mut r = ByteReader::new(&w.bytes);
    r.version = 41;
    let back = Inventory::read_state(&mut r).expect("reads");
    assert!(back.has_jetpack() && back.count(JETPACK) == 0, "worn after loading an older save");
}

#[test]
fn the_cursor_swaps_gear_and_only_the_right_gear_fits() {
    let mut inv = Inventory { cursor: Stack { item: MINING_RIG, count: 1 }, ..Default::default() };
    inv.click_gear(1, false);
    assert_eq!((inv.worn[1], inv.cursor.item), (ItemId::NONE, MINING_RIG), "a mining rig is not boots");
    inv.click_gear(3, false);
    assert_eq!((inv.worn[3], inv.cursor.is_empty()), (MINING_RIG, true));
    inv.click_gear(3, false);
    assert_eq!((inv.worn[3], inv.cursor.item), (ItemId::NONE, MINING_RIG), "an empty cursor takes it off");

    inv.cursor = Stack { item: SPRING_BOOTS, count: 1 };
    inv.click_gear(1, false);
    inv.cursor = Stack { item: SERVO_BOOTS, count: 1 };
    inv.click_gear(1, false);
    assert_eq!((inv.worn[1], inv.cursor.item), (SERVO_BOOTS, SPRING_BOOTS), "boots swap");
    inv.cursor = Stack { item: STONE.into(), count: 1 };
    inv.click_gear(1, false);
    assert_eq!(inv.worn[1], SERVO_BOOTS, "stone is no gear");
}

#[test]
fn gear_rides_the_actions_and_the_save() {
    let mut sim = Sim::new(7, 2);
    sim.apply(P, Action::Give { item: HAULER_PACK, count: 1 });
    sim.apply(P, Action::Give { item: SPRING_BOOTS, count: 1 });
    for slot in 0..2 {
        sim.apply(P, Action::ClickSlot { slot, shift: true });
    }
    assert_eq!((inv(&sim).worn[0], inv(&sim).worn[1]), (HAULER_PACK, SPRING_BOOTS));
    assert_eq!(inv(&sim).count(HAULER_PACK), 0, "worn gear is not in the pack");
    sim.apply(P, Action::Give { item: STONE.into(), count: 64 * 40 });
    assert!(inv(&sim).slots[INVENTORY_SLOTS..].iter().any(|s| !s.is_empty()), "the pack's rows fill up");

    let mut w = ByteWriter::default();
    inv(&sim).write_state(&mut w);
    let back = Inventory::read_state(&mut ByteReader::new(&w.bytes)).expect("reads back");
    assert_eq!(back.worn, inv(&sim).worn);
    assert_eq!(back.slots, inv(&sim).slots);
    assert_eq!(
        SAVE_VERSION, 43,
        "the layout above is unchanged since version 29 (30 to 43: rails, trains, hover pack, cargo, game mode, tools on belts, fibre nodes, research queue; 42 made the pack rows 99, read back as 18 before; 43 added the jetpack slot, four slots before)"
    );

    // A save with a pack's rows full but no pack on is refused.
    let mut bad = inv(&sim).clone();
    bad.worn = [ItemId::NONE; SLOTS];
    let mut w = ByteWriter::default();
    bad.write_state(&mut w);
    assert!(Inventory::read_state(&mut ByteReader::new(&w.bytes)).is_none());
}

#[test]
fn old_saves_read_without_gear() {
    let mut inv = Inventory::default();
    inv.slots[2] = Stack { item: STONE.into(), count: 7 };
    let mut w = ByteWriter::default();
    inv.write_state(&mut w);
    // Version 28 wrote the 36 slots, the cursor and the selection, and nothing more.
    let old_len = w.bytes.len() - (MAX_SLOTS - INVENTORY_SLOTS) * 5 - SLOTS * 2;
    let mut r = ByteReader::new(&w.bytes[..old_len]);
    r.version = 28;
    let back = Inventory::read_state(&mut r).expect("reads");
    assert_eq!((back.slots[2].count, back.worn), (7, [ItemId::NONE; SLOTS]));
}

#[test]
fn boots_and_frames_scale_the_body_and_a_rig_the_hands() {
    let mut inv = Inventory::default();
    assert_eq!((inv.boost(), inv.mining_speed()), (Default::default(), 1.0));
    inv.worn = [ItemId::NONE, SERVO_BOOTS, EXO_FRAME, MINING_RIG, ItemId::NONE];
    let b = inv.boost();
    assert!((b.walk - 1.15).abs() < 1e-9 && (b.sprint - 1.15 * 1.1).abs() < 1e-9 && b.jump == 1.0);
    assert_eq!(inv.mining_speed(), 1.5);
}

#[test]
fn spring_boots_jump_two_blocks_and_servo_boots_walk_faster() {
    let flat = |_x: i32, y: i32, _z: i32| y < 10;
    let jump_height = |boost| {
        let mut p = Player::new(Vec3::new(0.5, 10.0, 0.5));
        p.boost = boost;
        (0..24).for_each(|_| p.step(1.0 / 120.0, &mut |x, y, z| flat(x, y, z), &mut |_, _, _| AIR));
        p.input.jump = true;
        let mut apex: f64 = 0.0;
        for _ in 0..120 {
            p.step(1.0 / 120.0, &mut |x, y, z| flat(x, y, z), &mut |_, _, _| AIR);
            apex = apex.max(p.pos.y - 10.0);
        }
        apex
    };
    let mut inv = Inventory::default();
    let bare = jump_height(inv.boost());
    inv.worn[1] = SPRING_BOOTS;
    let sprung = jump_height(inv.boost());
    assert!(bare < 1.45 && sprung > 1.9 && sprung < 2.2, "bare {bare}, sprung {sprung}");

    let walked = |boost| {
        let mut p = Player::new(Vec3::new(0.5, 10.0, 0.5));
        p.boost = boost;
        p.input.forward = 1.0;
        (0..240).for_each(|_| p.step(1.0 / 120.0, &mut |x, y, z| flat(x, y, z), &mut |_, _, _| AIR));
        -(p.pos.z - 0.5)
    };
    inv.worn[1] = SERVO_BOOTS;
    let (slow, fast) = (walked(Default::default()), walked(inv.boost()));
    assert!(fast > slow * 1.1 && fast < slow * 1.2, "{slow} vs {fast}");
}

#[test]
fn worn_gear_reaches_the_body_and_the_hands_in_a_running_game() {
    let mut g = crate::Game::new(2024, 3);
    crate::tests::run_until_ready(&mut g);
    g.act(Action::Give { item: SERVO_BOOTS, count: 1 });
    g.act(Action::Give { item: MINING_RIG, count: 1 });
    for slot in 0..2 {
        g.act(Action::ClickSlot { slot, shift: true });
    }
    for _ in 0..30 {
        g.update(1.0 / 60.0);
    }
    assert_eq!(g.inventory().worn[1], SERVO_BOOTS);
    assert!((g.body().boost.walk - 1.15).abs() < 1e-9, "the body walks faster");
    assert_eq!(g.inventory().mining_speed(), 1.5);
}
