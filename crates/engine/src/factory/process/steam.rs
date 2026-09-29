//! Steam power and bulk storage: the processors that are not recipe machines (`Energy::Boiler`,
//! `Energy::Turbine`, `Pick::Store`).
//!
//! - A boiler burns fuel from its fuel buffer into steam (`FUEL_GAIN` times a generator's energy from
//!   the same item), and each item it burns needs water: a pipe piece touching it puts it on a pipe
//!   network, and `draw_water` takes a pumped unit from a pump of that network (`UNIT_ENERGY` of steam
//!   a unit) when its water runs low. It stops burning while it holds `STEAM_CAP`.
//! - A steam turbine touching boilers (at most `TURBINES_PER_BOILER` turbines to a boiler, the lower
//!   processor indices first) is a power source on the grid of the pole it hangs on: `power.rs`
//!   asks it for what the grid lacks, up to `TURBINE_KW`, and it takes that much steam from its boilers.
//! - A silo (`Pick::Store`) is a big box with ports on every side: everything goes into its output buffer.
//!
//! Units are those of `power.rs`: steam and water are kW·ticks (1 kJ = `TICK_RATE`). What is derived
//! (`net`, `boilers`, `output`) is never saved; `water` and `steam` are, for boilers only.
//!
//! To add a steam machine: a spec row using these energies (`specs.rs`).

use rustc_hash::FxHashMap;

use crate::math::IVec3;
use crate::recipes::fuel_energy;
use crate::TICK_RATE;

use super::super::links::Slot;
use super::super::pipes::{Part, Pipework};
use super::super::Factory;
use super::super::FACES;
use super::{Energy, Pick, Processor, Status};

/// Steam a fuel item makes, as a multiple of the energy a generator gets from it.
pub const FUEL_GAIN: u32 = 2;
/// Steam a boiler holds at most: two coal's worth.
pub const STEAM_CAP: u32 = 270 * FUEL_GAIN * 2 * TICK_RATE;
/// What a pumped unit of water is worth as steam: 2,000 kJ.
pub const UNIT_ENERGY: u32 = 2_000 * TICK_RATE;
/// A boiler asks for a unit of water when it holds less than one coal's worth.
const WATER_LOW: u32 = 270 * FUEL_GAIN * TICK_RATE;
/// The most a steam turbine gives, in kW.
pub const TURBINE_KW: u32 = 240;
pub const TURBINES_PER_BOILER: u32 = 2;

/// A boiler's or turbine's state; other processors keep the default.
#[derive(Default)]
pub struct Steam {
    /// A boiler's steam and water, in kW·ticks.
    pub steam: u32,
    pub water: u32,
    /// A boiler's pipe network, if a pipe piece touches it (derived).
    pub net: Option<u32>,
    /// A turbine's boilers, as processor indices (derived).
    pub boilers: Vec<u32>,
    /// kW a turbine gave last tick (derived).
    pub output: u32,
}

impl Processor {
    /// One tick of a boiler: burns the first fuel item it holds if its steam has room and its water covers it.
    pub(super) fn boil(&mut self) {
        let mut why = None;
        match self.fuel.slots.iter().position(|s| !s.is_empty()) {
            Some(i) => {
                let e = fuel_energy(self.fuel.slots[i].item).map_or(0, |kj| kj * FUEL_GAIN * TICK_RATE);
                if self.steam.steam + e <= STEAM_CAP {
                    if self.steam.water >= e {
                        self.fuel.take(i, 1);
                        self.steam.water -= e;
                        self.steam.steam += e;
                    } else {
                        why = Some(Status::NoWater);
                    }
                }
            }
            None if self.steam.steam == 0 => why = Some(Status::NoFuel),
            None => {}
        }
        self.status = if self.steam.steam > 0 { Status::Working } else { why.unwrap_or(Status::NoInput) };
    }

