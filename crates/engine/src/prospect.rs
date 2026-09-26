//! Prospecting: the scanner lists the ore deposits around the local player, and the core drill gives
//! exact figures for the deposits under one column.
//!
//! Invariants: both are queries, never actions. They read the core and survey untracked deposits into
//! throwaway copies (`DepositState::survey`), so the state hash never moves and co-op peers hear of
//! nothing. Scans leave out deposits that are worked out. Readings are flat `i32` records for the host
//! (`api/prospect.rs`, then `web/src/ui/prospect.ts`), and `seq` changes whenever a new one arrives.
//!
//! Driven by the hands (`interaction.rs`): with a device selected (`tools::device`), the use button
//! prospects instead of placing. To add a figure: append it to its record, bump `*_FIELDS`, and read
//! it in `ui/prospect.ts`.

use crate::block;
use crate::deposits::{Deposit, DepositState};
use crate::math::{sort_small_by_key, IVec3, Vec3};
use crate::sound;
use crate::tools::{self, ToolKind};
use crate::worldgen::WORLD_HEIGHT;
use crate::Game;

/// The scanner finds deposits whose centre lies within this many blocks horizontally.
pub const SCAN_RANGE: i32 = 48;
/// Seconds between scans while the use button is held.
const SCAN_COOLDOWN: f32 = 2.0;
/// Seconds of holding use on a block for a core sample.
const DRILL_SECONDS: f32 = 3.0;
/// A core sample covers deposits reaching within this many blocks of the drilled column.
const DRILL_REACH: i32 = 2;
const DRILL_SOUND_INTERVAL: f32 = 0.3;
/// Size bands, as the share of its tier's largest shape that a deposit still holds.
const BAND_MEDIUM: f32 = 0.35;
const BAND_LARGE: f32 = 0.65;
/// The largest radii product per tier (lode, vein, outcrop), from the shape ranges in `worldgen/ore.rs`.
const LARGEST: [f32; 3] = [460.0, 54.6, 12.2];

/// Scan record: ore, tier, dx, dz (from the player's feet), depth below the feet, size band (0 small,
/// 1 medium, 2 large).
pub const SCAN_FIELDS: usize = 6;
/// Core-sample record: ore, tier, blocks left, blocks at first, units left, top y, bottom y.
pub const DRILL_FIELDS: usize = 7;
pub const READING_SCAN: u8 = 1;
pub const READING_DRILL: u8 = 2;

/// The local player's latest reading and the devices' timers (presentation only).
#[derive(Default)]
pub struct Prospect {
    /// `READING_SCAN` or `READING_DRILL` (0 before the first reading).
    pub kind: u8,
    /// Scans list lodes first, then veins, then outcrops, nearest first; core samples go in ownership order.
    pub records: Vec<i32>,
    /// Where the reading was taken: the player's feet (scan) or the drilled block.
    pub origin: IVec3,
    /// Bumped with every new reading.
    pub seq: u32,
    cooldown: f32,
    /// The block being drilled and the seconds spent on it.
    drill: Option<(IVec3, f32)>,
    drill_sound: f32,
}

impl Prospect {
    /// 0..1 while a core sample is being drilled.
    pub fn drill_progress(&self) -> Option<f32> {
        self.drill.map(|(_, t)| t / DRILL_SECONDS)
    }
}

/// A deposit the scanner found.
pub struct Found {
    pub deposit: Deposit,
    /// 0 small, 1 medium, 2 large.
    pub band: u8,
}

/// Exact figures for one deposit under a drilled column.
pub struct Sample {
    pub deposit: Deposit,
    pub remaining_blocks: u32,
    pub initial_blocks: u32,
    pub remaining_units: u32,
    pub top: i32,
    pub bottom: i32,
}

impl Game {
    /// Runs the selected prospecting device for one tick of `dt` seconds. Returns whether one is
    /// selected (the use button is then the device's, not placing's).
    pub(crate) fn update_prospecting(&mut self, dt: f32) -> bool {
        self.prospect.cooldown = (self.prospect.cooldown - dt).max(0.0);
        let device = tools::device(self.inventory().selected_stack().item);
        if device != Some(ToolKind::CoreDrill) {
            self.prospect.drill = None;
        }
        match device {
            Some(ToolKind::Scanner) if self.using && self.prospect.cooldown == 0.0 => {
                self.prospect.cooldown = SCAN_COOLDOWN;
                let at = self.body().pos.floor();
                let found = self.scan(at);
                let r = &mut self.prospect.records;
                r.clear();
                for Found { deposit: d, band } in found {
                    let (dx, dz, depth) = (d.center.x - at.x, d.center.z - at.z, at.y - d.center.y);
                    r.extend_from_slice(&[d.ore() as i32, d.tier() as i32, dx, dz, depth, band as i32]);
                }
                self.new_reading(READING_SCAN, at);
            }
            Some(ToolKind::CoreDrill) => self.update_drill(dt),
            _ => {}
        }
        device.is_some()
    }

