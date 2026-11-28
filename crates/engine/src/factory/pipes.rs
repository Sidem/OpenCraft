//! Pipework: pumps, pipes and outlets, one machine kind (`Kind::Pipe`, like splitters and filters share
//! `Router`). Every piece joins the pieces on its six faces; each connected group is a network
//! (derived in `relink` by `link_pipework`, never saved). What moves water lives in `pumping.rs`; 3D models live
//! in `pipe_model.rs`.
//!
//! Invariants: a pump holds at most its tier's `hold` units of water, and a unit is always one source block
//! taken out of the world or put back into it (water is never made or lost in the pipes). `progress`
//! counts toward the next unit in 1/`UNIT`ths. `net`, `arms`, `peak_y`, `pressure` and `pouring` are derived.
//!
//! To add a piece: a `Part`, its byte, a `MACHINES` row sharing `Kind::Pipe`, and its arms in `step`
//! (`pumping.rs`), `describe` here and `pipe_model.rs`.

use crate::block::{OUTLET, PIPE, PUMP};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::math::{IVec3, Vec3};

use super::links::Slot;
use super::pumping::{pump_pressure, PumpTier, OUTLET_RATE, PUMP_RANGE, PUMP_TIERS};
use super::{Factory, Machine, DIRS, FACES};

/// Progress per unit of water: a unit per second at a rate of 1 takes `UNIT` / `TICK_RATE` a tick.
pub const UNIT: u32 = 60_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Pipe,
    Pump,
    Outlet,
}

/// What a pipe network carries (derived by `process::steam::link`): water, steam when a boiler's outlet or a
/// turbine's inlet is on it, or both (`Mixed`: it works for neither).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Fluid {
    #[default]
    Water,
    Steam,
    Mixed,
}

/// What a pump or outlet did last tick (for readouts and the lamp).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Flow {
    #[default]
    Idle,
    /// Pumping or pouring.
    Working,
    NoPower,
    /// A pump with no source block in reach.
    NoWater,
    /// A pump whose water nothing takes.
    Full,
    /// An outlet with nowhere to pour.
    Blocked,
    /// Pipe network rises above the pump's maximum lift (`max_head`).
    TooHigh,
}

pub struct Pipework {
    pub pos: IVec3,
    pub part: Part,
    /// A pump's tier (`PUMP_TIERS`); 0 for the other parts.
    pub tier: u8,
    /// An outlet's spout, as a `DIRS` index (the placing player's facing).
    pub facing: u8,
    pub progress: u32,
    /// A pump's water waiting for an outlet.
    pub held: u32,
    /// Active outlet spout flow level (0 = not pouring, 1..=7 = live falling stream at `front`; derived).
    pub pouring: u8,
    /// Network index (derived).
    pub net: u32,
    /// Highest `y` coordinate of any piece in `net` (derived in `link_pipework`).
    pub peak_y: i32,
    /// Current hydraulic pressure in permille `0..=1000` (derived).
    pub pressure: u16,
    /// What its network carries (derived).
    pub fluid: Fluid,
    /// Faces joined to another piece or to a boiler's or turbine's pipe port, as bits in `FACES` order
    /// (derived).
    pub arms: u8,
    pub flow: Flow,
}

impl Pipework {
    pub fn new(pos: IVec3, block: crate::block::BlockId, facing: u8) -> Pipework {
        let part = match block {
            PUMP => Part::Pump,
            OUTLET => Part::Outlet,
            _ => Part::Pipe,
        };
        Pipework {
            pos,
            part,
            tier: 0,
            facing: facing % 4,
            progress: 0,
            held: 0,
            pouring: 0,
            net: 0,
            peak_y: pos.y,
            pressure: if part == Part::Pump { 1000 } else { 0 },
            fluid: Fluid::Water,
            arms: 0,
            flow: Flow::Idle,
        }
    }

    /// Vertical blocks this piece's pipe network climbs above it.
    pub fn lift(&self) -> u32 {
        (self.peak_y - self.pos.y).max(0) as u32
    }

    /// Total electric power (kW) a pump draws at its current lift height (`base + lift * lift_power`).
    pub fn power_kw(&self) -> u32 {
        let t = self.pump_stats();
        t.power + self.lift().min(t.max_head) * t.lift_power
    }

    /// Whether a pump wants power this tick: it has room for more water and enough head for its pipework.
    pub fn wants_power(&self) -> bool {
        self.part == Part::Pump && self.held < self.pump_stats().hold && self.pressure > 0
    }

    /// What its tier lets it do (a pump's numbers; the other parts are Mk1).
    pub fn pump_stats(&self) -> &'static PumpTier {
        &PUMP_TIERS[self.tier as usize]
    }
}

