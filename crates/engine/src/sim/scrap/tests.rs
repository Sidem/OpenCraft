use crate::action::Action;
use crate::block::{self, AIR, RUSTED_GIRDER, RUSTED_MACHINE, RUSTED_PLATING, STONE};
use crate::item::{CIRCUIT, IRON_INGOT, SCRAP_METAL};
use crate::math::IVec3;
use crate::recipes::MACHINE_RECIPES;
use crate::sim::{PlayerId, Sim, SimEvent};

const P: PlayerId = PlayerId(0);

/// Puts `id` at a spot out in the sky, breaks it, and returns what fell out as (item, count).
fn break_one(sim: &mut Sim, id: block::BlockId) -> Vec<(crate::item::ItemId, u32)> {
    let pos = IVec3::new(300, 200, -300);
    assert!(sim.world.set_block_anywhere(pos, id));
    sim.events.clear();
    sim.apply(P, Action::BreakBlock { pos });
    assert_eq!(sim.world.block_anywhere(pos), Some(AIR));
    sim.events
        .iter()
        .filter_map(|e| match e {
            SimEvent::Dropped { item, count, .. } => Some((*item, *count)),
            _ => None,
        })
        .collect()
}

#[test]
fn rusted_blocks_break_into_scrap_and_nothing_else_but_a_circuit() {
    let mut sim = Sim::new(7, 2);
    for (id, most) in [(RUSTED_PLATING, 1), (RUSTED_GIRDER, 2), (RUSTED_MACHINE, 4)] {
        for _ in 0..30 {
            for (item, count) in break_one(&mut sim, id) {
                assert!(item == SCRAP_METAL && (1..=most).contains(&count) || item == CIRCUIT && id == RUSTED_MACHINE);
            }
        }
    }
    let drops = break_one(&mut sim, STONE);
    assert!(drops.iter().all(|d| d.0 != SCRAP_METAL), "other blocks leave no scrap");
}

#[test]
fn a_machine_husk_holds_a_circuit_now_and_then() {
    let mut sim = Sim::new(7, 2);
    let breaks = 400;
    let circuits = (0..breaks).filter(|_| break_one(&mut sim, RUSTED_MACHINE).iter().any(|d| d.0 == CIRCUIT)).count();
    assert!((60..=140).contains(&circuits), "{circuits} of {breaks} husks held a circuit");
    let plating = (0..breaks).filter(|_| break_one(&mut sim, RUSTED_PLATING).iter().any(|d| d.0 == CIRCUIT)).count();
    assert_eq!(plating, 0, "plating holds none");
}

#[test]
fn a_smelter_melts_three_scrap_into_an_ingot() {
    let row = 135;
    let r = &MACHINE_RECIPES[row];
    assert_eq!((r.inputs, r.outputs), (&[(SCRAP_METAL, 3)][..], &[(IRON_INGOT, 1)][..]));
    assert!(crate::factory::process_spec(block::SMELTER).unwrap().recipe(row as u16).is_some());
}
