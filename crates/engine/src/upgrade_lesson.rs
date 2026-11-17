//! L5, the upgrade lesson (a need lesson, beside `need_lesson.rs`): the player points at a machine that is a bottleneck
//! (`Processor::is_bottleneck`: working at full pace with its inputs piled up) while holding the kits for its next Mk and
//! the research for the step. The card says a kit would make it faster and how to apply it, and an Open prompt hangs over
//! the machine (the panel has the upgrade button, `web/src/ui/machine-upgrade.ts`).
//!
//! Presentation: the watch lives in `NeedWatch`, scans once a second until the world was taught (`lessons::UPGRADE_BIT`,
//! never cleared) and is never saved. The card shows after `POINT_TICKS` of the watch, so a sweep of the crosshair brings
//! nothing. Upgrading the watched machine teaches it, and so does H; a machine that is gone, out of reach, or a player
//! who no longer holds the kits drops it without teaching. Only a stall, power or recipe card goes ahead of it.

use crate::action::Action;
use crate::block::{self, BlockId};
use crate::item::{self, ItemId};
use crate::lessons::UPGRADE_BIT;
use crate::math::IVec3;
use crate::research::Unlock;
use crate::stall_lesson::{QUIET_TICKS, RANGE};
use crate::{Game, TICK_RATE};

/// How long the machine stays a candidate before its card shows.
const POINT_TICKS: u64 = 2 * TICK_RATE as u64;

/// The card. `{m}` is the machine's name, `{n}` the kits, `{k}` the kit's name.
const UPGRADE: &str = "Upgrade: the {m} runs flat out with inputs piling up. [Right-click] it and press the upgrade \
                       arrow ({n} × {k}) to make it faster.";

/// The machine watched: its anchor, its family block and tier when first seen, what the next step costs, and when.
#[derive(Clone, Copy)]
pub(crate) struct Watch {
    pos: IVec3,
    block: BlockId,
    tier: u8,
    kit: ItemId,
    kits: u32,
    since: u64,
}

impl Game {
    /// The upgrade lesson on show now: the machine's cell. Never over another card.
    pub(crate) fn upgrade_lesson(&self) -> Option<IVec3> {
        let w = self.need.upgrade?;
        let shown = self.sim.tick >= w.since + POINT_TICKS;
        let free = self.stall_lesson().is_none() && self.low_power_lesson().is_none() && self.recipe_lesson().is_none();
        (shown && free).then_some(w.pos)
    }

    /// The card's text for the lesson on show (empty for none).
    pub(crate) fn upgrade_text(&self) -> String {
        let Some(w) = self.need.upgrade.filter(|_| self.upgrade_lesson().is_some()) else { return String::new() };
        let name = self.sim.factory.name_at(w.pos).unwrap_or_else(|| block::def(w.block).name.into());
        UPGRADE.replace("{m}", &name).replace("{n}", &w.kits.to_string()).replace("{k}", item::name(w.kit))
    }

    /// H on the card: taught.
    pub(crate) fn upgrade_dismiss(&mut self) {
        if self.upgrade_lesson().is_some() {
            self.need.upgrade = None;
            self.teach_upgrade(true);
        }
    }

    /// Called once a second by `watch_needs`, only until the world was taught.
    pub(crate) fn scan_upgrades(&mut self) {
        let tick = self.sim.tick;
        let me = self.body().pos;
        let near = |p: IVec3| (p.x as f64 - me.x).hypot(p.z as f64 - me.z);
        if let Some(w) = self.need.upgrade {
            let tier = self.sim.factory.tiered_at(w.pos).map(|t| t.1);
            let kept = tier == Some(w.tier) && near(w.pos) <= RANGE + 8.0 && self.inventory().count(w.kit) >= w.kits;
            if kept {
                return;
            }
            self.need.upgrade = None;
            if tier.is_some_and(|t| t > w.tier) {
                self.teach_upgrade(tick >= w.since + POINT_TICKS); // upgraded: the lesson learned
            }
        }
        if tick < self.stall.quiet_until || self.stall_lesson().is_some() {
            return;
        }
        let f = &self.sim.factory;
        let Some((pos, step)) = self.target.and_then(|t| f.bottleneck_at(t.block)) else { return };
        let ready =
            f.research.has(Unlock::Upgrade(step.block, step.tier)) && self.inventory().count(step.kit) >= step.kits;
        if ready && near(pos) <= RANGE {
            let tier = step.tier - 1;
            self.need.upgrade =
                Some(Watch { pos, block: step.block, tier, kit: step.kit, kits: step.kits, since: tick });
        }
    }

    /// Marks the lesson taught; the quiet spell follows only a card the player saw.
    fn teach_upgrade(&mut self, shown: bool) {
        self.act(Action::LessonTaught { bit: UPGRADE_BIT });
        if shown {
            self.stall.quiet_until = self.sim.tick + QUIET_TICKS;
        }
    }
}
