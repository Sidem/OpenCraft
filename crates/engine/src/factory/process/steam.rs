//! Steam power and bulk storage: the processors that are not recipe machines (`Energy::Boiler`,
//! `Energy::Turbine`, `Pick::Store`).
//!
//! - A boiler burns fuel from its fuel buffer into steam (`FUEL_GAIN` times a generator's energy from
//!   the same item), and each item it burns needs water: a pipe piece touching one of its water inlets puts
//!   it on a pipe network (each touch is a tap, drawn by `steam_view.rs`), and `draw_water` takes a pumped
//!   unit from a pump of one of them (`UNIT_ENERGY` of steam a unit) when its water runs low. It stops
//!   burning while it holds `STEAM_CAP`. Its steam leaves through its two outlets into pipes.
//! - A steam turbine with its steam inlet on a pipe network that also holds a boiler's outlet (at most
//!   `TURBINES_PER_BOILER` turbines to a boiler, the lower processor indices first) is a power source on the
//!   grid of the pole it hangs on: `power.rs` asks it for what the grid lacks, up to `TURBINE_KW`, and it
//!   takes that much steam from its boilers.
//! - A pipe network is water or steam by what is on it: a network holding a pump, an outlet or a water inlet
//!   and a steam port too is `Fluid::Mixed` and works for neither (`link`).
//! - A silo (`Pick::Store`) is a big box with ports on every side: everything goes into its output buffer.
//!
//! Units are those of `power.rs`: steam and water are kW·ticks (1 kJ = `TICK_RATE`). What is derived
//! (`net`, `boilers`, `output`) is never saved; `water` and `steam` are, for boilers only.
//!
//! To add a steam machine: a spec row using these energies (`specs.rs`).

use rustc_hash::FxHashMap;

use crate::bytes::{ByteReader, ByteWriter};
use crate::math::IVec3;
use crate::recipes::{fuel_energy, water_use};
use crate::TICK_RATE;

use super::super::footprint::Role;
use super::super::links::Slot;
use super::super::pipes::{Fluid, Part, Pipework};
use super::super::{Factory, DIRS};
use super::nuclear::{COOLANT_LOW, COOLANT_UNIT};
use super::{Energy, Pick, Processor, Status};

/// Steam a fuel item makes, as a multiple of the energy a generator gets from it.
pub const FUEL_GAIN: u32 = 2;
/// Steam a boiler holds at most: two coal's worth.
pub const STEAM_CAP: u32 = 270 * FUEL_GAIN * 2 * TICK_RATE;
/// What a pumped unit of water is worth as steam: 2,000 kJ.
pub const UNIT_ENERGY: u32 = 2_000 * TICK_RATE;
/// A boiler asks for a unit of water when it holds less than one coal's worth.
pub(super) const WATER_LOW: u32 = 270 * FUEL_GAIN * TICK_RATE;
/// A water-using machine asks for a unit when its tank holds fewer: it keeps up to this many.
pub(super) const MACHINE_WATER: u32 = 4;
/// The most a steam turbine gives, in kW.
pub const TURBINE_KW: u32 = 240;
pub const TURBINES_PER_BOILER: u32 = 2;

/// Where a pipe piece touches a machine: the machine's cell and the direction of the piece.
pub type Tap = (IVec3, IVec3);

/// A boiler's or turbine's state; other processors keep the default.
#[derive(Default)]
pub struct Steam {
    /// A boiler's steam and water, in kW·ticks.
    pub steam: u32,
    pub water: u32,
    /// The water networks a boiler's water inlets are piped to, and where (derived; `steam_view.rs` draws
    /// the fittings).
    pub nets: Vec<u32>,
    pub taps: Vec<Tap>,
    /// The same for the steam a boiler's outlets or a turbine's inlets are piped to.
    pub vent_nets: Vec<u32>,
    pub vents: Vec<Tap>,
    /// A tap is on a pipe network that carries both water and steam (derived).
    pub crossed: bool,
    /// A turbine's boilers, as processor indices (derived).
    pub boilers: Vec<u32>,
    /// kW a turbine gave last tick (derived).
    pub output: u32,
}

