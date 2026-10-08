//! The research center: packs only (a stack of each kind), twice a lab's speed, one pack of each kind per unit, never
//! more units than a tech has left, its unit in a save, and the rule that decides which techs a small lab can work.

use crate::block::{COAL_ORE, GENERATOR, POLE, RESEARCH_CENTER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::lab::LAB_TIERS;
use crate::factory::{Factory, Machine};
use crate::item::{IRON_PLATE, RED_PACK};
use crate::math::IVec3;
use crate::research::{TechState, PACKS, TECHS};
use crate::world::World;

use super::*;

/// A center at the origin, a pole and a coal generator (60 kW).
fn rig() -> Factory {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    f.place(&mut world, RESEARCH_CENTER, IVec3::ZERO, 0, IVec3::ZERO, 0);
    let pole = IVec3::new(4, 0, 0);
    f.place(&mut world, POLE, pole, 0, pole, 0);
    let gen = pole + IVec3::new(1, 0, 0);
    f.place(&mut world, GENERATOR, gen, 0, gen, 0);
    assert_eq!(f.insert(gen, COAL_ORE.into(), 64), 64);
    f
}

fn run(f: &mut Factory, seconds: f64) {
    let mut world = World::new(1, 2);
    let mut events = Vec::new();
    for tick in 0..(seconds * crate::TICK_RATE as f64) as u64 {
        f.update(&mut world, tick, &mut events);
    }
}

fn center(f: &Factory) -> &Processor {
    f.processors.iter().find(|p| p.is_center()).unwrap()
}

#[test]
fn it_takes_science_packs_only_a_stack_of_each() {
    let mut f = rig();
    assert_eq!(f.insert(IVec3::ZERO, RED_PACK, 100), stack_size(RED_PACK));
    assert_eq!(f.insert(IVec3::ZERO, IRON_PLATE, 5), 0);
    assert_eq!(center(&f).input.slots.len(), CENTER_SLOTS);
}

#[test]
fn it_researches_at_twice_a_labs_speed_with_one_pack_a_unit_and_never_overshoots() {
    let mut f = rig();
    f.insert(IVec3::ZERO, RED_PACK, 64);
    f.research.set_current(Some(0)); // Belt Routing: 10 units of 5 s, so 2.5 s a unit here
    run(&mut f, 5.05);
    assert_eq!((f.research.progress(0), center(&f).input.count(RED_PACK)), (2, 61), "two done, a third started");
    assert_eq!(f.power.demand[0], 20);
    run(&mut f, 30.0);
    assert_eq!(f.research.state(0), TechState::Done);
    assert_eq!(center(&f).input.count(RED_PACK), 54, "exactly one pack per unit");
    assert!(center(&f).status_text().starts_with("No research chosen"));
}

#[test]
fn it_waits_for_packs_and_names_them() {
    let mut f = rig();
    f.research.set_current(Some(0));
    run(&mut f, 1.0);
    assert_eq!(center(&f).status_text(), "Waiting for Red Science Pack");
    f.insert(IVec3::ZERO, RED_PACK, 1);
    run(&mut f, 1.0);
    assert!(center(&f).status_text().starts_with("Researching Belt Routing"), "{}", center(&f).status_text());
    let text = f.describe(IVec3::ZERO).unwrap();
    assert!(text.contains("×2 speed · needs 20 kW") && text.contains("Belt Routing: 0 of 10 units"), "{text}");
}

#[test]
fn a_unit_in_progress_survives_a_save() {
    let mut f = rig();
    f.insert(IVec3::ZERO, RED_PACK, 10);
    f.research.set_current(Some(0));
    run(&mut f, 1.0);
    let (unit, progress) = (center(&f).study.unit, center(&f).study.progress);
    assert!(unit.is_some() && progress > 0);
    let mut w = ByteWriter::default();
    center(&f).write_state(&mut w);
    let back = Processor::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!((back.study.unit, back.study.progress), (unit, progress));
    // Given back with the rest if the center is taken down: the packs of an unfinished unit.
    assert_eq!(back.contents().len(), back.input.contents().len() + back.study_contents().len());
}

#[test]
fn tiers_are_twice_a_labs_and_the_lab_rule_follows_the_packs() {
    for (c, l) in CENTER_SPEC.tiers.iter().zip(&LAB_TIERS) {
        assert_eq!((c.speed, c.power), (l.speed * 2000, l.power * 2));
    }
    assert!(PACKS.len() <= CENTER_SLOTS && LAB_PACK_SLOTS <= CENTER_SLOTS);
    // Only the techs that use a gold pack, past the lab's slots, need a center.
    let centered: Vec<&str> =
        (0..TECHS.len() as u8).filter(|&t| needs_center(t)).map(|t| TECHS[t as usize].name).collect();
    assert_eq!(
        centered,
        [
            "Mk5 Machines",
            "Wafers",
            "Accelerators",
            "Data Network",
            "Datacenters",
            "Cooling",
            "AI Labs",
            "AI Research",
            "Optimizer"
        ]
    );
}

#[test]
fn a_gold_tech_is_researched_in_a_center_and_a_small_lab_has_no_slot_for_the_pack() {
    let mut f = rig();
    let mk5 = TECHS.iter().position(|t| t.name == "Mk5 Machines").unwrap() as u8;
    for (i, t) in TECHS.iter().enumerate().filter(|(i, _)| *i as u8 != mk5) {
        (0..t.units).for_each(|_| f.research.add_unit(i as u8));
    }
    assert!(needs_center(mk5) && f.research.state(mk5) == TechState::Available);
    for pack in PACKS {
        assert_eq!(f.insert(IVec3::ZERO, pack, 2), 2);
    }
    f.research.set_current(Some(mk5));
    run(&mut f, 60.0);
    assert_eq!(f.research.progress(mk5), 2, "two units, a gold pack each");
    assert!(PACKS.iter().all(|&p| center(&f).input.count(p) == 0));
    let lab = IVec3::new(0, 0, 6);
    f.place(&mut World::new(1, 2), crate::block::LAB, lab, 0, lab, 0);
    assert_eq!(f.insert(lab, crate::item::GOLD_PACK, 1), 0, "a small lab holds four kinds of pack");
}
