use crate::action::Action;
use crate::block::{COAL_ORE, CONSTRUCTOR, FILTER, GENERATOR, IRON_ORE, POLE, QUARRY, SILO, SMELTER};
use crate::item::{IRON_INGOT, IRON_PLATE};
use crate::math::IVec3;
use crate::recipes::MACHINE_RECIPES;
use crate::sim::PlayerId;
use crate::Game;

const P: PlayerId = PlayerId(0);

fn recipe_for(out: crate::item::ItemId) -> u16 {
    MACHINE_RECIPES.iter().position(|r| r.main().0 == out).unwrap() as u16
}

#[test]
fn switching_off_a_constructor_stops_its_work_and_power_draw_and_survives_save() {
    let mut g = Game::new(2024, 2);
    let pole = IVec3::new(0, 120, 0);
    let gen = IVec3::new(1, 120, 0);
    let con = IVec3::new(-1, 120, 0);
    let down = IVec3::new(0, 119, 0);
    g.sim.factory.place(&mut g.sim.world, POLE, pole, 0, down, 0);
    g.sim.factory.place(&mut g.sim.world, GENERATOR, gen, 0, down, 0);
    g.sim.factory.place(&mut g.sim.world, CONSTRUCTOR, con, 0, down, 0);
    g.sim.factory.connect(pole, gen);
    g.sim.factory.connect(pole, con);
    g.sim.factory.insert(gen, COAL_ORE.into(), 10);
    g.sim.factory.set_recipe(con, Some(recipe_for(IRON_PLATE)));
    g.sim.factory.insert(con, IRON_INGOT, 20);
    g.run_ticks(5);
    assert!(g.sim.factory.power_totals().demand > 0, "draws power while on");
    assert!(g.machine_switchable(con.x, con.y, con.z));
    assert!(g.machine_enabled(con.x, con.y, con.z));

    // Switch it off through the action.
    g.set_machine_enabled(con.x, con.y, con.z, false);
    g.run_ticks(5);
    assert!(!g.machine_enabled(con.x, con.y, con.z));
    assert_eq!(g.sim.factory.power_totals().demand, 0, "draws no power while off");
    assert_eq!(g.machine_status(con.x, con.y, con.z), "Switched off");
    let made_before =
        g.sim.factory.panel(con).unwrap().slots.iter().filter(|s| s.0 == 2).map(|s| s.1.count).sum::<u32>();
    g.run_ticks(180);
    let made_after =
        g.sim.factory.panel(con).unwrap().slots.iter().filter(|s| s.0 == 2).map(|s| s.1.count).sum::<u32>();
    assert_eq!(made_before, made_after, "does no work while off");

    // Efficiency samples skip switched-off machines (not a stall).
    let mut sampled = Vec::new();
    g.sim.factory.samples(|_, pos, s| {
        if pos == con {
            sampled.push(s);
        }
    });
    assert_eq!(sampled, vec![None]);

    // Round-trip through save preserves the off state.
    let saved = g.save();
    let mut back = Game::load(&saved, 2).expect("loads");
    assert!(!back.machine_enabled(con.x, con.y, con.z));
    back.run_ticks(5);
    assert_eq!(back.sim.factory.power_totals().demand, 0);

    // Switch back on: resumes work and power draw.
    back.set_machine_enabled(con.x, con.y, con.z, true);
    back.run_ticks(180);
    assert!(back.machine_enabled(con.x, con.y, con.z));
    let resumed =
        back.sim.factory.panel(con).unwrap().slots.iter().filter(|s| s.0 == 2).map(|s| s.1.count).sum::<u32>();
    assert!(resumed > made_after, "resumes producing when switched back on");
}

#[test]
fn switching_off_a_burner_smelter_stops_fuel_burn_and_passive_blocks_cannot_switch() {
    let mut g = Game::new(2024, 2);
    let smelt = IVec3::new(0, 120, 0);
    let filter = IVec3::new(2, 120, 0);
    let silo = IVec3::new(6, 120, 0);
    let quarry = IVec3::new(-4, 120, 0);
    let down = IVec3::new(0, 119, 0);
    g.sim.factory.place(&mut g.sim.world, SMELTER, smelt, 0, down, 0);
    g.sim.factory.place(&mut g.sim.world, FILTER, filter, 0, down, 0);
    g.sim.factory.place(&mut g.sim.world, SILO, silo, 0, down, 0);
    g.sim.factory.place(&mut g.sim.world, QUARRY, quarry, 0, down, 0);

    assert!(g.machine_switchable(smelt.x, smelt.y, smelt.z));
    assert!(!g.machine_switchable(filter.x, filter.y, filter.z));
    assert!(!g.machine_switchable(silo.x, silo.y, silo.z));
    assert!(g.machine_switchable(quarry.x, quarry.y, quarry.z));

    g.sim.factory.insert(smelt, IRON_ORE.into(), 10);
    g.sim.factory.insert(smelt, COAL_ORE.into(), 10);
    g.sim.apply(P, Action::SetMachineEnabled { pos: smelt, on: false });
    g.run_ticks(120);
    assert_eq!(g.machine_status(smelt.x, smelt.y, smelt.z), "Switched off");
    let slots = &g.sim.factory.panel(smelt).unwrap().slots;
    assert_eq!(slots[1].1.count, 10, "burned no coal while off");
    assert_eq!(slots[2].1.count, 0, "smelted nothing while off");

    // Quarry switch drives its paused flag.
    g.sim.apply(P, Action::SetMachineEnabled { pos: quarry, on: false });
    assert!(!g.machine_enabled(quarry.x, quarry.y, quarry.z));
    assert!(g.sim.factory.quarry(quarry).unwrap().paused);
    g.sim.apply(P, Action::SetMachineEnabled { pos: quarry, on: true });
    assert!(g.machine_enabled(quarry.x, quarry.y, quarry.z));
    assert!(!g.sim.factory.quarry(quarry).unwrap().paused);
}
