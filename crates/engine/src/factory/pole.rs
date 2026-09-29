//! Power poles: a position and a tier (`POLE_TIERS`); its links are derived by `power.rs`. A pole links to
//! every pole within the longer `link` of the two, and a machine hangs on the nearest pole within that
//! pole's `reach` of any of its cells (ties: the lower pole index). Higher tiers are pylons: longer
//! spans and a wider reach.

use crate::block::tex;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::math::{IVec3, Vec3};

use super::render::push_box;
use super::{Factory, Machine};

/// What a pole tier reaches, in blocks between cell centres.
pub struct PoleTier {
    /// Poles this close are wired together.
    pub link: i32,
    /// Machines this close join its grid.
    pub reach: i32,
}

/// Mk1, Mk2 and Mk3.
pub const POLE_TIERS: [PoleTier; 3] =
    [PoleTier { link: 10, reach: 5 }, PoleTier { link: 16, reach: 7 }, PoleTier { link: 32, reach: 9 }];

pub struct Pole {
    pub pos: IVec3,
    pub tier: u8,
}

impl Pole {
    pub fn stats(&self) -> &'static PoleTier {
        &POLE_TIERS[self.tier as usize]
    }
}

/// Whether poles `a` and `b` are wired together.
pub(super) fn linked(a: &Pole, b: &Pole) -> bool {
    let link = a.stats().link.max(b.stats().link);
    dist2(a.pos, b.pos) <= link * link
}

/// Squared distance between two cell centres.
pub(super) fn dist2(a: IVec3, b: IVec3) -> i32 {
    let d = a - b;
    d.x * d.x + d.y * d.y + d.z * d.z
}

/// The nearest pole with `pos` in its reach (the lower index on ties).
pub(super) fn nearest_pole(poles: &[Pole], pos: IVec3) -> Option<u32> {
    let mut best: Option<(i32, u32)> = None;
    for (i, p) in poles.iter().enumerate() {
        let d = dist2(p.pos, pos);
        if d <= p.stats().reach * p.stats().reach && best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, i as u32));
        }
    }
    best.map(|b| b.1)
}

/// The pole nearest any of `cells` within its reach (a machine several cells big hangs on it).
pub(super) fn hang_any(poles: &[Pole], cells: &[IVec3]) -> Option<u32> {
    let near = |&c: &IVec3| nearest_pole(poles, c).map(|i| (dist2(poles[i as usize].pos, c), i));
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
        ((tier as usize) < POLE_TIERS.len()).then_some(Pole { pos, tier })
    }

    fn contents(&self) -> Vec<Stack> {
        Vec::new()
    }

    fn describe(&self, f: &Factory) -> String {
        if f.dirty {
            return "Power pole".to_string();
        }
        let me = f.poles.iter().position(|p| p.pos == self.pos).map(|i| i as u32);
        let grid = me.map(|i| f.power.pole_grid[i as usize]);
        let on = |p: &Option<u32>| p.is_some_and(|p| Some(f.power.pole_grid[p as usize]) == grid);
        let poles = f.power.pole_grid.iter().filter(|&&g| Some(g) == grid).count();
        let gens = f.power.gen_pole.iter().filter(|p| on(p)).count();
        let machines = f.power.miner_pole.iter().chain(&f.power.process_pole).chain(&f.power.router_pole);
        let machines = machines.chain(&f.power.lab_pole).chain(&f.power.pipe_pole).chain(&f.power.quarry_pole);
        let machines = machines.filter(|p| on(p)).count();
        let PoleTier { link, reach } = *self.stats();
        format!(
            "{}\n{poles} poles, {gens} generators, {machines} machines on this grid\nLinks to poles within \
             {link} blocks and powers machines within {reach}",
            f.power.grid_line(me)
        )
    }

    /// A steel post with a crossarm and two copper insulators; a tier wears its stripe on the post.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, _: f64) {
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
