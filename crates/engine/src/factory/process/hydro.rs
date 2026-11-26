//! The water wheel (Milestone 10's hydro): an upright 3×1×3 processor that is a power source like a solar panel
//! (`solar.rs`), driven only by falling water pouring down one side of the wheel (left or right). Each full `FLOW_7`
//! falling-water cell on the driving side gives `FALL_KW` (10 kW), up to `MAX_KW` (30 kW for a full 3-block fall),
//! scaled linearly by the water's flow level (`FLOW_1..=FLOW_7`) and split evenly if multiple wheels touch the same
//! falling column. Still water, flat horizontal flow, or falling water on both sides at once (opposing torques
//! cancelling out) produce 0 kW and leave the wheel stationary.
//!
//! Thermodynamics: pump lift power (`pumping.rs`, 7 kW per block/s·block) strictly exceeds maximum hydro recovery
//! (`<= 5.71 kW` per block/s·block), so pumped loops are always a net energy loss and decay to 0 kW without external
//! power — no artificial water tags needed.
//!
//! Presentation (`hydro/view.rs`, `Look::WaterWheel`): the static timber A-frame trestles and bearings live in
//! `WHEEL_PARTS`; the circular wooden wheel, axle, spokes, shroud rims, sole-board drum, 12 paddles, water-filled
//! descending buckets and splash spray are drawn dynamically and spin smoothly in the direction pushed by the water.
//!
//! Invariants: `sense` reads the world with `block_anywhere_or_generate` every tick and keeps nothing saved
//! (`Store::flow`, `hydro_mask`, `wheel_angle`, `wheel_spin` are derived), so a loaded game and a running one agree.

use rustc_hash::FxHashMap;

use crate::block::{flow_level, tex, LIQUID, WATER_WHEEL};
use crate::math::IVec3;
use crate::world::World;
use crate::TICK;

