//! Miners (Mk1 and Mk2, one kind): drill the deposit behind the drill face, keep their recovery share
//! of what they draw as ore items in the output buffer (slots in `MACHINES`), stop drilling when that
//! is full, and push ore round-robin into belts leading away or adjacent boxes and smelters (never
//! through the drill face). The deposit's shared draw cap and taper decide how much it actually gets
//! (`deposits.rs`). The Mk1 is unpowered; the Mk2 draws faster, recovers more and runs at its grid's
//! `speed` (set before each step from `power.rs`).

use crate::block::{tex, STONE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::deposits::{DepositKey, Deposits};
use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::math::{IVec3, Vec3};
use crate::sim::SimEvent;
use crate::world::World;
use crate::{TICK, TICK_RATE};

use super::belt::Belt;
use super::buffer::Buffer;
use super::describe::{fmt_duration, fmt_int};
use super::links::{deliver, Link, Sinks};
use super::power::{FULL_SPEED, POLE_REACH};
use super::render::push_box;
use super::{Factory, Kind, Machine, FACES};

/// Ore units a Mk1 miner draws per second (before the deposit's taper and draw cap).
pub const MINER_RATE: f64 = 1.0;
/// Share of drawn units a Mk1 miner turns into ore items.
pub const MINER_RECOVERY: f64 = 0.6;
/// The Mk2's draw at full power and its recovery: upgrading gets more ore out of the same deposit.
pub const MK2_RATE: f64 = 2.0;
pub const MK2_RECOVERY: f64 = 0.75;
/// Ticks between `SimEvent::MinerWorking` reports while drawing (0.9 s; the view plays a drill sound).
const MINER_PULSE_TICKS: u32 = TICK_RATE * 9 / 10;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MinerStatus {
    Running,
    OutputFull,
    NoDeposit,
    Exhausted,
    NoPower,
}

/// Every status, in declaration order: saves store `status as u8`.
const STATUSES: [MinerStatus; 5] = [
    MinerStatus::Running,
    MinerStatus::OutputFull,
    MinerStatus::NoDeposit,
    MinerStatus::Exhausted,
    MinerStatus::NoPower,
];

pub struct Miner {
    pub pos: IVec3,
    /// Face index (see [`FACES`]) pointing from the miner into the ore it was placed against.
    pub drill: u8,
    pub deposit: Option<DepositKey>,
    /// The deposit's ore, the only item in `out`.
    pub ore: ItemId,
    pub out: Buffer,
    /// Recovered ore not yet a whole item.
    pub carry: f64,
    pub outs: Vec<Link>,
    pub next_out: usize,
    pub status: MinerStatus,
    /// Smoothed draw in units per second, for the readout.
    pub draw_rate: f64,
    /// Ticks until the next `MinerWorking` report.
    pub pulse: u32,
    pub mk2: bool,
    /// This tick's speed from its grid, in thousandths (Mk2 only; derived).
    pub speed: u32,
}

impl Miner {
    pub fn new(pos: IVec3, drill: u8, deposit: Option<DepositKey>, mk2: bool) -> Miner {
        Miner {
            pos,
            drill: drill.min(5),
            deposit,
            ore: ItemId::block(deposit.map_or(STONE, |k| k.ore)),
            out: Buffer::new(Kind::Miner.def().slots),
            carry: 0.0,
            outs: Vec::new(),
            next_out: 0,
            status: if deposit.is_some() { MinerStatus::Running } else { MinerStatus::NoDeposit },
            draw_rate: 0.0,
            pulse: 0,
            mk2,
            speed: 0,
        }
    }

    /// Units a second it draws at full power.
    pub fn rate(&self) -> f64 {
        if self.mk2 {
            MK2_RATE
        } else {
            MINER_RATE
        }
    }

    /// Share of what it draws that becomes ore items.
    pub fn recovery(&self) -> f64 {
        if self.mk2 {
            MK2_RECOVERY
        } else {
            MINER_RECOVERY
        }
    }

    /// Whether it would draw this tick if powered (a Mk2's grid counts it as demand).
    pub fn wants_power(&self) -> bool {
        self.mk2 && self.deposit.is_some() && self.out.can_accept(self.ore) && self.status != MinerStatus::Exhausted
    }

    /// One tick: draw from the deposit, push one item on, report the drilling.
    pub fn step(
        &mut self,
        deposits: &mut Deposits,
        world: &mut World,
        tick: u64,
        belts: &mut [Belt],
        sinks: &mut Sinks,
        events: &mut Vec<SimEvent>,
    ) {
        let drawn = self.draw(deposits, world, tick);
        self.push_out(belts, sinks);
        self.pulse(drawn, events);
    }

    /// Draws from the deposit into `out`, then updates the status and the smoothed draw rate.
    /// Returns the units drawn.
    fn draw(&mut self, deposits: &mut Deposits, world: &mut World, tick: u64) -> f64 {
        let dt = TICK;
        let mut drawn = 0.0;
        match self.deposit {
            None => self.status = MinerStatus::NoDeposit,
            Some(key) => {
                let speed = if self.mk2 { self.speed } else { FULL_SPEED };
                let room = self.out.can_accept(self.ore);
                if room && speed > 0 {
                    let face = self.pos + FACES[self.drill as usize];
                    let rate = self.rate() * speed as f64 / FULL_SPEED as f64;
                    drawn = deposits.draw(world, &key, rate, face, tick, dt);
                    self.carry += drawn * self.recovery();
                    let whole = self.carry.floor();
                    self.out.add(self.ore, whole as u32);
                    self.carry -= whole;
                }
                let exhausted = deposits.get(&key).is_none_or(|s| s.exhausted());
                self.status = if !self.out.can_accept(self.ore) {
                    MinerStatus::OutputFull
                } else if speed == 0 {
                    MinerStatus::NoPower
                } else if exhausted && drawn == 0.0 {
                    MinerStatus::Exhausted
                } else {
                    MinerStatus::Running
                };
            }
        }
        // A new miner's average starts from its first reading, so the readout is right at once.
        let rate = drawn / dt.max(1e-6);
        if self.draw_rate == 0.0 {
            self.draw_rate = rate;
        } else {
            self.draw_rate += (rate - self.draw_rate) * (dt / 2.0).min(1.0);
        }
        drawn
    }

    /// Pushes one held item into the next output that accepts it (round-robin).
    fn push_out(&mut self, belts: &mut [Belt], sinks: &mut Sinks) {
        if self.out.total() == 0 || self.outs.is_empty() {
            return;
        }
        let n = self.outs.len();
        for i in 0..n {
            let slot = (self.next_out + i) % n;
            if deliver(belts, sinks, self.outs[slot], self.ore, 0.0) {
                self.out.take(0, 1);
                self.next_out = (slot + 1) % n;
                break;
            }
        }
    }

    /// While drawing, reports `MinerWorking` on the first tick and every [`MINER_PULSE_TICKS`] after.
    fn pulse(&mut self, drawn: f64, events: &mut Vec<SimEvent>) {
        if drawn <= 0.0 {
            return;
        }
        if self.pulse == 0 {
            self.pulse = MINER_PULSE_TICKS;
            events.push(SimEvent::MinerWorking { pos: self.pos + FACES[self.drill as usize] });
        }
        self.pulse -= 1;
    }
}

impl Machine for Miner {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state (`ore` follows from the deposit, `out` is written as its count; `outs` is rebuilt
    /// by `relink`).
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.drill);
        w.bool(self.deposit.is_some());
        if let Some(key) = &self.deposit {
            key.write_state(w);
        }
        w.u32(self.out.total());
        w.f64(self.carry);
        w.u32(self.next_out as u32);
        w.u8(self.status as u8);
        w.f64(self.draw_rate);
        w.u32(self.pulse);
        w.bool(self.mk2);
    }

    fn read_state(r: &mut ByteReader) -> Option<Miner> {
        let (pos, drill) = (r.ivec3()?, r.u8()?);
        let deposit = if r.bool()? { Some(DepositKey::read_state(r)?) } else { None };
        let mut m = Miner::new(pos, drill, deposit, false);
        let held = r.u32()?;
        m.carry = r.f64()?;
        m.next_out = r.u32()? as usize;
        m.status = *STATUSES.get(r.u8()? as usize)?;
        m.draw_rate = r.f64()?;
        m.pulse = r.u32()?;
        if r.version >= 9 {
            m.mk2 = r.bool()?;
        }
        (drill < 6 && m.out.add(m.ore, held) == 0).then_some(m)
    }

    fn contents(&self) -> Vec<Stack> {
        self.out.contents()
    }

    /// Deposit, status, what it holds and how long the deposit lasts at the current draw.
    fn describe(&self, f: &Factory) -> String {
        let Some(key) = self.deposit else {
            return "Not on an ore deposit. Place miners against an ore block.".to_string();
        };
        let Some(st) = f.deposits.get(&key) else { return String::new() };
        let mut lines = vec![format!(
            "{} · {} of {} blocks left",
            st.deposit.name(),
            fmt_int(st.remaining_blocks as u64),
            fmt_int(st.initial_blocks as u64)
        )];
        lines.push(match self.status {
            MinerStatus::Running if self.draw_rate > 0.01 => format!(
                "Running · {} ore/min ({}% recovery)",
                (self.draw_rate * self.recovery() * 60.0).round() as u32,
                (self.recovery() * 100.0).round() as u32
            ),
            MinerStatus::Running => "Waiting: other miners are using this deposit's full draw".to_string(),
            MinerStatus::OutputFull => "Output full: put a belt leading away, or a box, next to it".to_string(),
            MinerStatus::Exhausted => "Deposit worked out".to_string(),
            MinerStatus::NoPower => {
                format!("No power: needs a power pole within {POLE_REACH} blocks, linked to a generator")
            }
            MinerStatus::NoDeposit => String::new(),
        });
        let held = self.out.total();
        if held > 0 {
            lines.push(format!("Holding {held} {} · right-click to take", item::name(self.ore)));
        }
        let total_draw: f64 = f.miners.iter().filter(|o| o.deposit == Some(key)).map(|o| o.draw_rate).sum();
        if total_draw > 0.01 && !st.exhausted() {
            lines.push(format!(
                "{} units left · about {} at the current draw",
                fmt_int(st.remaining_units() as u64),
                fmt_duration(st.remaining_units() / total_draw)
            ));
        }
        lines.join("\n")
    }

    /// A housing, a collar and a drill pumping into the ore, with a status lamp on the back.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        let f = FACES[self.drill as usize].as_vec3();
        let axis = self.drill as usize / 2;
        let size = |along: f32, across: f32| {
            let mut s = [across; 3];
            s[axis] = along;
            s
        };
        let running = self.status == MinerStatus::Running && self.draw_rate > 0.01;
        let pump = if running { 0.05 * (0.5 + 0.5 * (time * 10.0).sin()) } else { 0.0 };
        let side = if self.mk2 { tex::MINER_MK2_SIDE } else { tex::MINER_SIDE };
        let housing = [tex::MINER_TOP, side, tex::FRAME];
        push_box(out, rel + f * -0.13, 0.0, size(0.7, 0.86), 0.0, housing, true);
        push_box(out, rel + f * 0.27, 0.0, size(0.1, 0.6), 0.0, [tex::FRAME; 3], true);
        push_box(out, rel + f * (0.44 + pump), 0.0, size(0.36, 0.22), 0.0, [tex::DRILL; 3], true);
        let lamp = match self.status {
            MinerStatus::Running => tex::LAMP_GREEN,
            MinerStatus::OutputFull => tex::LAMP_YELLOW,
            MinerStatus::NoDeposit | MinerStatus::Exhausted | MinerStatus::NoPower => tex::LAMP_RED,
        };
        push_box(out, rel + f * -0.5, 0.0, size(0.04, 0.2), 0.0, [lamp; 3], false);
    }
}
