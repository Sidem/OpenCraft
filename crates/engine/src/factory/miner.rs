//! Miner Mk1: drills the deposit behind its drill face, keeps [`MINER_RECOVERY`] of what it draws
//! as ore items, holds up to [`MINER_BUFFER`], and pushes them round-robin into belts leading away
//! or adjacent boxes (never through the drill face). The deposit's shared draw cap and taper
//! decide how much it actually gets (`deposits.rs`).

use crate::block::{BlockId, STONE};
use crate::bytes::ByteWriter;
use crate::deposits::{DepositKey, Deposits};
use crate::math::IVec3;
use crate::sim::SimEvent;
use crate::world::World;
use crate::TICK_RATE;

use super::belt::Belt;
use super::storage::Storage;
use super::{deliver, Link, FACES};

/// Ore units a Mk1 miner draws per second (before the deposit's taper and draw cap).
pub const MINER_RATE: f64 = 1.0;
/// Share of drawn units a Mk1 miner turns into ore items.
pub const MINER_RECOVERY: f64 = 0.6;
/// Ore a miner holds before it stops drilling.
pub const MINER_BUFFER: u32 = 64;
/// Ticks between `SimEvent::MinerWorking` reports while drawing (0.9 s; the view plays a drill sound).
const MINER_PULSE_TICKS: u32 = TICK_RATE * 9 / 10;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MinerStatus {
    Running,
    OutputFull,
    NoDeposit,
    Exhausted,
}

pub struct Miner {
    pub pos: IVec3,
    /// Face index (see [`FACES`]) pointing from the miner into the ore it was placed against.
    pub drill: u8,
    pub deposit: Option<DepositKey>,
    pub ore: BlockId,
    pub held: u32,
    /// Recovered ore not yet a whole item.
    pub carry: f64,
    pub outs: Vec<Link>,
    pub next_out: usize,
    pub status: MinerStatus,
    /// Smoothed draw in units per second, for the readout.
    pub draw_rate: f64,
    /// Ticks until the next `MinerWorking` report.
    pub pulse: u32,
}

impl Miner {
    pub fn new(pos: IVec3, drill: u8, deposit: Option<DepositKey>) -> Miner {
        Miner {
            pos,
            drill: drill.min(5),
            deposit,
            ore: deposit.map_or(STONE, |k| k.ore),
            held: 0,
            carry: 0.0,
            outs: Vec::new(),
            next_out: 0,
            status: if deposit.is_some() { MinerStatus::Running } else { MinerStatus::NoDeposit },
            draw_rate: 0.0,
            pulse: 0,
        }
    }

    /// Core state (`ore` follows from the deposit; `outs` is rebuilt by `relink`).
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.drill);
        w.bool(self.deposit.is_some());
        if let Some(key) = &self.deposit {
            key.write_state(w);
        }
        w.u32(self.held);
        w.f64(self.carry);
        w.u32(self.next_out as u32);
        w.u8(self.status as u8);
        w.f64(self.draw_rate);
        w.u32(self.pulse);
    }

    /// Draws from the deposit into `held`, then updates the status and the smoothed draw rate.
    /// Returns the units drawn this step.
    pub fn draw_step(&mut self, deposits: &mut Deposits, world: &mut World, tick: u64, dt: f64) -> f64 {
        let mut drawn = 0.0;
        match self.deposit {
            None => self.status = MinerStatus::NoDeposit,
            Some(key) => {
                if self.held < MINER_BUFFER {
                    let face = self.pos + FACES[self.drill as usize];
                    drawn = deposits.draw(world, &key, MINER_RATE, face, tick, dt);
                    self.carry += drawn * MINER_RECOVERY;
                    let whole = self.carry.floor();
                    self.held += whole as u32;
                    self.carry -= whole;
                }
                let exhausted = deposits.get(&key).is_none_or(|s| s.exhausted());
                self.status = if self.held >= MINER_BUFFER {
                    MinerStatus::OutputFull
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
    pub fn output_step(&mut self, belts: &mut [Belt], storages: &mut [Storage]) {
        if self.held == 0 || self.outs.is_empty() {
            return;
        }
        let n = self.outs.len();
        for i in 0..n {
            let slot = (self.next_out + i) % n;
            if deliver(belts, storages, self.outs[slot], self.ore, 0.0) {
                self.held -= 1;
                self.next_out = (slot + 1) % n;
                break;
            }
        }
    }

    /// While drawing, reports `MinerWorking` on the first tick and every [`MINER_PULSE_TICKS`] after.
    pub fn pulse_step(&mut self, drawn: f64, events: &mut Vec<SimEvent>) {
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
