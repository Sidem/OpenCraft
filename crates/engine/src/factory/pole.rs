//! Power poles: a position and a tier (`POLE_TIERS`). A pole has a few `slots`; the player fills them by
//! hand (`wiring.rs`) with poles within the longer `link` of the two and machines within its `reach` of
//! any of their cells. Nothing hangs on a pole by itself. Higher tiers are pylons: more slots, longer
//! spans and a wider reach.
//!
//! A cable (block `CABLE`, stored as a pole of tier [`CABLE_TIER`]) is a short node for shafts and
//! tunnels that needs no wiring: cables touching each other (even diagonally) are one wire, a cable
//! within a pole's reach joins that pole's grid, and a machine within [`CABLE_STATS`]`.reach` of a cable
//! hangs on it, unless it is wired to a pole by hand (poles win). Hang a cable from a pole at the rim
//! and lower it.

use crate::block::tex;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::math::{IVec3, Vec3};

use super::render::push_box;
use super::{Factory, Machine};

/// What a pole tier reaches, in blocks between cell centres.
pub struct PoleTier {
    /// Poles this close can be wired together.
    pub link: i32,
    /// Machines this close can be wired to it.
    pub reach: i32,
    /// How many things (poles and machines) it can be wired to.
    pub slots: u8,
}

/// Mk1 to Mk4 (the substation: as far as the pylon, but reaching 16).
pub const POLE_TIERS: [PoleTier; 4] = [
    PoleTier { link: 10, reach: 5, slots: 4 },
    PoleTier { link: 16, reach: 7, slots: 8 },
    PoleTier { link: 32, reach: 9, slots: 12 },
    PoleTier { link: 32, reach: 16, slots: 16 },
];

/// The tier a cable is stored as: not an upgrade of poles (`tiers.rs` never reaches it). It was 3 in the few saves of
/// version 21 that hold cables, before the Mk4 pole took that number.
pub const CABLE_TIER: u8 = 15;
/// A cable's numbers: `link` and `slots` are unused (cables join by touching and by a pole's reach), `reach`
/// is how far machines hang on it.
pub const CABLE_STATS: PoleTier = PoleTier { link: 1, reach: 2, slots: 0 };
/// Cables this close (squared: touching, diagonals too) are one wire.
const CABLE_TOUCH2: i32 = 2;

pub struct Pole {
    pub pos: IVec3,
    pub tier: u8,
}

impl Pole {
    pub fn stats(&self) -> &'static PoleTier {
        if self.is_cable() {
            &CABLE_STATS
        } else {
            &POLE_TIERS[self.tier as usize]
        }
    }

    pub fn is_cable(&self) -> bool {
        self.tier == CABLE_TIER
    }
}

/// Whether poles `a` and `b` are within range of each other: cables touching, a cable in a pole's reach, or
/// two real poles within their link. Cables join by range alone; real poles only by hand (`wiring.rs`),
/// and saves before version 24 are converted with this rule (`Factory::hook_by_reach`).
pub(super) fn linked(a: &Pole, b: &Pole) -> bool {
    let d = dist2(a.pos, b.pos);
    match (a.is_cable(), b.is_cable()) {
        (true, true) => d <= CABLE_TOUCH2,
        (true, false) => d <= b.stats().reach * b.stats().reach,
        (false, true) => d <= a.stats().reach * a.stats().reach,
        (false, false) => d <= a.stats().link.max(b.stats().link).pow(2),
    }
}

/// Squared distance between two cell centres.
pub(super) fn dist2(a: IVec3, b: IVec3) -> i32 {
    let d = a - b;
    d.x * d.x + d.y * d.y + d.z * d.z
}

/// The nearest cable (with `cable` off: real pole) with `pos` in its reach; the lower index on ties.
pub(super) fn nearest_of(poles: &[Pole], pos: IVec3, cable: bool) -> Option<u32> {
    let mut best: Option<(i32, u32)> = None;
    for (i, p) in poles.iter().enumerate().filter(|(_, p)| p.is_cable() == cable) {
        let d = dist2(p.pos, pos);
        if d <= p.stats().reach * p.stats().reach && best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, i as u32));
        }
    }
    best.map(|b| b.1)
}

/// The cable nearest any of `cells` within its reach (a machine several cells big hangs on it).
pub(super) fn hang_cable(poles: &[Pole], cells: &[IVec3]) -> Option<u32> {
    let near = |&c: &IVec3| nearest_of(poles, c, true).map(|i| (dist2(poles[i as usize].pos, c), i));
    cells.iter().filter_map(near).min().map(|best| best.1)
}

