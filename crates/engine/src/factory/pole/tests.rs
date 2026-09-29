use super::super::tests::{place_block, rod_maker, round_trip, run};
use super::*;
use crate::block::{CABLE, COAL_ORE, GENERATOR, POLE};

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
