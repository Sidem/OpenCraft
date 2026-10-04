use super::super::Kind;
use super::*;
use crate::block::{BELT, GENERATOR, POLE, SENSOR, STORAGE};
use crate::item::IRON_PLATE;
use crate::world::World;

fn put(f: &mut Factory, block: BlockId, pos: IVec3, dir: u8) {
    f.place(&mut World::new(1, 2), block, pos, dir, pos, 0);
}

fn tick(f: &mut Factory) {
    f.update(&mut World::new(1, 2), 1, &mut Vec::new());
}

fn at(x: i32) -> IVec3 {
    IVec3::new(x, 3, 0)
}

/// A box at x 0, a sensor at 1 facing east, a generator at 2 wired to a pole at 4.
fn line() -> Factory {
    let mut f = Factory { by_hand: true, ..Factory::default() };
    put(&mut f, STORAGE, at(0), 0);
    put(&mut f, SENSOR, at(1), 1);
    put(&mut f, GENERATOR, at(2), 0);
    put(&mut f, POLE, at(4), 0);
    f.connect(at(4), at(2));
    tick(&mut f);
    f
}

fn fill(f: &mut Factory, plates: u32) {
    let Some(&Slot::Storage(i)) = f.at.get(&at(0)) else { panic!("no box") };
    f.storages[i as usize].buf = super::super::buffer::Buffer::new(24);
    f.storages[i as usize].buf.add(IRON_PLATE, plates);
}

#[test]
fn rules_flip_only_at_the_far_threshold() {
    let mut s = Sensor::new(at(0), 0);
    s.rule = 3; // on at 75 or less, off at 95 or more
    assert!(s.decide(Some(0)) && s.decide(None), "empty or nothing to read: on");
    assert!(!s.decide(Some(96)));
    s.on = false;
    assert!(!s.decide(Some(80)), "between the thresholds it keeps its state");
    assert!(s.decide(Some(75)));
    s.rule = 4; // on at 60 or more, off at 40 or less
    s.on = true;
    assert!(s.decide(Some(50)) && !s.decide(Some(40)));
    s.on = false;
    assert!(!s.decide(Some(50)) && s.decide(Some(60)));
    s.rule = 1;
    assert!(!s.decide(None) && !s.decide(Some(0)), "always off");
    s.rule = 0;
    s.on = false;
    assert!(s.decide(Some(100)), "always on");
}

#[test]
fn a_full_box_switches_the_generator_off_and_a_drained_one_back_on() {
    let mut f = line();
    assert_eq!(f.power.gen_pole, [Some(0)], "wired and on");
    fill(&mut f, 24 * 64);
    tick(&mut f);
    assert!(!f.sensor(at(1)).unwrap().on);
    assert_eq!(f.power.gen_pole, [None], "full: the wire is cut");
    assert!(f.describe(at(2)).unwrap().contains(SWITCHED_OFF), "the generator says why");
    fill(&mut f, 24 * 64 * 80 / 100);
    tick(&mut f);
    assert!(!f.sensor(at(1)).unwrap().on, "80% sits between the thresholds");
    fill(&mut f, 24 * 64 * 70 / 100);
    tick(&mut f);
    assert_eq!(f.power.gen_pole, [Some(0)], "drained: back on");
}

#[test]
fn rules_are_chosen_and_a_switch_cuts_whatever_is_read() {
    let mut f = line();
    f.set_sensor(at(1), 1);
    tick(&mut f);
    assert_eq!(f.power.gen_pole, [None], "always off");
    f.set_sensor(at(1), 99);
    assert_eq!(f.sensor(at(1)).unwrap().rule, 1, "a rule that does not exist is ignored");
    f.set_sensor(at(1), 0);
    tick(&mut f);
    assert_eq!(f.power.gen_pole, [Some(0)]);
}

#[test]
fn a_belt_is_read_by_the_items_on_it_and_a_sensor_in_the_dark_stays_on() {
    let mut f = Factory { by_hand: true, ..Factory::default() };
    put(&mut f, BELT, at(0), 1);
    put(&mut f, SENSOR, at(2), 1);
    f.belts[0].items.clear();
    assert_eq!(f.level_at(at(0)), Some(0));
    for p in [0.2, 0.55] {
        f.belts[0].items.push(super::super::belt::BeltItem { item: IRON_PLATE, p });
    }
    assert_eq!(f.level_at(at(0)), Some(66));
    assert_eq!(f.level_at(at(5)), None);
    tick(&mut f);
    assert!(f.sensor(at(2)).unwrap().on, "nothing behind it: on");
    assert!(f.describe(at(2)).unwrap().contains("nothing in front"));
}

#[test]
fn sensors_survive_a_save_round_trip_and_leave_with_their_block() {
    use crate::bytes::{ByteReader, ByteWriter};
    let mut f = line();
    f.set_sensor(at(1), 4);
    fill(&mut f, 24 * 64);
    tick(&mut f);
    tick(&mut f);
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let mut g = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).unwrap();
    let mut again = ByteWriter::default();
    g.write_state(&mut again);
    assert!(again.bytes == w.bytes);
    let s = g.sensor(at(1)).unwrap();
    assert_eq!((s.rule, s.dir, s.on), (4, 1, true));
    assert!(g.remove(at(1)).is_empty());
    assert!(g.sensor(at(1)).is_none());
    assert_eq!(g.count(Kind::Sensor), 0);
}