    /// Every deposit, not worked out, whose centre lies within [`SCAN_RANGE`] blocks horizontally of
    /// `at`: lodes first, then veins, then outcrops, each nearest first.
    pub(crate) fn scan(&mut self, at: IVec3) -> Vec<Found> {
        let lo = IVec3::new(at.x - SCAN_RANGE, 0, at.z - SCAN_RANGE);
        let hi = IVec3::new(at.x + SCAN_RANGE, WORLD_HEIGHT - 1, at.z + SCAN_RANGE);
        let mut found = Vec::new();
        for d in self.sim.world.generator_mut().deposits_touching(lo, hi) {
            if flat_dist2(d.center, at) > SCAN_RANGE * SCAN_RANGE {
                continue;
            }
            let left = match self.sim.factory.deposits.get(&d.key) {
                Some(st) if st.exhausted() => continue,
                Some(st) => st.remaining_blocks as f32 / st.initial_blocks.max(1) as f32,
                None => 1.0,
            };
            found.push(Found { band: band(&d, left), deposit: d });
        }
        sort_small_by_key(&mut found, |f| (f.deposit.tier(), flat_dist2(f.deposit.center, at)));
        found
    }

    /// Exact figures for every deposit whose shape reaches within [`DRILL_REACH`] blocks of the column
    /// from `top` down to bedrock, in ownership order.
    pub(crate) fn core_sample(&mut self, top: IVec3) -> Vec<Sample> {
        let lo = IVec3::new(top.x - DRILL_REACH, 0, top.z - DRILL_REACH);
        let hi = IVec3::new(top.x + DRILL_REACH, top.y, top.z + DRILL_REACH);
        let world = &mut self.sim.world;
        let mut out = Vec::new();
        for d in world.generator_mut().deposits_touching(lo, hi) {
            if !reaches(&d, lo, hi) {
                continue;
            }
            let surveyed;
            let st = match self.sim.factory.deposits.get(&d.key) {
                Some(st) => st,
                None => {
                    surveyed = DepositState::survey(world, d);
                    &surveyed
                }
            };
            if st.initial_blocks == 0 {
                continue; // a shape generation never filled (overlapped or in unreplaceable ground)
            }
            let (bottom, top) = st.y_span();
            out.push(Sample {
                deposit: d,
                remaining_blocks: st.remaining_blocks,
                initial_blocks: st.initial_blocks,
                remaining_units: st.remaining_units() as u32,
                top,
                bottom,
            });
        }
        out
    }

    /// Holding use on a block with the core drill: after [`DRILL_SECONDS`] on the same block, takes a
    /// core sample and releases the button.
    fn update_drill(&mut self, dt: f32) {
        let Some(hit) = self.target.filter(|_| self.using) else {
            self.prospect.drill = None;
            return;
        };
        let t = match self.prospect.drill {
            Some((b, t)) if b == hit.block => t + dt,
            _ => 0.0,
        };
        self.prospect.drill_sound -= dt;
        if self.prospect.drill_sound <= 0.0 {
            self.prospect.drill_sound = DRILL_SOUND_INTERVAL;
            let center = hit.block.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
            self.play(sound::DIG, block::def(hit.id).sound, center, 0.7);
        }
        if t < DRILL_SECONDS {
            self.prospect.drill = Some((hit.block, t));
            return;
        }
        self.prospect.drill = None;
        self.using = false;
        let samples = self.core_sample(hit.block);
        let r = &mut self.prospect.records;
        r.clear();
        for s in samples {
            let d = &s.deposit;
            let figures = [s.remaining_blocks as i32, s.initial_blocks as i32, s.remaining_units as i32];
            r.extend_from_slice(&[d.ore() as i32, d.tier() as i32]);
            r.extend_from_slice(&figures);
            r.extend_from_slice(&[s.top, s.bottom]);
        }
        self.new_reading(READING_DRILL, hit.block);
    }

    fn new_reading(&mut self, kind: u8, origin: IVec3) {
        let p = &mut self.prospect;
        p.kind = kind;
        p.origin = origin;
        p.seq = p.seq.wrapping_add(1);
    }
}

/// Size band of a deposit still holding `left` of its blocks: 0 small, 1 medium, 2 large.
fn band(d: &Deposit, left: f32) -> u8 {
    let share = d.radii[0] * d.radii[1] * d.radii[2] * left / LARGEST[d.tier() as usize];
    (share >= BAND_MEDIUM) as u8 + (share >= BAND_LARGE) as u8
}

/// Whether `d`'s shape takes in any block of the box `lo..=hi`.
fn reaches(d: &Deposit, lo: IVec3, hi: IVec3) -> bool {
    let (a, b) = d.bounds();
    let lo = IVec3::new(lo.x.max(a.x), lo.y.max(a.y), lo.z.max(a.z));
    let hi = IVec3::new(hi.x.min(b.x), hi.y.min(b.y), hi.z.min(b.z));
    (lo.y..=hi.y).any(|y| (lo.z..=hi.z).any(|z| (lo.x..=hi.x).any(|x| d.contains(IVec3::new(x, y, z)))))
}

fn flat_dist2(p: IVec3, at: IVec3) -> i32 {
    let (dx, dz) = (p.x - at.x, p.z - at.z);
    dx * dx + dz * dz
}

#[cfg(test)]
mod tests;