impl Processor {
    /// A boiler's or turbine's pipe connections: the cell, the `DIRS` index it faces out of and `Role::Water`
    /// or `Role::Steam` (none for other processors).
    pub(in crate::factory) fn pipe_ports(&self) -> Vec<(IVec3, u8, Role)> {
        if !self.spec.footprint.ports.iter().any(|p| matches!(p.role, Role::Water | Role::Steam)) {
            return Vec::new();
        }
        let faces = |role| self.spec.footprint.faces(self.pos, self.dir, role);
        [Role::Water, Role::Steam].into_iter().flat_map(|r| faces(r).into_iter().map(move |(c, s)| (c, s, r))).collect()
    }

    /// Whether it has a water inlet: a boiler, or a machine whose recipes use water (`recipes::water_use`).
    pub fn takes_water(&self) -> bool {
        self.spec.footprint.ports.iter().any(|p| p.role == Role::Water)
    }

    /// Whether a batch of recipe `i` is short of water (a machine's tank holds whole units).
    pub(super) fn short_of_water(&self, i: u16) -> bool {
        self.steam.water < water_use(i)
    }

    /// Writes a boiler's steam and water, or a water-using machine's tank (`Boiler`'s are kW·ticks, a machine's
    /// are units).
    pub(super) fn write_tanks(&self, w: &mut ByteWriter) {
        if self.energy() == Energy::Boiler {
            w.u32(self.steam.steam);
        }
        if self.takes_water() {
            w.u32(self.steam.water);
        }
    }

