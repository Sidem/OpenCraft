use super::*;
use crate::block::{
    ARC_FURNACE, ASSEMBLER, BELT, BLAST_FURNACE, CONSTRUCTOR, DRONE_PORT, GENERATOR, POLE, PUMP, QUARRY, RAIL, SMELTER,
    SOLAR_PANEL, STORAGE, TURBINE,
};
use crate::lessons::SIDE_BIT;
use crate::math::IVec3;
use crate::world::World;

fn place(f: &mut Factory, block: crate::block::BlockId, x: i32) {
    let pos = IVec3::new(x, 0, 0);
    f.place(&mut World::new(1, 2), block, pos, 0, pos, 0);
}

#[test]
fn a_tip_done_once_stays_done_in_the_world_and_its_save() {
    use crate::action::Action;
    use crate::bytes::{ByteReader, ByteWriter};
    use crate::sim::{PlayerId, Sim};
    let mut sim = Sim::new(7, 2);
    sim.apply(PlayerId(0), Action::Give { item: IRON_ORE.into(), count: 3 });
    sim.step();
    assert_eq!(sim.hints_reached, 1, "found ore");
    // The ore is used up or put away: the live progress goes back, the world's does not.
    sim.players[0].as_mut().unwrap().inventory = Inventory::default();
    for _ in 0..2 * crate::TICK_RATE {
        sim.step();
    }
    let inv = &sim.players[0].as_ref().unwrap().inventory;
    assert_eq!(progress(inv, &sim.factory), 0);
    assert_eq!(sim.hints_reached, 1);

    let mut w = ByteWriter::default();
    sim.write_state(&mut w);
    let mut back = Sim::new(7, 2);
    back.read_state(&mut ByteReader::new(&w.bytes)).expect("reads back");
    assert_eq!(back.hints_reached, 1, "saved with the world");
}

#[test]
fn hints_follow_what_the_player_has_done() {
    let (mut inv, mut f) = (Inventory::default(), Factory::default());
    assert_eq!(progress(&inv, &f), 0);
    inv.add(IRON_ORE.into(), 3);
    assert_eq!(progress(&inv, &f), 1, "found ore");
    inv.add(IRON_INGOT, 2);
    assert_eq!(progress(&inv, &f), 2, "smelted");
    inv.add(MINER.into(), 1);
    assert_eq!(progress(&inv, &f), 3, "crafted a miner");
    place(&mut f, MINER, 0);
    assert_eq!(progress(&inv, &f), 4, "placed it");
    place(&mut f, GENERATOR, 5);
    assert_eq!(progress(&inv, &f), 4, "a generator but no pole yet");
    place(&mut f, POLE, 8);
    assert_eq!(progress(&inv, &f), 5, "powered");
    place(&mut f, BELT, 1);
    assert_eq!(progress(&inv, &f), 5, "a belt but no box yet");
    place(&mut f, STORAGE, 2);
    assert_eq!(progress(&inv, &f), 6);
    place(&mut f, SMELTER, 3);
    assert_eq!(progress(&inv, &f), 7);
    place(&mut f, CONSTRUCTOR, 4);
    assert_eq!(progress(&inv, &f), 8);
    f.research.add_unit(0);
    assert_eq!(progress(&inv, &f), 9, "researching");
    place(&mut f, PUMP, 6);
    assert_eq!(progress(&inv, &f), 10, "moved water");
    place(&mut f, QUARRY, 7);
    assert_eq!(progress(&inv, &f), 11, "dug");
    let mechanics = TECHS.iter().position(|t| t.name == "Mechanics").unwrap() as u8;
    (0..TECHS[mechanics as usize].units).for_each(|_| f.research.add_unit(mechanics));
    assert_eq!(progress(&inv, &f), 12, "kits");
    place(&mut f, ASSEMBLER, 20);
    assert_eq!(progress(&inv, &f), 13);
    place(&mut f, BLAST_FURNACE, 30);
    assert_eq!(progress(&inv, &f), 14);
    place(&mut f, TURBINE, 40);
    assert_eq!(progress(&inv, &f), 15);
    place(&mut f, ARC_FURNACE, 50);
    assert_eq!(progress(&inv, &f), 16, "silicon");
    let n = HINTS.len() - 9; // the chemistry hints (oil, nuclear, gold), the compute and the orbit hints come last
    let violet = TECHS.iter().position(|t| t.name == "Violet Science").unwrap() as u8;
    (0..TECHS[violet as usize].units).for_each(|_| f.research.add_unit(violet));
    assert_eq!(progress(&inv, &f), n - 8, "violet science");
    place(&mut f, SOLAR_PANEL, 60);
    assert_eq!(progress(&inv, &f), n - 7, "solar");
    let processors = TECHS.iter().position(|t| t.name == "Processors").unwrap() as u8;
    (0..TECHS[processors as usize].units).for_each(|_| f.research.add_unit(processors));
    assert_eq!(progress(&inv, &f), n - 6, "ghosts: processors done");
    place(&mut f, DRONE_PORT, 70);
    assert_eq!(progress(&inv, &f), n - 5, "a drone port");
    inv.worn[4] = crate::item::JETPACK;
    assert_eq!(progress(&inv, &f), n - 4, "flying");
    inv.add(crate::item::PLANNER, 1);
    assert_eq!(progress(&inv, &f), n - 3, "terraforming");
    let bauxite = TECHS.iter().position(|t| t.name == "Bauxite Processing").unwrap() as u8;
    (0..TECHS[bauxite as usize].units).for_each(|_| f.research.add_unit(bauxite));
    assert_eq!(progress(&inv, &f), n - 2, "far ground");
    place(&mut f, RAIL, 80);
    assert_eq!(progress(&inv, &f), n - 1, "rails");
    inv.add(crate::item::HOVER_PACK, 1);
    assert_eq!(progress(&inv, &f), n, "the older hints are done");
    place(&mut f, crate::block::PUMPJACK, 90);
    assert_eq!(progress(&inv, &f), n + 1, "oil");
    place(&mut f, crate::block::REACTOR, 100);
    assert_eq!(progress(&inv, &f), n + 2, "nuclear");
    let gold = TECHS.iter().position(|t| t.name == "Gold Science").unwrap() as u8;
    (0..TECHS[gold as usize].units).for_each(|_| f.research.add_unit(gold));
    assert_eq!(progress(&inv, &f), n + 3, "gold science");
    place(&mut f, crate::block::DATACENTER, 110);
    assert_eq!(progress(&inv, &f), n + 4, "compute");
    place(&mut f, crate::block::LASER_EMITTER, 120);
    assert_eq!(progress(&inv, &f), n + 5, "lasers");
    let survey = TECHS.iter().position(|t| t.name == "AI Survey").unwrap() as u8;
    (0..TECHS[survey as usize].units).for_each(|_| f.research.add_unit(survey));
    assert_eq!(progress(&inv, &f), n + 6, "smarter tools");
    place(&mut f, crate::block::LAUNCH_SITE, 130);
    assert_eq!(progress(&inv, &f), n + 7, "a launch site");
    f.orbit.launch(crate::item::SURVEY_SATELLITE, IVec3::ZERO, 0);
    assert_eq!(progress(&inv, &f), n + 7, "a survey satellite is not yet the other kinds");
    f.orbit.launch(crate::item::POWER_SATELLITE, IVec3::ZERO, 0);
    assert_eq!(progress(&inv, &f), n + 8, "satellites of another kind");
    place(&mut f, crate::block::POD_LAUNCHER, 150);
    assert_eq!(progress(&inv, &f), HINTS.len(), "all done");
    assert!(HINTS.iter().all(|h| !h.text.is_empty()));
    // Markup the host draws (`web/src/ui/hints.ts`): `*words*` and `[keys]` come in closed pairs.
    for h in HINTS {
        assert_eq!(h.text.matches('*').count() % 2, 0, "{}", h.text);
        assert_eq!(h.text.matches('[').count(), h.text.matches(']').count(), "{}", h.text);
    }
}

