use super::*;
use crate::block::{AIR, DIRT, POLE};
use crate::factory::pole::Pole;
use crate::factory::Machine;
use crate::sim::{PlayerId, Sim};

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// A world of the given solid cells only.
fn solids(cells: &[IVec3]) -> impl FnMut(IVec3) -> bool + '_ {
    move |p| cells.contains(&p)
}

#[test]
fn ground_wins_a_wall_only_hangs_a_pole_with_nothing_under_it() {
    let pos = v(5, 20, 5);
    let ground = [pos - UP, pos + SIDES[0]];
    assert_eq!(wall_for(solids(&ground), pos, pos - UP), 0, "standing on the ground, wall or not");
    let wall = [pos + SIDES[3]];
    assert_eq!(wall_for(solids(&wall), pos, pos - UP), 4, "north side");
    assert_eq!(wall_for(solids(&[]), pos, pos - UP), 0);
}

#[test]
fn the_wall_that_was_clicked_is_chosen_among_several() {
    let pos = v(5, 20, 5);
    let walls = [pos + SIDES[0], pos + SIDES[2]];
    assert_eq!(wall_for(solids(&walls), pos, pos), 1, "the first side without a click");
    assert_eq!(wall_for(solids(&walls), pos, pos + SIDES[2]), 3, "the clicked one");
    assert_eq!(wall_for(solids(&walls), pos, pos + SIDES[1]), 1, "a clicked side with nothing solid is ignored");
}

#[test]
fn open_air_holds_no_pole() {
    let pos = v(5, 20, 5);
    assert!(!is_held_up(solids(&[]), pos));
    assert!(!is_held_up(solids(&[pos + UP]), pos), "a ceiling alone does not hold it");
    assert!(is_held_up(solids(&[pos - UP]), pos));
    assert!(SIDES.iter().all(|&s| is_held_up(solids(&[pos + s]), pos)), "any of the four walls");
}

#[test]
fn a_mounted_pole_draws_a_plate_and_struts_a_standing_one_a_foot() {
    let draw = |wall| {
        let mut out = Vec::new();
        Pole { pos: IVec3::ZERO, tier: 0, wall }.model(&mut out, Vec3::ZERO, 0.0);
        out.len() / crate::factory::INSTANCE_FLOATS
    };
    assert_eq!(draw(1), draw(0) + 2, "plate and two struts instead of the foot");
    assert!((1..=4).all(|w| draw(w) == draw(1)));
}

#[test]
fn a_wall_survives_a_save_and_a_tier_does_not_swallow_it() {
    use crate::bytes::{ByteReader, ByteWriter};
    for (tier, wall) in [(0, 0), (0, 4), (3, 2), (CABLE_TIER_FOR_TEST, 0)] {
        let mut w = ByteWriter::default();
        Pole { pos: v(1, 2, 3), tier, wall }.write_state(&mut w);
        let mut r = ByteReader::new(&w.bytes);
        r.version = crate::save::SAVE_VERSION;
        let p = Pole::read_state(&mut r).expect("reads back");
        assert_eq!((p.tier, p.wall), (tier, wall));
    }
    // A wall number no side has is a damaged save, not a pole.
    let mut w = ByteWriter::default();
    w.ivec3(v(0, 0, 0));
    w.u8(5 << 4);
    let mut r = ByteReader::new(&w.bytes);
    r.version = crate::save::SAVE_VERSION;
    assert!(Pole::read_state(&mut r).is_none());
}

const CABLE_TIER_FOR_TEST: u8 = crate::factory::pole::CABLE_TIER;

#[test]
fn the_core_refuses_a_pole_in_open_air_and_hangs_one_on_a_wall() {
    let me = PlayerId(0);
    let mut sim = Sim::new(7, 2);
    let air = v(500, 200, -500);
    sim.apply(me, crate::action::Action::Give { item: POLE.into(), count: 3 });
    let place = |sim: &mut Sim, pos, against| {
        sim.apply(me, crate::action::Action::PlaceBlock { pos, slot: 0, facing: 0, against });
        sim.world.block_anywhere_or_generate(pos) == POLE
    };
    assert!(!place(&mut sim, air, air - UP), "nothing solid near");
    assert_eq!(sim.world.block_anywhere_or_generate(air), AIR);
    assert_eq!(sim.factory.poles().count(), 0);

    sim.world.set_block_anywhere(air + SIDES[1], DIRT);
    assert!(place(&mut sim, air, air + SIDES[1]), "a wall to the west holds it");
    let pole = sim.factory.pole_wall(air);
    assert_eq!(pole, Some(2), "mounted on the west side");

    sim.world.set_block_anywhere(air + v(10, -1, 0), DIRT);
    assert!(place(&mut sim, air + v(10, 0, 0), air + v(10, -1, 0)));
    assert_eq!(sim.factory.pole_wall(air + v(10, 0, 0)), Some(0), "on the ground it stands as before");
}
