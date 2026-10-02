use super::*;
use crate::block::{BELT, MINER};
use crate::item::{BLUE_KIT, GREEN_KIT};
use crate::raycast::RayHit;

const AT: IVec3 = IVec3::new(0, 70, 0);
const BELTS: IVec3 = IVec3::new(0, 70, 2);

/// A game with every tech done, a miner at `AT` and three belts east from `BELTS`.
fn factory_game() -> Game {
    let mut g = Game::new(7, 2);
    for (t, tech) in research::TECHS.iter().enumerate() {
        (0..tech.units).for_each(|_| g.sim.factory.research.add_unit(t as u8));
    }
    g.sim.factory.add_miner(AT, 3, None, 0);
    for x in 0..3 {
        g.sim.factory.add_belt(IVec3::new(x, 70, 2), 1);
    }
    g.run_ticks(1);
    g
}

/// The aim of `kit` at the block `pos` (a tick re-aims the real crosshair, so the target is set last).
fn aim_at(g: &mut Game, pos: IVec3, id: u8, kit: ItemId) -> Aim {
    g.target = Some(RayHit { block: pos, normal: IVec3::ZERO, id });
    g.upgrade_aim(kit)
}

#[test]
fn a_kit_over_a_machine_outlines_it_and_says_what_it_costs() {
    let mut g = factory_game();
    g.give(GREEN_KIT.0, 2);
    g.run_ticks(1);
    let aim = aim_at(&mut g, AT, MINER, GREEN_KIT);
    assert!(aim.label.contains("Mk1 → Mk2"), "{}", aim.label);
    assert!(aim.label.contains("4 × Green Kit (you have 2)"), "{}", aim.label);
    assert_eq!((aim.machine, aim.colour), (Some((AT, AT)), BLOCKED_RED), "too few kits: red");
    g.give(GREEN_KIT.0, 2);
    g.run_ticks(1);
    let aim = aim_at(&mut g, AT, MINER, GREEN_KIT);
    assert_eq!(aim.colour, upgrades::TIER_COLOURS[1] as i32);
    assert!(!aim.label.contains("you have"), "{}", aim.label);
    g.line.aim = aim;
    assert_eq!(g.aim_box(), [0, 70, 0, 0, 70, 0, upgrades::TIER_COLOURS[1] as i32]);
}

#[test]
fn the_wrong_kit_or_missing_research_is_explained() {
    let mut g = factory_game();
    let wrong = aim_at(&mut g, AT, MINER, BLUE_KIT);
    assert!(wrong.label.contains("Needs a Green Kit") && wrong.machine.is_none(), "{}", wrong.label);
    g.sim.factory.research = Default::default();
    let locked = aim_at(&mut g, AT, MINER, GREEN_KIT);
    assert!(locked.label.contains("Research") && locked.machine.is_none(), "{}", locked.label);
    assert!(aim_at(&mut g, AT + IVec3::new(5, 0, 5), MINER, GREEN_KIT).label.is_empty(), "nothing there");
}

#[test]
fn a_kit_over_a_belt_outlines_one_belt_or_with_shift_the_whole_line() {
    let mut g = factory_game();
    g.give(GREEN_KIT.0, 2);
    g.run_ticks(1);
    let one = aim_at(&mut g, BELTS, BELT, GREEN_KIT);
    assert_eq!(one.belts, [BELTS]);
    assert!(one.label.contains("1 belt · 1 × Green Kit") && one.label.contains("Shift"), "{}", one.label);
    g.body_mut().input.sprint = true;
    let line = aim_at(&mut g, BELTS, BELT, GREEN_KIT);
    assert_eq!(line.belts.len(), 3);
    assert!(line.label.contains("3 belts · 3 × Green Kit (you have 2)"), "{}", line.label);
    assert!(line.label.contains("whole line"), "{}", line.label);
    g.line.aim = line;
    // Two kits pay for two belts: the third outline is red.
    let cells = g.planned_cells();
    let ok = upgrades::TIER_COLOURS[1] as i32;
    assert_eq!([cells[3], cells[7], cells[11]], [ok, ok, 0]);
}
