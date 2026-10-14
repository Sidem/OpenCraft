//! The build-menu card of a processing machine (`factory/facts.rs`): its size, how it is driven and the numbers of the
//! tier, all read from its `ProcessSpec`. A power source's output is a constant of its own module, named here.
//! To give a new processor its card: nothing, unless it has an `Energy` kind that is new; then add its arm.

use super::super::describe::fmt_int;
use super::super::facts::{fuel_names, Facts};
use super::hangar::TIERS as HANGAR_TIERS;
use super::specs::{Energy, Pick, ProcessSpec};
use super::steam::TURBINE_KW;
use crate::block::{BLAST_FURNACE, HOIST, WINCH};
use crate::item::{self, DIESEL_CANISTER, FUEL_CELL};

pub fn facts(spec: &ProcessSpec, tier: usize) -> Facts {
    let t = &spec.tiers[tier.min(spec.tiers.len() - 1)];
    let mut f = Facts::new(spec.footprint.size);
    match t.energy {
        Energy::Burner => f.fuel = Some(fuel_names()),
        Energy::Electric | Energy::Hoist | Energy::Compute | Energy::Cooling | Energy::Optimizer => {
            f.uses = Some(t.power);
        }
        Energy::Recipe if spec.block == BLAST_FURNACE => f.stat("Heat", "From the coal it is given"),
        Energy::Recipe | Energy::Beam => {}
        Energy::Boiler => {
            f.fuel = Some(fuel_names());
            f.stat("Also needs", "Water, piped in");
        }
        Energy::Turbine => {
            f.makes = Some((TURBINE_KW, ""));
            f.stat("Needs", "Steam from boilers");
        }
        Energy::Solar => f.makes = Some((super::solar::SOLAR_KW, ", in full sun")),
        Energy::Accumulator => {
            f.makes = Some((super::solar::ACCUMULATOR_KW, ", from stored power"));
            f.stat("Stores", format!("{} kJ", fmt_int((super::solar::CHARGE_CAP / crate::TICK_RATE).into())));
        }
        Energy::Diesel => {
            f.fuel = Some(item::name(DIESEL_CANISTER).to_string());
            f.makes = Some((super::diesel::DIESEL_KW, ""));
        }
        Energy::Hydro => {
            f.makes = Some((super::hydro::MAX_KW, ", with flowing water"));
            f.stat("Needs", "Flowing water touching it");
        }
        Energy::Reactor => {
            f.fuel = Some(item::name(FUEL_CELL).to_string());
            f.makes = Some((super::nuclear::REACTOR_KW, ""));
            f.stat("Also needs", "Water coolant, piped in");
        }
    }
    if matches!(spec.block, HOIST | WINCH) {
        f.stat("Lifts", "You, fast, up the shaft beside it");
    }
    if matches!(spec.pick, Pick::Chosen | Pick::ByInput | Pick::Research) && t.speed != 1000 {
        let (whole, tenth) = (t.speed / 1000, t.speed % 1000 / 100);
        f.stat("Speed", if tenth == 0 { format!("×{whole}") } else { format!("×{whole}.{tenth}") });
    }
    if spec.compute != 0 {
        let n = fmt_int(spec.compute.unsigned_abs().into());
        f.stat(if spec.compute > 0 { "Makes compute" } else { "Uses compute" }, format!("{n} TF"));
    }
    if spec.pick == Pick::Hangar {
        let h = &HANGAR_TIERS[tier.min(HANGAR_TIERS.len() - 1)];
        f.stat("Fleet", format!("{} drones", h.fleet));
        f.stat("Reach", format!("{} blocks", h.reach));
        f.stat("Cargo range", format!("{} blocks", h.cargo_range));
    }
    if spec.pick.stores() {
        f.stat("Holds", format!("{} stacks", spec.buffers[2]));
    }
    f
}
