use super::*;
use crate::action::Action;
use crate::item::{CIRCUIT, MOTOR};
use crate::sim::SimEvent;

const P: PlayerId = PlayerId(0);
const DOOR: IVec3 = IVec3::new(300, 200, -300);

/// The digits whose square ends in …444444 (one of the two roots of -1), found the way a player would: ring by ring.
fn solution() -> [u8; RINGS] {
    let mut digits = [0u8; RINGS];
    for ring in 0..RINGS {
        let fits: Vec<u8> = (0..5)
            .filter(|&d| {
                digits[ring] = d;
                lit_rings(pack(&digits)) > ring
            })
            .collect();
        assert!(fits.len() == if ring == 0 { 2 } else { 1 }, "ring {ring}: {fits:?}");
        digits[ring] = fits[0];
    }
    digits
}

fn door_sim() -> Sim {
    let mut sim = Sim::new(7, 2);
    assert!(sim.world.set_block_anywhere(DOOR, VAULT_DOOR));
    assert!(sim.world.set_block_anywhere(DOOR + IVec3::new(0, 1, 0), VAULT_DOOR));
    sim
}

fn dropped(sim: &Sim) -> Vec<(ItemId, u32)> {
    sim.events
        .iter()
        .filter_map(|e| match e {
            SimEvent::Dropped { item, count, .. } => Some((*item, *count)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_first_ring_takes_two_digits_and_every_ring_after_exactly_one() {
    let digits = solution();
    assert_eq!(&digits[..3], &[2, 1, 2], "2, then 12, then 212 in base 5 (read from the right)");
    assert_eq!(lit_rings(pack(&digits)), RINGS);
    // The other root of -1 is the number's negative.
    let top = 5u32.pow(RINGS as u32);
    assert_eq!(lit_rings(top - pack(&digits)), RINGS);
    assert_eq!((0..top).filter(|&n| lit_rings(n) == RINGS).count(), 2, "no other number opens it");
}

#[test]
fn rings_light_in_order_and_the_square_is_shown_digit_by_digit() {
    assert_eq!(lit_rings(0), 0);
    assert_eq!(lit_rings(2), 1, "2 x 2 = 4 ends in a 4");
    assert_eq!(square_digits(2), [4, 0, 0, 0, 0, 0]);
    assert_eq!(lit_rings(pack(&[2, 1])), 2, "12 in base 5 is 7 and 49 ends in 44");
    assert_eq!(square_digits(pack(&[2, 1]))[..2], [4, 4]);
    assert_eq!(pack(&[4, 4, 4, 4, 4, 4]), 5u32.pow(6) - 1);
    assert_eq!(pack(&[9, 0]), 4, "a digit past 4 is held at 4");
}

#[test]
fn the_right_dials_open_the_door_and_drop_the_loot() {
    let mut sim = door_sim();
    sim.events.clear();
    sim.apply(P, Action::OpenVault { pos: DOOR, dials: pack(&solution()) });
    assert_eq!(sim.world.block_anywhere(DOOR), Some(AIR));
    assert_eq!(sim.world.block_anywhere(DOOR + IVec3::new(0, 1, 0)), Some(AIR), "both blocks of the door go");
    let loot = dropped(&sim);
    assert!(loot.contains(&(CIRCUIT, 5)) && loot.contains(&(MOTOR, 1)), "{loot:?}");
    assert_eq!(loot.len(), LOOT.len());
}

#[test]
fn the_door_opens_from_either_block_of_it() {
    let mut sim = door_sim();
    sim.apply(P, Action::OpenVault { pos: DOOR + IVec3::new(0, 1, 0), dials: pack(&solution()) });
    assert_eq!(sim.world.block_anywhere(DOOR), Some(AIR));
}

#[test]
fn wrong_dials_a_missing_door_and_a_second_try_do_nothing() {
    let mut sim = door_sim();
    sim.events.clear();
    let mut nearly = solution();
    nearly[RINGS - 1] = (nearly[RINGS - 1] + 1) % 5;
    sim.apply(P, Action::OpenVault { pos: DOOR, dials: pack(&nearly) });
    sim.apply(P, Action::OpenVault { pos: DOOR, dials: 0 });
    assert_eq!(sim.world.block_anywhere(DOOR), Some(VAULT_DOOR));
    assert!(dropped(&sim).is_empty());
    // Not a door: the right dials leave other blocks alone.
    let stone = DOOR + IVec3::new(5, 0, 0);
    assert!(sim.world.set_block_anywhere(stone, crate::block::STONE));
    sim.apply(P, Action::OpenVault { pos: stone, dials: pack(&solution()) });
    assert_eq!(sim.world.block_anywhere(stone), Some(crate::block::STONE));
    assert!(dropped(&sim).is_empty());
    // Once open, turning the rings again gives nothing more.
    sim.apply(P, Action::OpenVault { pos: DOOR, dials: pack(&solution()) });
    let first = dropped(&sim).len();
    sim.apply(P, Action::OpenVault { pos: DOOR, dials: pack(&solution()) });
    assert_eq!(dropped(&sim).len(), first);
}

#[test]
fn the_door_cannot_be_broken_by_hand() {
    let mut sim = door_sim();
    sim.apply(P, Action::BreakBlock { pos: DOOR });
    assert_eq!(sim.world.block_anywhere(DOOR), Some(VAULT_DOOR));
}
