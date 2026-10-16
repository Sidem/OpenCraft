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

use super::footprint::Footprint;
use super::links::Slot;
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

/// The tier a long-distance pole (block `LONG_POLE`, 2×2×3) is stored as: its own block, so no upgrade reaches it.
pub const LONG_TIER: u8 = 14;
/// A long-distance pole's numbers: `link` is the span to other long-distance poles only.
pub const LONG_STATS: PoleTier = PoleTier { link: 160, reach: 10, slots: 6 };
/// How far a long-distance pole links to an ordinary pole (it is no further than a Mk2 pole there).
pub const LONG_LOCAL_LINK: i32 = 16;
/// A long-distance pole stands on these cells, from its anchor, whichever way it was placed (it stores no facing).
pub const LONG_FOOTPRINT: Footprint = Footprint { size: [2, 2, 3], ports: &[] };

/// The numbers of pole tier `tier` (a cable and a long-distance pole have their own).
pub fn tier_stats(tier: u8) -> &'static PoleTier {
    match tier {
        CABLE_TIER => &CABLE_STATS,
        LONG_TIER => &LONG_STATS,
        t => &POLE_TIERS[t as usize],
    }
}

/// How far apart poles of tiers `a` and `b` can be wired: two long-distance poles span `LONG_STATS.link`, but a
/// long-distance pole reaches an ordinary pole only as far as `LONG_LOCAL_LINK` (the longer link of the two wins).
pub fn span(a: u8, b: u8) -> i32 {
    let local = |t: u8| if t == LONG_TIER { LONG_LOCAL_LINK } else { tier_stats(t).link };
    if a == LONG_TIER && b == LONG_TIER {
        LONG_STATS.link
    } else {
        local(a).max(local(b))
    }
}

pub struct Pole {
    pub pos: IVec3,
    pub tier: u8,
}

impl Pole {
    pub fn stats(&self) -> &'static PoleTier {
        tier_stats(self.tier)
    }

    pub fn is_cable(&self) -> bool {
        self.tier == CABLE_TIER
    }

    pub fn is_long(&self) -> bool {
        self.tier == LONG_TIER
    }

    /// Where its wires hang, in the world: a crossarm, a cable's middle, or the top of a long-distance pole.
    pub fn wire_point(&self) -> Vec3 {
        let base = self.pos.as_vec3();
        match self.tier {
            CABLE_TIER => base + Vec3::new(0.5, 0.5, 0.5),
            LONG_TIER => {
                let c = LONG_FOOTPRINT.centre(0);
                base + Vec3::new(0.5 + c.x, 2.95, 0.5 + c.z)
            }
            _ => base + Vec3::new(0.5, 0.92, 0.5),
        }
    }
}

