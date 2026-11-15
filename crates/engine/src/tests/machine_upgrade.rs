//! The panel's upgrade button (`Game::machine_upgrade`, `upgrade_machine`): what it offers, what it costs and that
//! pressing it does what a kit in hand does.

use super::run_until_ready;
use crate::block::SMELTER;
use crate::item::GREEN_KIT;
use crate::math::IVec3;
use crate::research::{Unlock, TECHS};
use crate::Game;

#[test]
fn a_smelter_panel_offers_mk2_for_green_kits_and_the_button_upgrades_it() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(0, 0);
    let pos = IVec3::new(0, y + 40, 0);
    g.place_at(pos.x, pos.y, pos.z, u16::from(SMELTER), 0);
    assert_eq!(g.machine_upgrade(pos.x, pos.y, pos.z).len(), 5, "a Mk1 smelter has a Mk2 to go to");
    assert_eq!(g.machine_name(pos.x, pos.y, pos.z), "Smelter Mk1");

    let kits = g.sim.factory.next_upgrade(pos).expect("a next step").kits;
    let offer = g.machine_upgrade(pos.x, pos.y, pos.z);
    assert_eq!(offer, [2, u32::from(GREEN_KIT.0), kits, 0, 0], "Mk2, green kits, none held, not researched");

    g.give(GREEN_KIT.0, kits);
    g.run_ticks(1);
    g.upgrade_machine(pos.x, pos.y, pos.z);
    g.run_ticks(1);
    assert_eq!(g.machine_name(pos.x, pos.y, pos.z), "Smelter Mk1", "the tech is missing: nothing happens");

    for (tech, row) in TECHS.iter().enumerate() {
        (0..row.units).for_each(|_| g.sim.factory.research.add_unit(tech as u8));
    }
    assert!(g.sim.factory.research.has(Unlock::Upgrade(SMELTER, 1)));
    assert_eq!(g.machine_upgrade(pos.x, pos.y, pos.z), [2, u32::from(GREEN_KIT.0), kits, kits, 1]);
    g.upgrade_machine(pos.x, pos.y, pos.z);
    g.run_ticks(1);
    assert_eq!(g.machine_name(pos.x, pos.y, pos.z), "Smelter Mk2");
    assert_eq!(g.inventory().count(GREEN_KIT), 0, "the kits were spent");
    assert_eq!(g.machine_upgrade(pos.x, pos.y, pos.z)[0], 3, "the button now offers Mk3");
}

#[test]
fn a_building_with_no_tiers_offers_no_upgrade() {
    let g = Game::new(2024, 3);
    assert!(g.machine_upgrade(0, 0, 0).is_empty(), "nothing is built there");
}
