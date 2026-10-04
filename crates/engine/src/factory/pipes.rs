//! Pipework: pumps, pipes and outlets, one machine kind (`Kind::Pipe`, like splitters and filters share
//! `Router`). Every piece joins the pieces on its six faces; each connected group is a network
//! (derived in `relink` by `link_pipework`, never saved). What moves water lives in `pumping.rs`.
//!
//! Invariants: a pump holds at most its tier's `hold` units of water, and a unit is always one source block
//! taken out of the world or put back into it (water is never made or lost in the pipes). `progress`
//! counts toward the next unit in 1/`UNIT`ths. `net` and `arms` are derived; `status` is last tick's.
//!
//! To add a piece: a `Part`, its byte, a `MACHINES` row sharing `Kind::Pipe`, and its arms in `step`
//! (`pumping.rs`), `describe` and `model` here.

use crate::block::{tex, OUTLET, PIPE, PUMP};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::math::{IVec3, Vec3};

use super::links::Slot;
use super::pumping::{PumpTier, OUTLET_RATE, PUMP_RANGE, PUMP_TIERS};
use super::render::push_box;
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
    /// Network index (derived).
    pub net: u32,
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
            net: 0,
            fluid: Fluid::Water,
            arms: 0,
            flow: Flow::Idle,
        }
    }

    /// Whether a pump wants power this tick: it has room for more water.
    pub fn wants_power(&self) -> bool {
        self.part == Part::Pump && self.held < self.pump_stats().hold
    }

    /// What its tier lets it do (a pump's numbers; the other parts are Mk1).
    pub fn pump_stats(&self) -> &'static PumpTier {
        &PUMP_TIERS[self.tier as usize]
    }
}

impl Factory {
    /// Numbers the pipe networks (in order of their lowest piece) and records each piece's arms.
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
        let mut nets = 0;
        for start in 0..net.len() {
            if net[start] != u32::MAX {
                continue;
            }
            net[start] = nets;
            let mut stack = vec![start];
            while let Some(i) = stack.pop() {
                for f in FACES {
                    if let Some(j) = pipe_at(self.pipework[i].pos + f).filter(|&j| net[j] == u32::MAX) {
                        net[j] = nets;
                        stack.push(j);
                    }
                }
            }
            nets += 1;
        }
        for ((p, a), n) in self.pipework.iter_mut().zip(arms).zip(net) {
            (p.arms, p.net) = (a, n);
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
                let me = f.pipework.iter().position(|p| p.pos == self.pos);
                let pole = me.and_then(|i| f.power.pipe_pole.get(i).copied()).flatten();
                let status = match self.flow {
                    Flow::Working | Flow::Idle => format!("Pumping {} blocks of water a second", tier.rate),
                    Flow::NoPower => f.power.grid_line(pole),
                    Flow::NoWater => format!("No water: it takes still water within {PUMP_RANGE} blocks of it"),
                    Flow::Full | Flow::Blocked => "Full: pipe it to an outlet that can pour".to_string(),
                };
                format!("{status}\nHolding {} of {} · needs {} kW\n{network}", self.held, tier.hold, tier.power)
            }
            Part::Outlet => {
                let status = match self.flow {
                    Flow::Working => format!("Pouring up to {OUTLET_RATE} blocks of water a second"),
                    Flow::Blocked => "Nowhere to pour: the space in front is full".to_string(),
                    _ => "Waiting for water from a pump".to_string(),
                };
                format!("{status}\n{network}")
            }
        }
    }

    /// Pipes along every joined face (blue banded for water, pale and red banded for steam); a pump housing or
    /// an outlet spout around them.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        let steel = [tex::STEEL; 3];
        let pipe = [match self.fluid {
            Fluid::Water => tex::PIPE_WATER,
            Fluid::Steam => tex::PIPE_STEAM,
            Fluid::Mixed => tex::LAMP_RED,
        }; 3];
        for (f, &n) in FACES.iter().enumerate() {
            if self.arms & (1 << f) != 0 {
                let arm = n.as_vec3() * 0.3;
                let size =
                    [0.24 + 0.36 * n.x.abs() as f32, 0.24 + 0.36 * n.y.abs() as f32, 0.24 + 0.36 * n.z.abs() as f32];
                push_box(out, rel + arm, 0.0, size, 0.0, pipe, false);
            }
        }
        match self.part {
            Part::Pipe => push_box(out, rel, 0.0, [0.3; 3], 0.0, pipe, false),
            Part::Pump => {
                let body = [tex::STEEL, tex::GENERATOR_SIDE, tex::FRAME];
                push_box(out, rel + Vec3::new(0.0, -0.1, 0.0), 0.0, [0.8, 0.8, 0.8], 0.0, body, false);
                if self.tier > 0 {
                    let band = [tex::stripe(self.tier); 3];
                    push_box(out, rel + Vec3::new(0.0, -0.36, 0.0), 0.0, [0.84, 0.1, 0.84], 0.0, band, false);
                }
                let bob = if self.flow == Flow::Working { (time * 6.0).sin() * 0.05 } else { 0.0 };
                push_box(out, rel + Vec3::new(0.0, 0.36 + bob, 0.0), 0.0, [0.3, 0.12, 0.3], 0.0, steel, false);
                push_box(
                    out,
                    rel + Vec3::new(0.3, 0.18, 0.41),
                    0.0,
                    [0.12, 0.08, 0.02],
                    0.0,
                    [lamp(self.flow); 3],
                    false,
                );
            }
            Part::Outlet => {
                let d = DIRS[self.facing as usize].as_vec3();
                push_box(out, rel, 0.0, [0.5; 3], 0.0, steel, false);
                let yaw = d.x.atan2(-d.z) as f32;
                push_box(out, rel + d * 0.4, yaw, [0.3, 0.3, 0.3], 0.0, [tex::FRAME; 3], false);
                if self.flow == Flow::Working {
                    let stream = rel + d * 0.62 - Vec3::new(0.0, 0.25, 0.0);
                    push_box(out, stream, yaw, [0.18, 0.5, 0.14], (time * 2.0) as f32, [tex::WATER; 3], false);
                }
                push_box(out, rel + Vec3::new(0.0, 0.3, 0.0), 0.0, [0.1, 0.1, 0.1], 0.0, [lamp(self.flow); 3], false);
            }
        }
    }
}

/// The status lamp: green working, yellow full or blocked, red without power or water.
fn lamp(flow: Flow) -> u16 {
    match flow {
        Flow::Working => tex::LAMP_GREEN,
        Flow::Full | Flow::Blocked => tex::LAMP_YELLOW,
        Flow::NoPower | Flow::NoWater => tex::LAMP_RED,
        Flow::Idle => tex::FRAME,
    }
}

#[cfg(test)]
mod tests;