#[test]
fn a_later_step_skips_the_earlier_hints() {
    let (inv, mut f) = (Inventory::default(), Factory::default());
    place(&mut f, SMELTER, 0);
    assert_eq!(progress(&inv, &f), 2, "a furnace but no miner");
    place(&mut f, BELT, 1);
    place(&mut f, STORAGE, 2);
    assert_eq!(progress(&inv, &f), 2, "the opening's line (furnace, belts, boxes) leaves the miner tips");
    place(&mut f, MINER, 3);
    place(&mut f, GENERATOR, 4);
    assert_eq!(progress(&inv, &f), 7, "a powered miner beside a belt line with a smelter");
}

#[test]
fn town_tips_show_once_settlements_is_researched_and_clear_as_the_town_grows() {
    let mut f = Factory::default();
    let mut s = Settlements::default();
    let base = SIDE_BASE as u32;
    assert!(side(&f, &s, 0).is_empty(), "not researched yet");
    let settlements = TECHS.iter().position(|t| t.name == "Settlements").unwrap() as u8;
    (0..TECHS[settlements as usize].units).for_each(|_| f.research.add_unit(settlements));
    assert_eq!(side(&f, &s, 0), [base, base + 1, base + 2]);
    assert_eq!(side(&f, &s, 1 << SIDE_BIT), [base + 1, base + 2], "a taught tip stays away");
    assert!(text(base as usize).starts_with("Found a town"));
    assert_eq!(text(base as usize + 9), "");
    let before = progress(&Inventory::default(), &f);
    s.towns.push(crate::settlements::Town::new(IVec3::new(0, 70, 900), 0));
    assert_eq!(side(&f, &s, 0), [base + 1, base + 2], "founded");
    s.towns[0].built = 1;
    assert_eq!(side(&f, &s, 0), [base + 2], "its camp stands");
    assert_eq!(progress(&Inventory::default(), &f), before, "side tips never move the main order");
    place(&mut f, RAIL, 0);
    assert!(side(&f, &s, 0).is_empty(), "a rail line can carry its goods");
}
