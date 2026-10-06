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
    assert_eq!((inv.worn[0], inv.capacity(), inv.slots[0].is_empty()), (HAULER_PACK, INVENTORY_SLOTS + 9, true));

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
    assert_eq!((inv.worn[0], inv.count(HAULER_PACK), inv.capacity()), (HAULER_PACK_MK2, 1, INVENTORY_SLOTS + 18));
    inv.slots.iter_mut().for_each(|s| *s = Stack::default());

    // Back to the smaller pack is refused while the big one's rows are in use, and nothing changes.
    inv.slots[INVENTORY_SLOTS + 12] = Stack { item: STONE.into(), count: 5 };
    inv.slots[3] = Stack { item: HAULER_PACK, count: 1 };
    assert!(!inv.wear_from(3));
    assert_eq!((inv.worn[0], inv.slots[3].item), (HAULER_PACK_MK2, HAULER_PACK));
    assert_eq!(inv.slots[INVENTORY_SLOTS + 12].count, 5);
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
        SAVE_VERSION, 38,
        "the layout above is unchanged since version 29 (30 to 38: rails, trains, hover pack, cargo, game mode)"
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
    inv.worn = [ItemId::NONE, SERVO_BOOTS, EXO_FRAME, MINING_RIG];
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
