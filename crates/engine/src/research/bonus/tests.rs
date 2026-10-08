use super::*;
use crate::research::{needs_ai_lab, TECHS};

const KINDS: [Bonus; 3] = [Bonus::Mining, Bonus::Machines, Bonus::Drones];

fn levels(r: &mut Research, bonus: Bonus, n: u32) {
    (0..n).for_each(|_| r.add_unit(FIRST + bonus as u8));
}

#[test]
fn the_bonus_techs_follow_ai_research_cost_no_packs_and_need_an_ai_lab() {
    for (i, b) in KINDS.into_iter().enumerate() {
        let t = &TECHS[FIRST as usize + i];
        assert_eq!(t.unlocks, &[Unlock::Bonus(b)], "{} is {b:?}", t.name);
        assert!(t.packs.is_empty() && is_bonus(FIRST + i as u8) && needs_ai_lab(FIRST + i as u8));
        assert_eq!(TECHS[t.needs[0] as usize].name, "AI Research");
    }
    assert!(!is_bonus(FIRST - 1) && !is_bonus(FIRST + 3), "the neighbours are not bonus techs");
}

#[test]
fn each_level_adds_three_percent_to_its_own_rate_only() {
    let mut r = Research::default();
    assert_eq!(KINDS.map(|b| r.rate_permille(b)), [1000; 3]);
    levels(&mut r, Bonus::Mining, 10);
    levels(&mut r, Bonus::Drones, 1);
    assert_eq!(KINDS.map(|b| r.rate_permille(b)), [1300, 1000, 1030]);
    levels(&mut r, Bonus::Mining, 1);
    assert_eq!(r.rate_permille(Bonus::Mining), 1330, "linear, not compounding");
}

#[test]
fn the_levels_never_end_in_play_and_a_complete_world_has_the_cap() {
    let mut r = Research::default();
    levels(&mut r, Bonus::Machines, LEVELS + 5);
    assert_eq!((r.level(Bonus::Machines), r.rate_permille(Bonus::Machines)), (LEVELS, 4000));
    r.complete_all();
    assert_eq!(KINDS.map(|b| r.level(b)), [LEVELS; 3]);
}

#[test]
fn a_unit_costs_a_quarter_more_than_the_one_before() {
    let mut r = Research::default();
    let tech = FIRST + Bonus::Mining as u8;
    let base = TECHS[tech as usize].seconds;
    assert_eq!(r.unit_seconds(tech), base);
    levels(&mut r, Bonus::Mining, 1);
    assert_eq!(r.unit_seconds(tech), base * 1.25);
    levels(&mut r, Bonus::Mining, 3);
    assert!((r.unit_seconds(tech) - base * 1.25f64.powi(4)).abs() < 1e-9);
    assert_eq!(r.unit_seconds(0), TECHS[0].seconds, "an ordinary tech's unit costs the same every time");
}

#[test]
fn the_level_is_saved_with_the_research() {
    let mut r = Research::default();
    levels(&mut r, Bonus::Drones, 7);
    let mut w = crate::bytes::ByteWriter::default();
    r.write_state(&mut w);
    let back = Research::read_state(&mut crate::bytes::ByteReader::new(&w.bytes)).unwrap();
    assert_eq!(back.rate_permille(Bonus::Drones), 1210);
}

#[test]
fn drone_speed_lengthens_the_flight_step() {
    let mut sim = crate::sim::Sim::new(7, 2);
    let plain = sim.flight_step(8.0);
    levels(&mut sim.factory.research, Bonus::Drones, 10);
    assert!((sim.flight_step(8.0) - plain * 1.3).abs() < 1e-12);
}