    /// The status line of a boiler, turbine or silo (`None`: an ordinary processor).
    pub(super) fn steam_text(&self) -> Option<String> {
        let kj = |v: u32| v / TICK_RATE;
        match (self.energy(), self.spec.pick) {
            (Energy::Boiler, _) => Some(match self.status {
                Status::NoWater => "Out of water: pipe it to a pump with water in reach".to_string(),
                Status::NoFuel => "Out of fuel: bring coal ore or logs".to_string(),
                Status::Working => {
                    format!("Steam: {} kJ held · water for {} kJ more", kj(self.steam.steam), kj(self.steam.water))
                }
                _ => "Idle".to_string(),
            }),
            (Energy::Turbine, _) => Some(if self.steam.boilers.is_empty() {
                "No boiler: set it against a boiler's side".to_string()
            } else if self.steam.output > 0 {
                format!("Giving {} of {TURBINE_KW} kW", self.steam.output)
            } else if self.status == Status::NoFuel {
                "No steam: its boiler needs fuel and water".to_string()
            } else {
                "Standing by: the grid wants no more power".to_string()
            }),
            (_, Pick::Store) => {
                let used = self.out.slots.iter().filter(|s| !s.is_empty()).count();
                Some(format!("Holding {used} of {} stacks", self.out.slots.len()))
            }
            _ => None,
        }
    }
}

/// A turbine's grid line for its readout: whether it hangs on a pole, and what its grid draws.
pub(super) fn grid_line(p: &Processor, f: &Factory) -> Option<String> {
    let i = f.processors.iter().position(|q| q.pos == p.pos)?;
    let pole = f.power.process_pole.get(i).copied().flatten();
    Some(f.power.grid_line(pole))
}

/// Turbine `t` gives up to `want` kW (at most `TURBINE_KW`) from its boilers' steam. Returns what it gave
/// and what it could give if asked.
pub(in crate::factory) fn run_turbine(processors: &mut [Processor], t: usize, want: u32) -> (u32, u32) {
    let boilers = std::mem::take(&mut processors[t].steam.boilers);
    let mut left = want.min(TURBINE_KW);
    let mut ready = false;
    for &b in &boilers {
        let boiler = &mut processors[b as usize];
        ready |= boiler.steam.steam > 0 || (boiler.fuel.total() > 0 && boiler.status != Status::NoWater);
        let take = boiler.steam.steam.min(left);
        boiler.steam.steam -= take;
        left -= take;
    }
    let given = want.min(TURBINE_KW) - left;
    let me = &mut processors[t];
    me.steam.boilers = boilers;
    me.steam.output = given;
    me.status = if given > 0 {
        Status::Working
    } else if ready {
        Status::NoInput
    } else {
        Status::NoFuel
    };
    (given, if ready { TURBINE_KW } else { 0 })
}

/// Every boiler whose water is low takes a unit from a pump of its network that holds one.
pub(in crate::factory) fn draw_water(processors: &mut [Processor], pipework: &mut [Pipework]) {
    for b in processors.iter_mut().filter(|p| p.energy() == Energy::Boiler) {
        let Some(net) = b.steam.net.filter(|_| b.steam.water < WATER_LOW) else { continue };
        if let Some(pump) = pipework.iter_mut().find(|p| p.net == net && p.part == Part::Pump && p.held > 0) {
            pump.held -= 1;
            b.steam.water += UNIT_ENERGY;
        }
    }
}

/// Derives each boiler's pipe network and each turbine's boilers from what touches them (`relink`, after
/// `link_pipework`).
pub(in crate::factory) fn link(processors: &mut [Processor], at: &FxHashMap<IVec3, Slot>, pipework: &[Pipework]) {
    let mut taken = vec![0u32; processors.len()];
    let mut links = Vec::with_capacity(processors.len());
    for p in processors.iter() {
        let touching: Vec<Slot> =
            p.cells().iter().flat_map(|&c| FACES.iter().filter_map(move |&f| at.get(&(c + f)).copied())).collect();
        let mut boilers = Vec::new();
        let mut net = None;
        match p.energy() {
            Energy::Boiler => {
                net = touching.iter().find_map(|s| {
                    if let Slot::Pipe(j) = s {
                        Some(pipework[*j as usize].net)
                    } else {
                        None
                    }
                });
            }
            Energy::Turbine => {
                for s in &touching {
                    let Slot::Process(j) = *s else { continue };
                    let is_boiler = processors[j as usize].energy() == Energy::Boiler;
                    if is_boiler && !boilers.contains(&j) && taken[j as usize] < TURBINES_PER_BOILER {
                        taken[j as usize] += 1;
                        boilers.push(j);
                    }
                }
            }
            _ => {}
        }
        links.push((net, boilers));
    }
    for (p, (net, boilers)) in processors.iter_mut().zip(links) {
        (p.steam.net, p.steam.boilers) = (net, boilers);
    }
}

#[cfg(test)]
mod tests;