impl Factory {
    /// Numbers the pipe networks (in order of their lowest piece), records each piece's arms, computes each
    /// network's peak height `peak_y` and refreshes each pump's head pressure.
    pub(super) fn link_pipework(&mut self) {
        let at = &self.at;
        let pipe_at = |q: IVec3| match at.get(&q) {
            Some(Slot::Pipe(j)) => Some(*j as usize),
            _ => None,
        };
        // A piece on a boiler's or turbine's pipe port reaches into it too (arms are drawing only).
        let port_at = |q: IVec3, from: IVec3| match at.get(&q) {
            Some(&Slot::Process(j)) => {
                self.processors[j as usize].pipe_ports().iter().any(|&(c, s, _)| c == q && c + DIRS[s as usize] == from)
            }
            _ => false,
        };
        let arms: Vec<u8> = self
            .pipework
            .iter()
            .map(|p| {
                let joined = |f: usize| pipe_at(p.pos + FACES[f]).is_some() || port_at(p.pos + FACES[f], p.pos);
                (0..6).filter(|&f| joined(f)).fold(0, |a, f| a | 1 << f)
            })
            .collect();
        let mut net = vec![u32::MAX; self.pipework.len()];
        let mut peak_y: Vec<i32> = Vec::new();
        for start in 0..net.len() {
            if net[start] != u32::MAX {
                continue;
            }
            let id = peak_y.len() as u32;
            let mut max_y = self.pipework[start].pos.y;
            net[start] = id;
            let mut stack = vec![start];
            while let Some(i) = stack.pop() {
                max_y = max_y.max(self.pipework[i].pos.y);
                for f in FACES {
                    if let Some(j) = pipe_at(self.pipework[i].pos + f).filter(|&j| net[j] == u32::MAX) {
                        net[j] = id;
                        stack.push(j);
                    }
                }
            }
            peak_y.push(max_y);
        }
        for ((p, a), n) in self.pipework.iter_mut().zip(arms).zip(net) {
            p.arms = a;
            p.net = n;
            p.peak_y = peak_y[n as usize];
            if p.part == Part::Pump {
                p.pressure = pump_pressure(p.pos.y, p.peak_y, p.pump_stats());
            }
        }
    }
}

impl Machine for Pipework {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.part as u8);
        w.u8(self.facing);
        w.u32(self.progress);
        w.u32(self.held);
        w.u8(self.tier);
    }

    fn read_state(r: &mut ByteReader) -> Option<Pipework> {
        let pos = r.ivec3()?;
        let block = match r.u8()? {
            0 => PIPE,
            1 => PUMP,
            2 => OUTLET,
            _ => return None,
        };
        let mut p = Pipework::new(pos, block, r.u8()?);
        (p.progress, p.held) = (r.u32()?.min(UNIT), r.u32()?);
        if r.version >= 21 {
            p.tier = r.u8()?;
        }
        if r.pipe_level_byte {
            r.u8()?; // an outlet's stream level, derived since
        }
        if p.tier as usize >= PUMP_TIERS.len() {
            return None;
        }
        p.held = p.held.min(p.pump_stats().hold);
        Some(p)
    }

    /// Water isn't an item: a picked-up pump loses what it held (at most its `hold` blocks).
    fn contents(&self) -> Vec<Stack> {
        Vec::new()
    }

    fn describe(&self, f: &Factory) -> String {
        if f.dirty {
            return String::new();
        }
        let net = f.pipework.iter().filter(|p| p.net == self.net);
        let (pumps, outlets) =
            net.fold((0, 0), |(a, b), p| (a + u32::from(p.part == Part::Pump), b + u32::from(p.part == Part::Outlet)));
        let network = match self.fluid {
            Fluid::Water => format!("Network: {pumps} pumps, {outlets} outlets"),
            Fluid::Steam => "Steam network".to_string(),
            Fluid::Mixed => "Water and steam share this network: keep them apart".to_string(),
        };
        match self.part {
            Part::Pipe if self.fluid == Fluid::Steam => {
                "Steam pipe\nCarries steam from a boiler's outlet to a turbine's inlet".to_string()
            }
            Part::Pipe => format!("{network}\nPipes join pumps to outlets and water inlets"),
            Part::Pump => {
                let tier = self.pump_stats();
                let lift = self.lift();
                let pct = self.pressure / 10;
                let rate_tenths = tier.rate * u32::from(self.pressure) / 100;
                let me = f.pipework.iter().position(|p| p.pos == self.pos);
                let pole = me.and_then(|i| f.power.pipe_pole.get(i).copied()).flatten();
                let status = match self.flow {
                    Flow::Working | Flow::Idle => format!(
                        "Pumping {}.{} blocks/s · lift {lift} / {} blocks ({pct}% pressure)",
                        rate_tenths / 10,
                        rate_tenths % 10,
                        tier.max_head
                    ),
                    Flow::TooHigh => {
                        format!(
                            "Too high: pipes rise {lift} blocks (Mk{} lifts up to {})",
                            self.tier + 1,
                            tier.max_head
                        )
                    }
                    Flow::NoPower => f.power.grid_line(pole),
                    Flow::NoWater => format!("No water: takes water within {PUMP_RANGE} blocks"),
                    Flow::Full | Flow::Blocked => {
                        format!("Full · lift {lift} / {} blocks ({pct}% pressure)", tier.max_head)
                    }
                };
                format!("{status}\nHolding {} of {} · needs {} kW\n{network}", self.held, tier.hold, self.power_kw())
            }
            Part::Outlet => {
                let pct = self.pressure / 10;
                let rate_tenths = OUTLET_RATE * u32::from(self.pressure) / 100;
                let status = match self.flow {
                    Flow::Working => {
                        format!("Pouring up to {}.{} blocks/s ({pct}% pressure)", rate_tenths / 10, rate_tenths % 10)
                    }
                    Flow::TooHigh => "No pressure: pipes rise too high for the pump".to_string(),
                    Flow::Blocked => "Nowhere to pour: the space in front is full".to_string(),
                    _ => "Waiting for water from a pump".to_string(),
                };
                format!("{status}\n{network}")
            }
        }
    }

    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        self.write_static(out, rel);
        self.write_dynamic(out, rel, time);
    }
}

#[cfg(test)]
mod tests;