impl Machine for Pole {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.tier);
    }

    fn read_state(r: &mut ByteReader) -> Option<Pole> {
        let pos = r.ivec3()?;
        let tier = if r.version >= 21 { r.u8()? } else { 0 };
        let tier = if r.version == 21 && tier == 3 { CABLE_TIER } else { tier };
        (((tier as usize) < POLE_TIERS.len()) || tier == CABLE_TIER).then_some(Pole { pos, tier })
    }

    fn contents(&self) -> Vec<Stack> {
        Vec::new()
    }

    fn describe(&self, f: &Factory) -> String {
        if f.dirty {
            return "Power pole".to_string();
        }
        let me = f.poles.iter().position(|p| p.pos == self.pos).map(|i| i as u32);
        if self.is_cable() {
            return format!("{}\nPower cable: joins the grid of a pole in reach", f.power.grid_line(me));
        }
        let grid = me.map(|i| f.power.pole_grid[i as usize]);
        let on = |p: &Option<u32>| p.is_some_and(|p| Some(f.power.pole_grid[p as usize]) == grid);
        let poles = f.power.pole_grid.iter().filter(|&&g| Some(g) == grid).count();
        let gens = f.power.gen_pole.iter().filter(|p| on(p)).count();
        let machines = f.power.miner_pole.iter().chain(&f.power.process_pole).chain(&f.power.lab_pole);
        let machines = machines.chain(&f.power.pipe_pole).chain(&f.power.quarry_pole).filter(|p| on(p)).count();
        let PoleTier { link, reach, slots } = *self.stats();
        format!(
            "{}\n{poles} poles, {gens} generators, {machines} machines on this grid\n{} of {slots} connections used: \
             poles within {link} blocks, machines within {reach}",
            f.power.grid_line(me),
            f.slots_used(self.pos)
        )
    }

    /// A steel post with a crossarm and two copper insulators; a tier wears its stripe on the post. A cable
    /// is drawn by `cable.rs`, which knows what it touches.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, _: f64) {
        if self.is_cable() {
            return;
        }
        push_box(out, rel + Vec3::new(0.0, -0.42, 0.0), 0.0, [0.38, 0.15, 0.38], 0.0, [tex::FRAME; 3], false);
        push_box(out, rel + Vec3::new(0.0, -0.04, 0.0), 0.0, [0.13, 0.85, 0.13], 0.0, [tex::FRAME; 3], true);
        if self.tier > 0 {
            let band = [tex::stripe(self.tier); 3];
            push_box(out, rel + Vec3::new(0.0, -0.28, 0.0), 0.0, [0.17, 0.14, 0.17], 0.0, band, false);
        }
        push_box(out, rel + Vec3::new(0.0, 0.34, 0.0), 0.0, [0.72, 0.08, 0.11], 0.0, [tex::FRAME; 3], false);
        for x in [-0.28, 0.28] {
            let c = [tex::COPPER_WIRE; 3];
            push_box(out, rel + Vec3::new(x, 0.45, 0.0), 0.0, [0.12, 0.17, 0.12], 0.0, [tex::FLASK_GLASS; 3], false);
            push_box(out, rel + Vec3::new(x, 0.55, 0.0), 0.0, [0.08, 0.05, 0.08], 0.0, c, false);
        }
    }
}

impl Factory {
    /// Every real pole (cables left out) as its cell and tier: what the pole tool chains from
    /// (`power_tools.rs`).
    pub fn poles(&self) -> impl Iterator<Item = (IVec3, u8)> + '_ {
        self.poles.iter().filter(|p| !p.is_cable()).map(|p| (p.pos, p.tier))
    }

    /// Whether a cable put at `pos` would join a grid: a pole reaches it or a cable touches it.
    pub fn cable_joins(&self, pos: IVec3) -> bool {
        let probe = Pole { pos, tier: CABLE_TIER };
        self.poles.iter().any(|p| p.pos != pos && linked(&probe, p))
    }
}

/// A pole's model where none stands yet (the pole tool's preview, `power_tools.rs`); `rel` is its cell
/// centre relative to the camera.
pub fn preview_pole(out: &mut Vec<f32>, tier: u8, rel: Vec3) {
    Pole { pos: IVec3::ZERO, tier }.model(out, rel, 0.0);
}

/// A wire between two camera-relative points, drawn like the placed ones (the pole tool's preview).
pub fn preview_wire(out: &mut Vec<f32>, a: Vec3, b: Vec3) {
    super::power::wire(out, a, b);
}

#[cfg(test)]
mod tests;
