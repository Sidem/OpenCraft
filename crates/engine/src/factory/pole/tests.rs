use super::super::tests::{place_block, rod_maker, round_trip, run};
use super::*;
use crate::block::{CABLE, COAL_ORE, GENERATOR, LONG_POLE, MINER, POLE};
use crate::factory::Hookup;
use crate::world::World;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// A pole with a generator on top at y = 10, and a rod maker down at y = 0 in the same column.
fn shaft(f: &mut Factory) {
    place_block(f, POLE, v(0, 10, 0));
    place_block(f, GENERATOR, v(0, 11, 0));
    f.insert(v(0, 11, 0), COAL_ORE.into(), 10);
    rod_maker(f, v(1, 0, 0), 20);
}

fn speed_of_the_maker(f: &mut Factory) -> u32 {
    run(f, 0.2, |_| {});
    f.power.speed(f.power.process_pole[0])
}

#[test]
fn a_column_of_cables_carries_power_from_a_pole_to_a_machine_below() {
    let mut f = Factory::default();
    shaft(&mut f);
    for y in 1..=9 {
        place_block(&mut f, CABLE, v(0, y, 0));
    }
    assert_eq!(speed_of_the_maker(&mut f), 1000);
    // A gap in the column cuts the machine off: no generator on its grid.
    f.remove(v(0, 5, 0));
    assert_eq!(speed_of_the_maker(&mut f), 0);
    // Cables link a block apart (diagonals too), so a bent run still joins.
    place_block(&mut f, CABLE, v(1, 5, 0));
    assert_eq!(speed_of_the_maker(&mut f), 1000);
}

#[test]
fn cables_link_by_touching_and_to_poles_within_the_poles_reach() {
    let cable = |x, y, z| Pole { pos: v(x, y, z), tier: CABLE_TIER };
    let pole = Pole { pos: v(0, 0, 0), tier: 0 };
    assert!(linked(&cable(0, 1, 0), &cable(0, 2, 1)) && !linked(&cable(0, 1, 0), &cable(0, 3, 0)));
    assert!(linked(&pole, &cable(0, 5, 0)), "a pole reaches a cable within its own machine reach");
    assert!(!linked(&pole, &cable(0, 6, 0)));
    assert!(linked(&cable(0, 1, 0), &pole) == linked(&pole, &cable(0, 1, 0)), "links are symmetric");
}

#[test]
fn a_machine_prefers_a_pole_over_a_closer_cable() {
    let mut f = Factory::default();
    place_block(&mut f, CABLE, v(1, 1, 0));
    place_block(&mut f, POLE, v(4, 1, 0));
    rod_maker(&mut f, v(0, 1, 0), 0);
    run(&mut f, 0.1, |_| {});
    let pole = f.poles.iter().position(|p| p.pos == v(4, 1, 0)).map(|i| i as u32);
    assert_eq!(f.power.process_pole, [pole], "the pole, though the cable is next to it");
    // With no pole in reach the cable will do.
    f.remove(v(4, 1, 0));
    run(&mut f, 0.1, |_| {});
    assert_eq!(f.power.process_pole, [Some(0)]);
}

#[test]
fn cables_survive_a_save_round_trip_and_stay_out_of_the_pole_upgrades() {
    let mut f = Factory::default();
    place_block(&mut f, POLE, v(0, 0, 0));
    place_block(&mut f, CABLE, v(0, 1, 0));
    let g = round_trip(&f);
    assert_eq!(
        g.poles.iter().map(|p| (p.pos, p.is_cable())).collect::<Vec<_>>(),
        [(v(0, 0, 0), false), (v(0, 1, 0), true)]
    );
    assert_eq!(g.poles().count(), 1, "poles() lists no cables");
    assert!(g.cable_joins(v(0, 2, 0)) && !g.cable_joins(v(0, 8, 0)));
}

