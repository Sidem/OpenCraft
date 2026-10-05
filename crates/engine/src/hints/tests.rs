use super::*;
use crate::block::{
    ARC_FURNACE, ASSEMBLER, BELT, BLAST_FURNACE, CONSTRUCTOR, DRONE_PORT, GENERATOR, POLE, PUMP, QUARRY, RAIL, SMELTER,
    SOLAR_PANEL, STORAGE, TURBINE,
};
use crate::math::IVec3;
use crate::world::World;

fn place(f: &mut Factory, block: crate::block::BlockId, x: i32) {
    let pos = IVec3::new(x, 0, 0);
    f.place(&mut World::new(1, 2), block, pos, 0, pos, 0);
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
    let violet = TECHS.iter().position(|t| t.name == "Violet Science").unwrap() as u8;
    (0..TECHS[violet as usize].units).for_each(|_| f.research.add_unit(violet));
    assert_eq!(progress(&inv, &f), HINTS.len() - 8, "violet science");
    place(&mut f, SOLAR_PANEL, 60);
    assert_eq!(progress(&inv, &f), HINTS.len() - 7, "solar");
    let processors = TECHS.iter().position(|t| t.name == "Processors").unwrap() as u8;
    (0..TECHS[processors as usize].units).for_each(|_| f.research.add_unit(processors));
    assert_eq!(progress(&inv, &f), HINTS.len() - 6, "ghosts: processors done");
    place(&mut f, DRONE_PORT, 70);
    assert_eq!(progress(&inv, &f), HINTS.len() - 5, "a drone port");
    inv.add(crate::item::JETPACK, 1);
    assert_eq!(progress(&inv, &f), HINTS.len() - 4, "flying");
    inv.add(crate::item::PLANNER, 1);
    assert_eq!(progress(&inv, &f), HINTS.len() - 3, "terraforming");
    let bauxite = TECHS.iter().position(|t| t.name == "Bauxite Processing").unwrap() as u8;
    (0..TECHS[bauxite as usize].units).for_each(|_| f.research.add_unit(bauxite));
    assert_eq!(progress(&inv, &f), HINTS.len() - 2, "far ground");
    place(&mut f, RAIL, 80);
    assert_eq!(progress(&inv, &f), HINTS.len() - 1, "rails");
    inv.add(crate::item::HOVER_PACK, 1);
    assert_eq!(progress(&inv, &f), HINTS.len(), "all done");
    assert!(HINTS.iter().all(|h| !h.text.is_empty()));
}

#[test]
fn a_later_step_skips_the_earlier_hints() {
    let (inv, mut f) = (Inventory::default(), Factory::default());
    place(&mut f, SMELTER, 0);
    assert_eq!(progress(&inv, &f), 2, "a furnace but no miner");
    place(&mut f, BELT, 1);
    assert_eq!(progress(&inv, &f), 7, "a belt line with a smelter");
}