/// Whether poles `a` and `b` are within range of each other: cables touching, a cable in a pole's reach, or
/// two real poles within their `span`. Cables join by range alone; real poles only by hand (`wiring.rs`),
/// and saves before version 24 are converted with this rule (`Factory::hook_by_reach`).
pub(super) fn linked(a: &Pole, b: &Pole) -> bool {
    let d = dist2(a.pos, b.pos);
    match (a.is_cable(), b.is_cable()) {
        (true, true) => d <= CABLE_TOUCH2,
        (true, false) => d <= b.stats().reach * b.stats().reach,
        (false, true) => d <= a.stats().reach * a.stats().reach,
        (false, false) => d <= span(a.tier, b.tier).pow(2),
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

    fn cells(&self) -> Vec<IVec3> {
        if self.is_long() {
            LONG_FOOTPRINT.cells(self.pos, 0)
        } else {
            vec![self.pos]
        }
    }

    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.tier);
    }

    fn read_state(r: &mut ByteReader) -> Option<Pole> {
        let pos = r.ivec3()?;
        let tier = if r.version >= 21 { r.u8()? } else { 0 };
        let tier = if r.version == 21 && tier == 3 { CABLE_TIER } else { tier };
        (((tier as usize) < POLE_TIERS.len()) || tier == CABLE_TIER || tier == LONG_TIER).then_some(Pole { pos, tier })
    }

    fn contents(&self) -> Vec<Stack> {
        Vec::new()
    }

    fn describe(&self, f: &Factory) -> String {
        if f.dirty {
            return "Power pole".to_string();
        }
        let me = f.poles.iter().position(|p| p.pos == self.pos).map(|i| i as u32);
        let label = me.map_or_else(String::new, |i| format!("{}\n", f.power.grid_label(i)));
        if self.is_cable() {
            return format!("{label}{}\nPower cable: joins the grid of a pole in reach", f.power.grid_line(me));
        }
        let grid = me.map(|i| f.power.pole_grid[i as usize]);
        let on = |p: &Option<u32>| p.is_some_and(|p| Some(f.power.pole_grid[p as usize]) == grid);
        let poles = f.power.pole_grid.iter().filter(|&&g| Some(g) == grid).count();
        let gens = f.power.gen_pole.iter().filter(|p| on(p)).count();
        let machines = f.power.miner_pole.iter().chain(&f.power.process_pole).chain(&f.power.lab_pole);
        let machines = machines.chain(&f.power.pipe_pole).chain(&f.power.quarry_pole).filter(|p| on(p)).count();
        let PoleTier { link, reach, slots } = *self.stats();
        if self.is_long() {
            return format!(
                "{label}{}\n{poles} poles, {gens} generators, {machines} machines on this grid\n{} of {slots} \
                 connections used: long-distance poles within {link} blocks, other poles within {LONG_LOCAL_LINK}, \
                 machines within {reach}",
                f.power.grid_line(me),
                f.slots_used(self.pos)
            );
        }
        format!(
            "{label}{}\n{poles} poles, {gens} generators, {machines} machines on this grid\n{} of {slots} connections \
             used: poles within {link} blocks, machines within {reach}",
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
        if self.is_long() {
            return long_model(out, rel + LONG_FOOTPRINT.centre(0));
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
    /// Every ordinary pole (cables and long-distance poles left out) as its cell and tier: what the pole tool
    /// chains from (`power_tools.rs`).
    pub fn poles(&self) -> impl Iterator<Item = (IVec3, u8)> + '_ {
        self.poles.iter().filter(|p| !p.is_cable() && !p.is_long()).map(|p| (p.pos, p.tier))
    }

    /// Every pole a hover pack can charge from (everything but cables) as its anchor, the middle of its body and
    /// whether it is a long-distance pole, which charges from further away (`helpers/`).
    pub fn charge_poles(&self) -> impl Iterator<Item = (IVec3, Vec3, bool)> + '_ {
        self.poles.iter().filter(|p| !p.is_cable()).map(|p| {
            let mid = p.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
            (p.pos, if p.is_long() { mid + LONG_FOOTPRINT.centre(0) } else { mid }, p.is_long())
        })
    }

    /// Where the wires of the pole anchored at `pos` hang, in the world (`None` when no pole is anchored there).
    pub fn pole_top(&self, pos: IVec3) -> Option<Vec3> {
        match self.at.get(&pos) {
            Some(&Slot::Pole(i)) => Some(self.poles[i as usize].wire_point()),
            _ => None,
        }
    }

    /// Whether a cable put at `pos` would join a grid: a pole reaches it or a cable touches it.
    pub fn cable_joins(&self, pos: IVec3) -> bool {
        let probe = Pole { pos, tier: CABLE_TIER };
        self.poles.iter().any(|p| p.pos != pos && linked(&probe, p))
    }

    /// What a long-distance pole put at `anchor` would be wired to (the placing preview, `power_tools/long_pole.rs`):
    /// `auto_hook`'s pole, else the nearest long-distance pole and why not. `None` while the grids are stale.
    pub fn long_link(&self, anchor: IVec3) -> Option<LongLink> {
        if self.dirty {
            return None;
        }
        if let Some(to) = self.auto_hook(anchor, LONG_TIER) {
            let Some(&Slot::Pole(i)) = self.at.get(&to) else { return None };
            let span = span(LONG_TIER, self.poles[i as usize].tier);
            return Some(LongLink::Links { to, span, layer: self.power.pole_layer(i) });
        }
        let nearest = self.poles.iter().filter(|p| p.is_long() && p.pos != anchor).min_by_key(|p| dist2(p.pos, anchor));
        Some(match nearest {
            None => LongLink::Alone,
            Some(p) if dist2(p.pos, anchor) > LONG_STATS.link.pow(2) => LongLink::TooFar(p.pos),
            Some(p) if self.slots_used(p.pos) >= LONG_STATS.slots as usize => LongLink::Full(p.pos),
            Some(p) => LongLink::Unpowered(p.pos),
        })
    }
}

