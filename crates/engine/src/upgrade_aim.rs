//! Aiming with an upgrade kit in hand (`factory/upgrades.rs`): what the kit would do to the machine or
//! belt under the crosshair, shown before the click. Presentation only; the click itself is in
//! `belt_line.rs` (belts, miners and the rest) and the core's `Action::Upgrade`.
//!
//! [`Aim`] is recomputed every tick a kit is held and no belt line is being dragged. A machine gets an
//! outline over all its cells; a belt or underpass gets its own outline, or with Shift held every belt and
//! underpass joined to it of the same tier (`Factory::belt_chain`). The colour is the kit's tier colour, or red when the kits in the
//! inventory don't pay for it or research hasn't unlocked the step. The HUD label gives the cost.
//!
//! To change what is said or drawn: `upgrade_aim`. The host reads `placement_box` (the machine's outline),
//! `line_cells` (belts) and `line_label` (`api/hud.rs`).

use crate::block;
use crate::factory::upgrades;
use crate::item::{self, ItemId};
use crate::math::IVec3;
use crate::research::{self, Unlock};
use crate::Game;

/// The outline of what a kit can't pay for or research hasn't unlocked (also a machine in the way).
pub(crate) const BLOCKED_RED: i32 = 0xff4a3d;
/// The most belts one Shift-click upgrades.
pub const MAX_CHAIN: usize = 256;

/// What the held kit would upgrade.
#[derive(Default)]
pub struct Aim {
    /// The lowest and highest cell of the machine that would be upgraded (not belts).
    pub machine: Option<(IVec3, IVec3)>,
    /// The belts that would be upgraded, nearest the crosshair first.
    pub belts: Vec<IVec3>,
    /// The machine outline's colour, 0xRRGGBB.
    pub colour: i32,
    /// The HUD card: a title line, then the details. Empty when there is nothing to say.
    pub label: String,
}

impl Game {
    /// The aim for the kit `kit` in hand: the targeted tiered machine or belt, or nothing.
    pub(crate) fn upgrade_aim(&self, kit: ItemId) -> Aim {
        let Some(hit) = self.target else { return Aim::default() };
        let f = &self.sim.factory;
        let Some((block, tier)) = f.tiered_at(hit.block) else { return Aim::default() };
        let (name, from) = (crate::factory::tiers::name(block, tier), tier + 1);
        let say = |detail: String| Aim { label: format!("{name}\n{detail}"), ..Aim::default() };
        let Some(step) = f.next_upgrade(hit.block) else { return say("Nothing higher to upgrade to yet".into()) };
        if step.kit != kit {
            return say(format!("Needs a {} to reach Mk{}", item::name(step.kit), from + 1));
        }
        if let Some(t) = f.research.locked_by(Unlock::Upgrade(step.block, step.tier)) {
            return say(format!("Research {} first", research::TECHS[t as usize].name));
        }
        let title = format!("{name} · Mk{from} → Mk{}", from + 1);
        let have = self.inventory().count(kit);
        let cost = |kits: u32| {
            let short = if have < kits { format!(" (you have {have})") } else { String::new() };
            (kits, format!("{kits} × {}{short}", item::name(kit)))
        };
        if block != block::BELT && block != block::UNDERPASS_IN {
            let cells = f.footprint_at(hit.block).map_or_else(|| vec![hit.block], |f| f.2);
            let (kits, cost) = cost(step.kits);
            let colour = if have >= kits { upgrades::TIER_COLOURS[step.tier as usize] as i32 } else { BLOCKED_RED };
            return Aim {
                machine: Some(bounds(&cells)),
                colour,
                label: format!("{title}\n{cost} · right-click to upgrade"),
                ..Aim::default()
            };
        }
        let whole_line = self.body().input.sprint;
        let belts = if whole_line { f.belt_chain(hit.block, MAX_CHAIN) } else { vec![hit.block] };
        let passes = belts.iter().filter(|&&p| f.tiered_at(p).is_some_and(|t| t.0 == block::UNDERPASS_IN)).count();
        let how =
            if whole_line { "click to upgrade them all" } else { "click to upgrade · hold Shift for the whole line" };
        let title = if whole_line { format!("{title} (whole line)") } else { title };
        let (_, cost) = cost(f.upgrade_kits(belts.iter().copied()));
        Aim {
            label: format!("{title}\n{} · {cost} · {how}", pieces(belts.len() - passes, passes)),
            belts,
            ..Aim::default()
        }
    }

    /// The outline of the machine the held kit would upgrade, as `placement_box` numbers; empty if none.
    pub(crate) fn aim_box(&self) -> Vec<i32> {
        let a = &self.line.aim;
        a.machine.map_or_else(Vec::new, |(lo, hi)| vec![lo.x, lo.y, lo.z, hi.x, hi.y, hi.z, a.colour])
    }
}

/// "3 belts", "1 underpass" or "3 belts + 2 underpasses".
fn pieces(belts: usize, passes: usize) -> String {
    let say = |n: usize, one: &str| {
        format!(
            "{n} {one}{}",
            if n == 1 {
                ""
            } else if one == "belt" {
                "s"
            } else {
                "es"
            }
        )
    };
    match (belts, passes) {
        (_, 0) => say(belts, "belt"),
        (0, _) => say(passes, "underpass"),
        _ => format!("{} + {}", say(belts, "belt"), say(passes, "underpass")),
    }
}

/// The lowest and highest corner of `cells`.
fn bounds(cells: &[IVec3]) -> (IVec3, IVec3) {
    let first = cells[0];
    cells.iter().fold((first, first), |(lo, hi), c| {
        (
            IVec3::new(lo.x.min(c.x), lo.y.min(c.y), lo.z.min(c.z)),
            IVec3::new(hi.x.max(c.x), hi.y.max(c.y), hi.z.max(c.z)),
        )
    })
}

#[cfg(test)]
mod tests;