#[test]
fn a_version_21_cable_kept_its_tier_3_and_is_still_a_cable_while_tier_3_is_now_the_substation() {
    use crate::bytes::{ByteReader, ByteWriter};
    let mut w = ByteWriter::default();
    Pole { pos: v(1, 2, 3), tier: 3 }.write_state(&mut w);
    let read = |version: u32| {
        let mut r = ByteReader::new(&w.bytes);
        r.version = version;
        Pole::read_state(&mut r).map(|p| p.is_cable())
    };
    assert_eq!((read(21), read(22)), (Some(true), Some(false)));
}

fn by_hand() -> Factory {
    Factory { by_hand: true, ..Factory::default() }
}

#[test]
fn a_long_distance_pole_stands_on_a_2x2x3_footprint_and_breaks_whole() {
    let mut f = by_hand();
    place_block(&mut f, LONG_POLE, v(10, 5, 10));
    let cells = f.machine_cells(v(11, 7, 9));
    assert_eq!(cells.len(), 12);
    assert_eq!(cells[0], v(10, 5, 10), "the anchor first");
    assert!(cells.iter().all(|&c| f.anchor_of(c) == Some(v(10, 5, 10))));
    assert_eq!(f.footprint_at(v(11, 6, 10)).map(|x| x.1), Some(LONG_POLE));
    assert_eq!(f.poles().count(), 0, "the pole tool never chains from a long-distance pole");
    assert!(f.hit_boxes(v(10, 5, 10)).is_none(), "the whole cell counts");
    f.remove(v(11, 7, 9));
    assert!(f.poles.is_empty() && f.at.is_empty(), "breaking any cell removes the whole tower");
}

#[test]
fn long_distance_poles_span_far_to_each_other_but_not_to_ordinary_poles() {
    let mut f = by_hand();
    for x in [0, 150, 170] {
        place_block(&mut f, LONG_POLE, v(x, 3, 0));
    }
    place_block(&mut f, POLE, v(15, 3, 5));
    place_block(&mut f, POLE, v(20, 3, 5));
    assert_eq!(f.hookup(v(0, 3, 0), v(150, 3, 0)), Hookup::Connect);
    assert_eq!(f.hookup(v(150, 3, 0), v(0, 3, 0)), Hookup::Connect);
    assert_eq!(f.hookup(v(0, 3, 0), v(170, 3, 0)), Hookup::TooFar, "past 160 blocks");
    assert_eq!(f.hookup(v(0, 3, 0), v(15, 3, 5)), Hookup::Connect, "an ordinary pole within 16");
    assert_eq!(f.hookup(v(0, 3, 0), v(20, 3, 5)), Hookup::TooFar, "but not past it, however long the tower's span");
    // An ordinary pole's own link is no longer for it either: a Mk3 pole links 32, so 150 is out of the question.
    assert_eq!(span(2, LONG_TIER), 32);
    assert_eq!(span(0, LONG_TIER), LONG_LOCAL_LINK);
}

#[test]
fn power_crosses_a_long_span_and_the_tower_survives_a_save() {
    let mut f = by_hand();
    place_block(&mut f, LONG_POLE, v(0, 3, 0));
    place_block(&mut f, LONG_POLE, v(150, 3, 0));
    place_block(&mut f, GENERATOR, v(4, 3, 0));
    place_block(&mut f, MINER, v(146, 3, 0));
    f.connect(v(0, 3, 0), v(150, 3, 0));
    f.connect(v(0, 3, 0), v(4, 3, 0));
    f.connect(v(150, 3, 0), v(146, 3, 0));
    f.update(&mut World::new(1, 2), 1, &mut Vec::new());
    let grid = |x| f.poles.iter().position(|p| p.pos == v(x, 3, 0)).map(|i| f.power.pole_grid[i]);
    assert_eq!(grid(0), grid(150), "the two towers are one grid");
    assert!(f.power.gen_pole[0].is_some() && f.power.miner_pole[0].is_some());
    let g = round_trip(&f);
    assert_eq!(g.poles.iter().filter(|p| p.is_long()).count(), 2);
    assert_eq!(g.machine_cells(v(150, 3, 0)).len(), 12);
    assert_eq!(g.slots_used(v(0, 3, 0)), 2);
}
