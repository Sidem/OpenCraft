//! The left hand as an instrument hand (GEAR-LEFT, `docs/ROADMAP_REQUESTS.md`): the right hand acts, the left hand
//! informs. `G` (`set_left_using`) works the item in the left hand (`Inventory::offhand`), which reads the right hand's:
//! a scanner pings by itself and looks for the ore the right hand holds (`prospect.rs`); a core drill samples the ground
//! while `G` is held and, with a miner in the right hand, shows the aimed deposit's reserve and how long that miner
//! would take to work it out (`drill_forecast`); a telescope goes up and down on `G` (`telescope.rs`) and logs the ruins
//! it is turned on to the map (`minimap/marks.rs`); the line hook fires on a press of `G` (`line_hook.rs`).
//!
//! Invariants: presentation only. Nothing here queues a core `Action` or changes the core; readings are the local
//! player's (co-op peers see only the item in the hand). In the right hand these devices do nothing but show which hand
//! they belong in (`left_hand_label`).
//!
//! To give another item a left-hand job: a `LEFT_*` number in `left_use_of`, its update in `update_left_hand`, and the
//! host's key chip reads `left_use` (`web/src/ui/offhand.ts`).

use crate::block::BlockId;
use crate::daytime::{DAY_TICKS, YEAR_DAYS};
use crate::deposits::{owner_of, DepositState};
use crate::factory::{self, hud_bar, hud_keys, hud_stats, Kind, MINER_TIERS};
use crate::item::{self, ItemId, CORE_DRILL};
use crate::math::IVec3;
use crate::prospect::FILTER_ORES;
use crate::tools::{self, ToolKind};
use crate::{Game, TICK_RATE};

/// What the left hand's item does on `G` (`left_use_of`): nothing, a scanner, a core drill, a telescope or the line hook.
pub const LEFT_NONE: u8 = 0;
pub const LEFT_SCAN: u8 = 1;
pub const LEFT_DRILL: u8 = 2;
pub const LEFT_SCOPE: u8 = 3;
pub const LEFT_HOOK: u8 = 4;

/// The left hand's state: whether `G` is held, and the core drill's forecast for the aimed block and the miner in the
/// right hand (worked out once per aim: a survey reads many blocks).
#[derive(Default)]
pub struct LeftHand {
    pub using: bool,
    /// `using` a tick ago, for the line hook's press.
    pub(crate) was_using: bool,
    pub(crate) hook: crate::line_hook::Hook,
    forecast: Option<(IVec3, ItemId, Option<String>)>,
}

/// What `item` does in the left hand.
pub fn left_use_of(item: ItemId) -> u8 {
    if item == item::LINE_HOOK {
        return LEFT_HOOK;
    }
    match tools::tool(item).map(|t| t.kind) {
        Some(ToolKind::Scanner) => LEFT_SCAN,
        Some(ToolKind::CoreDrill) => LEFT_DRILL,
        Some(ToolKind::Telescope) => LEFT_SCOPE,
        _ => LEFT_NONE,
    }
}

impl Game {
    /// The item in the local player's left hand.
    pub(crate) fn left_item(&self) -> ItemId {
        self.inventory().offhand.item
    }

    /// The ore the right hand holds, for a scanner in the left hand to look for.
    pub(crate) fn right_hand_ore(&self) -> Option<BlockId> {
        let item = self.inventory().selected_stack().item;
        FILTER_ORES.iter().copied().find(|&ore| ItemId::block(ore) == item)
    }

    /// Runs the left hand for one tick: the telescope, the prospecting devices and the drill's forecast.
    pub(crate) fn update_left_hand(&mut self, dt: f32) {
        self.update_telescope();
        self.update_prospecting(dt);
        self.update_drill_forecast();
        self.update_line_hook();
    }

    /// The HUD label the left hand asks for: the raised telescope's, the drill's forecast, or, with a left-hand device
    /// in the right hand, the keys that put it where it works.
    pub(crate) fn left_hand_label(&self) -> Option<String> {
        if self.scope_active() {
            return self.telescope_label();
        }
        if let Some((_, _, Some(label))) = &self.left.forecast {
            return Some(label.clone());
        }
        let right = self.inventory().selected_stack().item;
        (left_use_of(right) != LEFT_NONE)
            .then(|| format!("{}\n{}", item::name(right), hud_keys(&[("J", "To left hand"), ("G", "Use")])))
    }

    /// With a core drill in the left hand and a miner in the right, the forecast for the deposit under the aim.
    fn update_drill_forecast(&mut self) {
        let right = self.inventory().selected_stack().item;
        let miner = factory::tiers::placed_by(right)
            .or_else(|| right.as_block().map(|b| (b, 0)))
            .filter(|&(b, _)| factory::machine(b).is_some_and(|m| m.kind == Kind::Miner))
            .map(|(_, tier)| tier);
        let (Some(tier), Some(hit), CORE_DRILL) = (miner, self.target, self.left_item()) else {
            self.left.forecast = None;
            return;
        };
        if self.left.forecast.as_ref().is_some_and(|f| f.0 == hit.block && f.1 == right) {
            return;
        }
        let label = self.drill_forecast(hit.block, tier);
        self.left.forecast = Some((hit.block, right, label));
    }

    /// The deposit at `pos`: its reserve, and how long it lasts with a miner of `tier` added to the ones on it.
    fn drill_forecast(&mut self, pos: IVec3, tier: u8) -> Option<String> {
        let world = &mut self.sim.world;
        let d = owner_of(world, pos)?;
        let surveyed;
        let st = match self.sim.factory.deposits.get(&d.key) {
            Some(st) => st,
            None => {
                surveyed = DepositState::survey(world, d);
                &surveyed
            }
        };
        let (units, left, total) = (st.remaining_units(), st.remaining_blocks, st.initial_blocks.max(1));
        let now = self.sim.factory.deposit_forecast(&d.key);
        let drawn = now.and_then(|f| f.secs_left).map_or(0.0, |s| units / s.max(1) as f64);
        let rate = MINER_TIERS[usize::from(tier).min(MINER_TIERS.len() - 1)].rate;
        let draw = (drawn + rate).min(d.tier().draw_cap());
        let secs = (units / draw.max(1e-6)) as u64;
        let year = YEAR_DAYS * DAY_TICKS / u64::from(TICK_RATE);
        let tenths = secs * 10 / year.max(1);
        let right = format!("{} units", factory::fmt_int(units as u64));
        let bar = hud_bar(&d.name(), u64::from(left), u64::from(total), &right, "ok");
        let miners = now.map_or(0, |f| f.miners);
        let chips = [
            format!("Mk{} +{} /s", tier + 1, rate as u32),
            format!("~{} h", (secs / 3600).max(1)),
            format!("{}.{} years", tenths / 10, tenths % 10),
            format!("{miners} on it"),
        ];
        Some(format!("Core drill\n{bar}\n{}", hud_stats(&chips).unwrap_or_default()))
    }
}

#[cfg(test)]
mod tests;
