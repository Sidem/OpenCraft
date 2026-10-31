use super::*;
use crate::block::{BELT, CONSTRUCTOR, IRON_ORE, LIFT, RAMP_UP, SPLITTER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::links::Link;
use crate::factory::tests::{powered, recipe_for, round_trip, run, stocked_box};
use crate::factory::{Factory, Machine};
use crate::item::{IRON_INGOT, IRON_PLATE};
use crate::math::IVec3;
use crate::world::World;

const NORTH: u8 = 0;
const EAST: u8 = 1;
const SOUTH: u8 = 2;
const WEST: u8 = 3;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// Places `block` at `pos` the way a player would, with `facing` (a belt's carries its bend).
fn place(f: &mut Factory, block: crate::block::BlockId, pos: IVec3, facing: u8) {
    f.place(&mut World::new(1, 2), block, pos, facing, pos - v(0, 1, 0), 0);
}

#[test]
fn the_entry_side_turns_with_the_belt() {
    let mut b = Belt::new(v(0, 0, 0), EAST, Shape::Flat, 0);
    assert_eq!(b.entry_side(), WEST, "straight: the back");
    b.bend = BEND_LEFT;
    assert_eq!(b.entry_side(), NORTH, "heading east, the left is north");
    b.bend = BEND_RIGHT;
    assert_eq!(b.entry_side(), SOUTH);
    for dir in 0..4 {
        for bend in 0..BENDS {
            let facing = belt_facing(dir, bend);
            assert_eq!((facing % 4, bend_of(facing)), (dir, bend));
            assert_eq!(clean_facing(BELT, facing), facing, "a flat belt keeps its bend");
            assert_eq!(clean_facing(LIFT, facing), dir, "a lift has none");
        }
    }
    assert_eq!(bend_of(belt_facing(1, 3)), BEND_STRAIGHT, "no fourth bend");
}

#[test]
fn a_corner_carries_items_round_a_quarter_circle() {
    let mut b = Belt::new(v(0, 0, 0), SOUTH, Shape::Flat, 0);
    b.curve_from = Some(WEST);
    let near = |(x, z): (f32, f32), (a, c): (f32, f32)| (x - a).abs() < 1e-5 && (z - c).abs() < 1e-5;
    assert!(near(b.curve_offset(WEST, 0.0), (-0.5, 0.0)), "in at the west edge");
    assert!(near(b.curve_offset(WEST, 1.0), (0.0, 0.5)), "out at the south edge");
    // Halfway round, half a block from the corner it turns about (south-west, at (-0.5, 0.5)).
    let (x, z) = b.curve_offset(WEST, 0.5);
    assert!(((x + 0.5).powi(2) + (z - 0.5).powi(2)).sqrt() - 0.5 < 1e-5 && x > -0.5 && z < 0.5);
    // Every piece faces the way items move there, for every heading and both turns.
    for dir in 0..4 {
        for side in [(dir + 1) % 4, (dir + 3) % 4] {
            let b = Belt::new(v(0, 0, 0), dir, Shape::Flat, 0);
            for (k, &(s, c)) in MIDS.iter().enumerate() {
                let phi = (k as f32 + 0.5) * std::f32::consts::FRAC_PI_2 / PIECES as f32;
                assert!((phi.sin() - s).abs() < 1e-5 && (phi.cos() - c).abs() < 1e-5);
                let (a, z) = (b.on_arc(side, 0.5, phi - 1e-3), b.on_arc(side, 0.5, phi + 1e-3));
                let yaw = b.tangent_yaw(side, phi);
                let (dx, dz) = (z.0 - a.0, z.1 - a.1);
                let len = (dx * dx + dz * dz).sqrt();
                assert!(
                    (dx / len - yaw.sin()).abs() < 1e-3 && (dz / len + yaw.cos()).abs() < 1e-3,
                    "dir {dir} side {side}"
                );
            }
        }
    }
    let mut out = Vec::new();
    b.model(&mut out, crate::math::Vec3::ZERO, 0.0);
    assert_eq!(out.len(), 6 * crate::factory::render::INSTANCE_FLOATS, "six boxes");
    assert!(!out.is_empty() && out.len() % crate::factory::render::INSTANCE_FLOATS == 0);
}

/// The playtester's case: a machine only feeds a belt that takes items in on the side facing it. A belt running past
/// it is left alone; bent towards it, the same belt takes its output and carries it along the machine.
#[test]
fn a_constructor_feeds_a_belt_bent_towards_it_but_not_one_running_past() {
    for bend in [BEND_STRAIGHT, BEND_LEFT] {
        let mut f = Factory::default();
        stocked_box(&mut f, v(0, 0, 0), IRON_INGOT, 11);
        f.add_belt(v(1, 0, 0), EAST);
        powered(&mut f);
        place(&mut f, CONSTRUCTOR, v(2, 0, 0), NORTH);
        assert!(f.set_recipe(v(2, 0, 0), Some(recipe_for(IRON_PLATE))).is_some());
        // Below the constructor (south of it), running east: its left side faces the constructor.
        place(&mut f, BELT, v(2, 0, 1), belt_facing(EAST, bend));
        f.add_storage(v(3, 0, 1));
        run(&mut f, 14.0, |_| {});
        let plates = f.storage_count_at(v(3, 0, 1), IRON_PLATE);
        if bend == BEND_STRAIGHT {
            assert_eq!(plates, 0, "a belt running past the machine takes nothing from it");
        } else {
            assert_eq!(plates, 5, "the bent belt takes every plate and carries it on");
            assert_eq!(f.belt_at(v(2, 0, 1)).curve_from, Some(NORTH));
        }
    }
}

/// A splitter's side outputs and a box feed bent belts too, and a bent belt's exit feeds a machine as any belt does.
#[test]
fn a_splitter_feeds_bent_belts_on_its_sides() {
    let mut f = Factory::default();
    stocked_box(&mut f, v(0, 0, 0), IRON_ORE.into(), 12);
    f.add_belt(v(1, 0, 0), EAST);
    place(&mut f, SPLITTER, v(2, 0, 0), EAST);
    // North of the splitter, running east: its right side faces south, the splitter. Ends in a box.
    place(&mut f, BELT, v(2, 0, -1), belt_facing(EAST, BEND_RIGHT));
    f.add_storage(v(3, 0, -1));
    // South of it, a belt running west past it: straight, it is not an output.
    place(&mut f, BELT, v(2, 0, 1), belt_facing(WEST, BEND_STRAIGHT));
    run(&mut f, 20.0, |_| {});
    assert_eq!(f.storage_count_at(v(3, 0, -1), IRON_ORE.into()), 12, "everything through the one output that takes");
    assert!(f.belt_at(v(2, 0, 1)).items.is_empty());
}

/// A bent belt can sit right before a ramp: it hands items onto the ramp's back like a straight belt, and never turns
/// into a ramp itself.
#[test]
fn a_bent_belt_feeds_a_ramp_and_stays_flat() {
    let mut f = Factory::default();
    stocked_box(&mut f, v(1, 0, -1), IRON_ORE.into(), 6);
    // Fed from the box north of it, heading east into an up ramp.
    place(&mut f, BELT, v(1, 0, 0), belt_facing(EAST, BEND_LEFT));
    place(&mut f, RAMP_UP, v(2, 0, 0), EAST);
    f.add_belt(v(3, 1, 0), EAST);
    f.add_storage(v(4, 1, 0));
    run(&mut f, 15.0, |_| {});
    assert_eq!(f.storage_count_at(v(4, 1, 0), IRON_ORE.into()), 6);
    // A plain belt with a belt one up ahead becomes an up ramp; bent, it stays flat.
    let mut g = Factory::default();
    place(&mut g, BELT, v(0, 0, 0), belt_facing(EAST, BEND_RIGHT));
    g.add_belt(v(1, 1, 0), EAST);
    g.relink();
    assert_eq!(g.belt_at(v(0, 0, 0)).shape, Shape::Flat);
    assert!(g.set_bend(v(0, 0, 0), BEND_STRAIGHT));
    g.relink();
    assert_eq!(g.belt_at(v(0, 0, 0)).shape, Shape::Up, "straight again, it climbs");
    assert!(g.set_bend(v(0, 0, 0), BEND_LEFT), "a ramp can be bent: it lies flat");
    assert_eq!(g.belt_at(v(0, 0, 0)).shape, Shape::Flat);
}

/// Belts joining a bent belt: onto its start only from its entry side; from its back they join its middle.
#[test]
fn belts_join_a_bent_belt_at_its_entry_or_its_middle() {
    let mut f = Factory::default();
    place(&mut f, BELT, v(0, 0, 0), belt_facing(EAST, BEND_LEFT));
    f.add_belt(v(0, 0, -1), SOUTH); // into its left side, the entry
    f.add_belt(v(-1, 0, 0), EAST); // into its back
    f.relink();
    assert!(matches!(f.belt_at(v(0, 0, -1)).out, Link::Belt { mid: false, .. }));
    assert!(matches!(f.belt_at(v(-1, 0, 0)).out, Link::Belt { mid: true, .. }));
}

#[test]
fn turning_keeps_the_bend_and_only_flat_belts_bend() {
    let mut f = Factory::default();
    place(&mut f, BELT, v(0, 0, 0), belt_facing(EAST, BEND_LEFT));
    assert!(f.rotate(v(0, 0, 0)));
    f.relink();
    let b = f.belt_at(v(0, 0, 0));
    assert_eq!((b.dir, b.bend, b.entry_side()), (SOUTH, BEND_LEFT, EAST), "the entry turns with it");
    place(&mut f, LIFT, v(5, 0, 0), EAST);
    assert!(!f.set_bend(v(5, 0, 0), BEND_LEFT), "a lift has no side to take items in from");
    assert!(!f.set_bend(v(0, 0, 0), BENDS));
    assert_eq!(f.bend_at(v(5, 0, 0)), None);
    assert_eq!(f.placed_as(v(0, 0, 0)).map(|p| p.0), Some(belt_facing(SOUTH, BEND_LEFT)), "what a blueprint copies");
}

#[test]
fn the_bend_is_saved_and_older_belts_load_straight() {
    let mut f = Factory::default();
    place(&mut f, BELT, v(0, 0, 0), belt_facing(NORTH, BEND_RIGHT));
    let g = round_trip(&f);
    assert_eq!(g.belt_at(v(0, 0, 0)).bend, BEND_RIGHT);

    // One belt's bytes: position (12), heading, shape, tier, then the bend.
    let mut w = ByteWriter::default();
    f.belt_at(v(0, 0, 0)).write_state(&mut w);
    assert_eq!(w.bytes[15], BEND_RIGHT, "the bend follows the tier");
    let mut old = w.bytes.clone();
    old.remove(15);
    let mut r = ByteReader::new(&old);
    r.version = 61;
    let b = Belt::read_state(&mut r).expect("a version 61 belt");
    assert_eq!((b.dir, b.bend), (NORTH, BEND_STRAIGHT));

    let mut bad = w.bytes.clone();
    bad[15] = BENDS;
    assert!(Belt::read_state(&mut ByteReader::new(&bad)).is_none(), "a bend that does not exist");
}