/// What a new long-distance pole would be wired to (`Factory::long_link`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LongLink {
    /// To the pole at `to`, at most `span` blocks off, on the grid whose colour is texture layer `layer`.
    Links { to: IVec3, span: i32, layer: u16 },
    /// Nothing: the nearest long-distance pole is out of reach,
    TooFar(IVec3),
    /// in reach with every connection used,
    Full(IVec3),
    /// or in reach with no power source on its grid;
    Unpowered(IVec3),
    /// or no long-distance pole stands, and no powered pole is near.
    Alone,
}

/// Where the wires of a long-distance pole anchored at `anchor` would hang, in the world (the placing preview).
pub fn long_top(anchor: IVec3) -> Vec3 {
    Pole { pos: anchor, tier: LONG_TIER }.wire_point()
}

/// The long-distance pole's lattice tower, 2×2×3; `c` is its centre relative to the camera.
fn long_model(out: &mut Vec<f32>, c: Vec3) {
    let frame = [tex::FRAME; 3];
    let mut put = |at: [f64; 3], size: [f32; 3], tex: [u16; 3]| {
        push_box(out, c + Vec3::new(at[0], at[1], at[2]), 0.0, size, 0.0, tex, false);
    };
    put([0.0, -1.4, 0.0], [1.9, 0.2, 1.9], frame);
    for (x, z) in [(-0.75, -0.75), (0.75, -0.75), (-0.75, 0.75), (0.75, 0.75)] {
        put([x, -0.05, z], [0.2, 2.8, 0.2], frame);
    }
    for y in [-0.6, 0.5] {
        for s in [-0.75, 0.75] {
            put([0.0, y, s], [1.7, 0.1, 0.1], frame);
            put([s, y, 0.0], [0.1, 0.1, 1.7], frame);
        }
    }
    put([0.0, 0.0, 0.0], [0.3, 3.0, 0.3], frame);
    put([0.0, 1.15, 0.0], [2.0, 0.12, 0.26], frame);
    for x in [-0.9, 0.9] {
        put([x, 1.3, 0.0], [0.14, 0.2, 0.14], [tex::FLASK_GLASS; 3]);
        put([x, 1.42, 0.0], [0.09, 0.05, 0.09], [tex::COPPER_WIRE; 3]);
    }
}

/// A long-distance pole's grid-coloured sleeves on its four legs and crossarm; `c` is its centre relative to the camera.
pub(super) fn long_accent(out: &mut Vec<f32>, c: Vec3, layer: u16) {
    let tex = [layer; 3];
    for (x, z) in [(-0.75, -0.75), (0.75, -0.75), (-0.75, 0.75), (0.75, 0.75)] {
        push_box(out, c + Vec3::new(x, 0.9, z), 0.0, [0.26, 0.2, 0.26], 0.0, tex, false);
    }
    push_box(out, c + Vec3::new(0.0, 1.15, 0.0), 0.0, [0.5, 0.18, 0.32], 0.0, tex, false);
}

/// A pole's model where none stands yet (the pole tool's preview, `power_tools.rs`); `rel` is its cell
/// centre relative to the camera.
pub fn preview_pole(out: &mut Vec<f32>, tier: u8, rel: Vec3) {
    Pole { pos: IVec3::ZERO, tier }.model(out, rel, 0.0);
}

/// A wire between two camera-relative points, drawn like the placed ones with texture `layer` (the pole tool's
/// preview: a grid's colour, `tex::GRID_FIRST` for the main grid's).
pub fn preview_wire(out: &mut Vec<f32>, a: Vec3, b: Vec3, layer: u16) {
    super::power::wire(out, a, b, layer);
}

#[cfg(test)]
mod tests;
