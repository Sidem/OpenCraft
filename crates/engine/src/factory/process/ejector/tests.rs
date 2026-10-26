//! The belt ejector: what it takes, how fast and which way it drops, that a belt backs up behind it, that a tool goes in
//! one piece and that what waits in it is saved.

use crate::block::{BELT, DIRT, STONE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::provenance::Tag;
use crate::factory::{Factory, Machine};
use crate::item::{ItemId, STEEL_PICKAXE};
use crate::math::IVec3;
use crate::sim::SimEvent;
use crate::world::World;

use super::*;

const EAST: u8 = 1;

fn rig() -> Factory {
    let mut f = Factory::default();
    f.place(&mut World::new(1, 2), BELT_EJECTOR, IVec3::ZERO, EAST, IVec3::ZERO, 0);
    f
}

/// Runs `seconds`, topping the ejector up each tick when `feed` is set; returns every drop (position, velocity, item, count).
fn run(f: &mut Factory, seconds: f64, feed: Option<ItemId>) -> Vec<(Vec3, Vec3, ItemId, u32)> {
    let mut world = World::new(1, 2);
    let (mut events, mut drops) = (Vec::new(), Vec::new());
    for tick in 0..(seconds * crate::TICK_RATE as f64) as u64 {
        if let Some(item) = feed {
            f.insert(IVec3::ZERO, item, 4);
        }
        f.update(&mut world, tick, &mut events);
        for e in events.drain(..) {
            if let SimEvent::Dropped { pos, vel, item, count } = e {
                drops.push((pos, vel, item, count));
            }
        }
    }
    drops
}

fn ejector(f: &Factory) -> &Processor {
    f.processors.iter().find(|p| p.spec.pick == Pick::Eject).unwrap()
}

#[test]
fn it_takes_a_few_of_anything_from_any_side() {
    let mut f = rig();
    assert_eq!(f.insert(IVec3::ZERO, DIRT.into(), 10), 4, "four at a time");
    assert_eq!(f.insert(IVec3::ZERO, STONE.into(), 10), 0, "the room is shared by every kind");
    assert!(EJECTOR_SPEC.footprint.is_single(), "belts may point into any side");
}

#[test]
fn it_drops_ten_a_second_in_front_of_the_way_it_faces() {
    let mut f = rig();
    let drops = run(&mut f, 10.0, Some(DIRT.into()));
    assert!((99..=101).contains(&drops.len()), "ten a second, not the belt's pace: {}", drops.len());
    let (pos, vel, item, count) = drops[0];
    assert_eq!((item, count), (DIRT.into(), 1));
    assert!(pos.x > 0.5 && (pos.z - 0.5).abs() < 1e-9, "forward is east: {pos:?}");
    assert!(pos.y >= 1.0 + 0.125, "starts above the block, clear of it: {pos:?}");
    assert!(vel.x > 0.0 && vel.z == 0.0 && vel.y > 0.0, "thrown forward and up: {vel:?}");
    assert!(ejector(&f).input.total() >= 3, "it never holds more than its room, and is kept topped up");
}

#[test]
fn it_drops_the_way_its_placer_faced_whichever_that_is() {
    for dir in 0..4u8 {
        let mut f = Factory::default();
        f.place(&mut World::new(1, 2), BELT_EJECTOR, IVec3::ZERO, dir, IVec3::ZERO, 0);
        f.insert(IVec3::ZERO, DIRT.into(), 1);
        let (pos, vel, ..) = run(&mut f, 1.0, None)[0];
        let want = crate::factory::DIRS[dir as usize];
        assert_eq!(
            (vel.x.signum() * (vel.x != 0.0) as i32 as f64, vel.z.signum() * (vel.z != 0.0) as i32 as f64),
            (f64::from(want.x), f64::from(want.z))
        );
        assert!(((pos.x - 0.5) * f64::from(want.x) + (pos.z - 0.5) * f64::from(want.z)) > 0.0);
    }
}

#[test]
fn a_belt_ending_in_it_is_emptied_at_its_pace_and_backs_up_beyond() {
    let mut f = rig();
    let mut world = World::new(1, 2);
    for x in -4..0 {
        f.place(&mut world, BELT, IVec3::new(x, 0, 0), EAST, IVec3::new(x, 0, 0), 3);
    }
    let mut events = Vec::new();
    let mut dropped = 0;
    for tick in 0..30 * crate::TICK_RATE as u64 {
        for belt in f.belts.iter_mut() {
            // A Mk4 belt full of dirt: top up the first cell whenever it has room.
            if belt.pos.x == -4 {
                belt.accept(DIRT.into(), 1, Tag::UNKNOWN, false, 0.0);
            }
        }
        f.update(&mut world, tick, &mut events);
        dropped += events.drain(..).filter(|e| matches!(e, SimEvent::Dropped { .. })).count();
    }
    assert!((295..=305).contains(&dropped), "ten a second for 30 s, however fast the belt: {dropped}");
    let held: usize = f.belts.iter().map(|b| b.items.len()).sum();
    assert!(held >= 4 * 2, "the belt is full behind it: {held}");
}

#[test]
fn its_readout_says_what_it_does_and_offers_no_panel() {
    let mut f = rig();
    assert!(f.describe(IVec3::ZERO).unwrap().starts_with("Waiting for a belt"));
    f.insert(IVec3::ZERO, DIRT.into(), 4);
    run(&mut f, 0.2, None);
    let text = f.describe(IVec3::ZERO).unwrap();
    assert!(text.starts_with("Dropping items") && !text.contains("Right-click"), "{text}");
}

#[test]
fn a_tool_goes_in_one_piece_with_its_uses() {
    let mut f = rig();
    let slot = &mut f.processors[0].input.slots[0];
    *slot = Stack { item: STEEL_PICKAXE, count: 87 };
    let drops = run(&mut f, 1.0, None);
    assert_eq!(drops.iter().map(|d| (d.2, d.3)).collect::<Vec<_>>(), vec![(STEEL_PICKAXE, 87)]);
}

#[test]
fn it_counts_what_it_drops_as_consumed_and_saves_what_waits() {
    let mut f = rig();
    f.insert(IVec3::ZERO, DIRT.into(), 4);
    let mut world = World::new(1, 2);
    let mut events = Vec::new();
    for tick in 0..8 {
        f.update(&mut world, tick, &mut events);
    }
    let consumed = events.iter().filter(|e| matches!(e, SimEvent::Consumed { .. })).count();
    assert_eq!(consumed, 1, "the first drop is due after 6 ticks");
    let mut w = ByteWriter::default();
    ejector(&f).write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    let back = Processor::read_state(&mut r).unwrap();
    assert_eq!(back.input.total(), 3);
    assert_eq!(r.u8(), None, "every byte read");
    assert_eq!(ejector(&f).contents().iter().map(|s| s.count).sum::<u32>(), 3, "breaking it gives the rest back");
}