    pub(super) fn read_tanks(&mut self, r: &mut ByteReader) -> Option<()> {
        if self.energy() == Energy::Boiler {
            self.steam.steam = r.u32()?.min(STEAM_CAP);
        }
        if self.takes_water() {
            self.steam.water = r.u32()?;
        }
        Some(())
    }

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
            (Energy::Boiler | Energy::Turbine, _) if self.steam.crossed => {
                Some("Water and steam share a pipe network: keep them apart".to_string())
            }
            (Energy::Boiler, _) => Some(match self.status {
                Status::NoWater => "Out of water: pipe a water inlet to a pump with water in reach".to_string(),
                Status::NoFuel => "Out of fuel: bring coal ore or logs".to_string(),
                Status::Working => {
                    format!("Steam: {} kJ held · water for {} kJ more", kj(self.steam.steam), kj(self.steam.water))
                }
                _ => "Idle".to_string(),
            }),
            (Energy::Turbine, _) => Some(if self.steam.boilers.is_empty() {
                "No boiler: pipe its steam inlet to a boiler's steam outlet".to_string()
            } else if self.steam.output > 0 {
                format!("Giving {} of {TURBINE_KW} kW", self.steam.output)
            } else if self.status == Status::NoFuel {
                "No steam: its boiler needs fuel and water".to_string()
            } else {
                "Standing by: the grid wants no more power".to_string()
            }),
            (_, pick) if pick.stores() => {
                let used = self.out.slots.iter().filter(|s| !s.is_empty()).count();
                let job = match pick {
                    Pick::Load => " · a train beside it takes them",
                    Pick::Unload => " · a train beside it unloads into it",
                    _ => "",
                };
                Some(format!("Holding {used} of {} stacks{job}", self.out.slots.len()))
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

/// Every boiler or machine whose water is low takes a unit from a pump of its network that holds one.
pub(in crate::factory) fn draw_water(processors: &mut [Processor], pipework: &mut [Pipework]) {
    for b in processors.iter_mut().filter(|p| p.takes_water()) {
        let (low, unit) = match b.energy() {
            Energy::Boiler => (WATER_LOW, UNIT_ENERGY),
            Energy::Reactor => (COOLANT_LOW, COOLANT_UNIT),
            _ => (MACHINE_WATER, 1),
        };
        if b.steam.water >= low {
            continue;
        }
        let nets = &b.steam.nets;
        if let Some(pump) = pipework.iter_mut().find(|p| p.part == Part::Pump && p.held > 0 && nets.contains(&p.net)) {
            pump.held -= 1;
            b.steam.water += unit;
        }
    }
}

/// Derives what each boiler's and turbine's pipe ports are piped to, each pipe network's fluid and each turbine's
/// boilers (`relink`, after `link_pipework`).
pub(in crate::factory) fn link(processors: &mut [Processor], at: &FxHashMap<IVec3, Slot>, pipework: &mut [Pipework]) {
    // A tap is a pipe piece on a port's face: (net, tap) by role.
    let taps_of = |p: &Processor, role: Role| -> Vec<(u32, Tap)> {
        let found = p.pipe_ports().into_iter().filter(|&(_, _, r)| r == role);
        let found = found.filter_map(|(c, s, _)| match at.get(&(c + DIRS[s as usize])) {
            Some(&Slot::Pipe(j)) => Some((pipework[j as usize].net, (c, DIRS[s as usize]))),
            _ => None,
        });
        found.collect()
    };
    let found: Vec<[Vec<(u32, Tap)>; 2]> =
        processors.iter().map(|p| [taps_of(p, Role::Water), taps_of(p, Role::Steam)]).collect();

    // A network is water when a pump, an outlet or a water inlet is on it, steam when a steam port is.
    let nets = pipework.iter().map(|p| p.net as usize + 1).max().unwrap_or(0);
    let (mut water, mut steam) = (vec![false; nets], vec![false; nets]);
    for p in pipework.iter().filter(|p| p.part != Part::Pipe) {
        water[p.net as usize] = true;
    }
    for [w, s] in &found {
        w.iter().for_each(|&(n, _)| water[n as usize] = true);
        s.iter().for_each(|&(n, _)| steam[n as usize] = true);
    }
    for p in pipework.iter_mut() {
        let n = p.net as usize;
        p.fluid = match (water[n], steam[n]) {
            (true, true) => Fluid::Mixed,
            (false, true) => Fluid::Steam,
            _ => Fluid::Water,
        };
    }
    let clean = |n: u32| !(water[n as usize] && steam[n as usize]);

    let boilers: Vec<usize> = (0..processors.len()).filter(|&i| processors[i].energy() == Energy::Boiler).collect();
    let mut taken = vec![0u32; processors.len()];
    let mut seats: Vec<Vec<u32>> = vec![Vec::new(); processors.len()];
    for (t, [_, vents]) in found.iter().enumerate().filter(|&(t, _)| processors[t].energy() == Energy::Turbine) {
        for &b in &boilers {
            let shared = found[b][1].iter().any(|&(n, _)| clean(n) && vents.iter().any(|&(m, _)| m == n));
            if shared && taken[b] < TURBINES_PER_BOILER {
                taken[b] += 1;
                seats[t].push(b as u32);
            }
        }
    }
    for ((p, [w, s]), boilers) in processors.iter_mut().zip(found).zip(seats) {
        let nets_of = |taps: &[(u32, Tap)]| {
            let mut nets = Vec::new();
            for &(n, _) in taps {
                if clean(n) && !nets.contains(&n) {
                    nets.push(n);
                }
            }
            nets
        };
        p.steam.crossed = w.iter().chain(&s).any(|&(n, _)| !clean(n));
        (p.steam.nets, p.steam.vent_nets) = (nets_of(&w), nets_of(&s));
        p.steam.taps = w.into_iter().map(|t| t.1).collect();
        p.steam.vents = s.into_iter().map(|t| t.1).collect();
        p.steam.boilers = boilers;
    }
}

#[cfg(test)]
mod tests;