use super::super::footprint::{Footprint, Port};
use super::super::power::Power;
use super::super::power_mix::Source;
use super::super::DIRS;
use super::model::{leaned, part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

mod view;

pub(super) use view::draw as draw_wheel;

/// kW for each full (`FLOW_7`) falling-water block along the driving side of the wheel (3 cells high = `MAX_KW`).
pub const FALL_KW: u32 = 10;
/// The most a wheel gives, in kW.
pub const MAX_KW: u32 = 30;
/// Seconds per full revolution at `MAX_KW`, and the share of the speed gap closed each tick (smooth wind-up/coast).
const SPIN_SECONDS: f64 = 3.2;
const SPIN_EASE: f64 = 0.045;
const UP: IVec3 = IVec3::new(0, 1, 0);

const NO_PORTS: &[Port] = &[];

pub const WHEEL_SPEC: ProcessSpec = ProcessSpec {
    block: WATER_WHEEL,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Hydro, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint { size: [3, 1, 3], ports: NO_PORTS },
    verb: "Generating",
    products: "power",
    waiting: "Idle",
    map_colour: 0x3d78b8,
    compute: 0,
    parts: &WHEEL_PARTS,
};

const STONE: Look = Look::Tex([tex::CONCRETE, tex::STONE, tex::STONE]);
const TIMBER: Look = Look::Tex([tex::HANDLE; 3]);
const IRON: Look = Look::Tex([tex::MAT_GUNMETAL; 3]);
const BRASS: Look = Look::Tex([tex::MAT_BRASS; 3]);
/// Leaning angle of the four A-frame trestle legs (radians from vertical, `atan2(0.48, 1.30)`).
const LEG_PITCH: f32 = 0.3537;

/// Upright 3×1×3 (±1.5 across and high, ±0.5 deep): twin stone footings, braced timber A-frame trestles front and
/// back, cast-iron pillow-block bearings with brass caps, status lamp, and the spinning wheel (`Look::WaterWheel`).
const WHEEL_PARTS: [Part; 14] = [
    // Stone footings seated flush on the floor (y = -1.50 .. -1.36).
    part([0.0, -1.43, 0.42], [1.68, 0.14, 0.14], STONE),
    part([0.0, -1.43, -0.42], [1.68, 0.14, 0.14], STONE),
    // Front and back A-frame timber legs leaning from the footings to the axle bearings.
    leaned([-0.40, -0.71, 0.42], [0.10, 1.38, 0.11], TIMBER, -1.0, LEG_PITCH),
    leaned([0.40, -0.71, 0.42], [0.10, 1.38, 0.11], TIMBER, 1.0, LEG_PITCH),
    leaned([-0.40, -0.71, -0.42], [0.10, 1.38, 0.11], TIMBER, -1.0, LEG_PITCH),
    leaned([0.40, -0.71, -0.42], [0.10, 1.38, 0.11], TIMBER, 1.0, LEG_PITCH),
    // Horizontal cross-ties bracing each A-frame.
    part([0.0, -0.81, 0.42], [0.96, 0.10, 0.10], TIMBER),
    part([0.0, -0.81, -0.42], [0.96, 0.10, 0.10], TIMBER),
    // Cast-iron pillow-block bearings and brass caps at the axle (y = 0).
    part([0.0, 0.0, 0.42], [0.38, 0.26, 0.12], IRON),
    part([0.0, 0.0, -0.42], [0.38, 0.26, 0.12], IRON),
    part([0.0, 0.16, 0.42], [0.24, 0.06, 0.10], BRASS),
    part([0.0, 0.16, -0.42], [0.24, 0.06, 0.10], BRASS),
    // Status lamp seated on the front bearing cap.
    part([0.0, 0.23, 0.42], [0.12, 0.08, 0.08], Look::Lamp),
    // Dynamic rotating wheel, water-filled buckets and splash spray (`hydro/view.rs`).
    part([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], Look::WaterWheel),
];

/// Falling-water flow level (1..=7) at `pos`, or `0` if not falling (still water, dry, or no liquid above).
fn falling_level(world: &mut World, pos: IVec3) -> u8 {
    let b = world.block_anywhere_or_generate(pos);
    match flow_level(b) {
        Some(lvl) if LIQUID[world.block_anywhere_or_generate(pos + UP) as usize] => lvl,
        _ => 0,
    }
}

impl Processor {
    /// The status line of a water wheel (`None`: another processor).
    pub(super) fn hydro_text(&self) -> Option<String> {
        if self.energy() != Energy::Hydro {
            return None;
        }
        let left = self.store.hydro_mask & 0x07;
        let right = (self.store.hydro_mask >> 4) & 0x07;
        Some(match self.store.flow {
            0 if left > 0 && right > 0 => "Stalled: falling water on both sides cancels out".to_string(),
            0 => "Dry: needs falling water on one side to spin".to_string(),
            flow => format!("Falling water gives {flow} of {MAX_KW} kW · giving {} kW", self.store.given),
        })
    }
}

/// Checks falling water on the left and right sides of every wheel, splits shared waterfall cells so multiple
/// wheels cannot double-dip one column, scales power & spin by flow strength (`FLOW_1..=FLOW_7`), and eases the
/// wheel's rotation. Called by `Factory::update` before the power balance.
pub(in crate::factory) fn sense(processors: &mut [Processor], wheels: &[u32], world: &mut World) {
    struct Scan {
        idx: usize,
        left_mask: u8,
        right_mask: u8,
        cells: [Option<(IVec3, u8)>; 3],
    }
    let mut scans = Vec::with_capacity(wheels.len());
    let mut shared: FxHashMap<IVec3, u32> = FxHashMap::default();
    for &w in wheels {
        let idx = w as usize;
        let p = &processors[idx];
        let right = DIRS[(p.dir as usize + 1) % 4];
        let (mut left_mask, mut right_mask) = (0u8, 0u8);
        let (mut left_lvls, mut right_lvls) = ([0u8; 3], [0u8; 3]);
        for y in 0..3i32 {
            let l = falling_level(world, p.pos - right + UP * y);
            if l > 0 {
                left_mask |= 1 << y;
                left_lvls[y as usize] = l;
            }
            let r = falling_level(world, p.pos + right * 3 + UP * y);
            if r > 0 {
                right_mask |= 1 << y;
                right_lvls[y as usize] = r;
            }
        }
        let mut cells = [None; 3];
        if (left_mask > 0) ^ (right_mask > 0) {
            let (lvls, side_off) = if left_mask > 0 { (left_lvls, right * -1) } else { (right_lvls, right * 3) };
            for y in 0..3i32 {
                let lvl = lvls[y as usize];
                if lvl > 0 {
                    let cell = p.pos + side_off + UP * y;
                    cells[y as usize] = Some((cell, lvl));
                    *shared.entry(cell).or_insert(0) += 1;
                }
            }
        }
        scans.push(Scan { idx, left_mask, right_mask, cells });
    }

    for s in scans {
        let p = &mut processors[s.idx];
        p.store.hydro_mask = s.left_mask | (s.right_mask << 4);
        let spin_dir = match (s.left_mask > 0, s.right_mask > 0) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0,
        };
        let flow = if spin_dir == 0.0 {
            0
        } else {
            let sum: u32 = s
                .cells
                .into_iter()
                .flatten()
                .map(|(c, lvl)| (FALL_KW * u32::from(lvl)) / (7 * shared[&c].max(1)))
                .sum();
            sum.min(MAX_KW)
        };
        p.store.flow = flow;
        let target = spin_dir * (f64::from(flow) / f64::from(MAX_KW)) * (TICK / SPIN_SECONDS);
        p.store.wheel_spin += (target - p.store.wheel_spin) * SPIN_EASE;
        if target == 0.0 && p.store.wheel_spin.abs() < 1e-6 {
            p.store.wheel_spin = 0.0;
        }
        p.store.wheel_angle = (p.store.wheel_angle + p.store.wheel_spin).rem_euclid(1.0);
    }
}

/// One tick of every water wheel on a grid: each gives what its grid still lacks, up to what its water gives.
/// Called by `Power::balance` with the sun, before any fuel burns.
pub(in crate::factory) fn run(power: &mut Power, processors: &mut [Processor], wheels: &[u32]) {
    for &i in wheels {
        let (i, p) = (i as usize, &mut processors[i as usize]);
        p.store.given = 0;
        let Some(pole) = power.process_pole[i] else {
            p.status = Status::NoPower;
            continue;
        };
        let grid = power.pole_grid[pole as usize] as usize;
        let given = p.store.flow.min(power.demand[grid] - power.supply[grid].min(power.demand[grid]));
        power.give(Source::Hydro, grid, given, p.store.flow);
        p.store.given = given;
        p.status = if given > 0 { Status::Working } else { Status::NoInput };
    }
}

#[cfg(test)]
mod tests;
