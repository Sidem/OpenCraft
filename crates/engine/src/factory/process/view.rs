//! What a processor says: its status line (the readout's first line and the panel's), what it takes and makes a
//! minute at full speed (`rates`, the second line), the readout, and its panel (`panel.rs`). Presentation only.

use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::recipes::{MachineRecipe, MACHINE_RECIPES};
use crate::TICK_RATE;

use super::super::buffer::Buffer;
use super::super::panel::{Panel, ROLE_FUEL, ROLE_INPUT, ROLE_OUTPUT, ROLE_STORE};
use super::super::power::{FULL_SPEED, NOT_WIRED};
use super::super::ticks;
use super::{Energy, Pick, Processor, Status};

/// Items, each with tenths of an item a minute.
pub type Rates = Vec<(ItemId, u32)>;

impl Processor {
    /// The first readout line, also the panel's status.
    pub fn status_text(&self) -> String {
        if self.status == Status::Off {
            return super::super::switch::SWITCHED_OFF.to_string();
        }
        let special = self
            .pods_text()
            .or_else(|| self.steam_text())
            .or_else(|| self.solar_text())
            .or_else(|| self.hangar_text())
            .or_else(|| self.diesel_text())
            .or_else(|| self.gas_text())
            .or_else(|| self.hydro_text())
            .or_else(|| self.rectenna_text())
            .or_else(|| self.hoist_text())
            .or_else(|| self.reactor_text())
            .or_else(|| self.center_text())
            .or_else(|| self.recycle_text())
            .or_else(|| self.eject_text())
            .or_else(|| self.launch_text())
            .or_else(|| self.half_fed_text())
            .or_else(|| self.datacenter_text())
            .or_else(|| self.tower_text())
            .or_else(|| self.optimizer_text())
            .or_else(|| self.hub_text())
            .or_else(|| self.ground_text())
            .or_else(|| self.forest_text())
            .or_else(|| self.laser_text());
        if let Some(text) = special.or_else(|| self.pump_text()) {
            return text;
        }
        let s = self.spec;
        match (self.status, self.batch_recipe().or(self.chosen())) {
            (Status::Working, Some(r)) => {
                let share = self.stats().speed as f64 * self.speed as f64 * self.boost as f64
                    / (FULL_SPEED * FULL_SPEED * FULL_SPEED) as f64;
                let rate = (r.main().1 as f64 * 60.0 / r.seconds * share).round() as u32;
                let slow =
                    if self.speed < FULL_SPEED { format!(" (low power: {}%)", self.speed / 10) } else { String::new() };
                let out = crate::wood::made_from(r.main().0, self.taken.iter().map(|s| s.item));
                format!("{} {} · {rate} a minute{slow}", s.verb, item::name(out))
            }
            (Status::NoFuel, _) => "Out of fuel: bring coal ore or logs".to_string(),
            (Status::NoWater, _) => "Out of water: pipe its blue inlet to a pump with water in reach".to_string(),
            (Status::NoPower, _) => NOT_WIRED.to_string(),
            (Status::OutputFull, r) => match r.or_else(|| self.held_recipe()).and_then(|r| self.full_output(r)) {
                Some((item, true)) => {
                    format!(
                        "{} has nowhere to go: put a belt leading away from the side port, or take it",
                        item::name(item)
                    )
                }
                _ => format!("Output full: put a belt leading away, or take the {}", s.products),
            },
            (Status::NoRecipe, _) => "No recipe: choose what it makes".to_string(),
            (_, Some(r)) if s.pick == Pick::Chosen => {
                // Only what is missing: a list of every input reads as if all of them were.
                let lacking = self.lacking();
                let list = if lacking.is_empty() { r.inputs.to_vec() } else { lacking };
                let each: Vec<String> = list.iter().map(|&(i, n)| format!("{n} {}", item::name(i))).collect();
                format!("Waiting for {}", each.join(", "))
            }
            _ => s.waiting.to_string(),
        }
    }

    /// What it takes and makes a minute at full speed (this tier, research's bonus, full power), in tenths of an item,
    /// for the recipe it runs, was given or would run from what it holds: `(inputs, outputs)`. None for a machine
    /// that does not work through recipes, or before it has one.
    pub fn rates(&self) -> Option<(Rates, Rates)> {
        if !matches!(self.spec.pick, Pick::Chosen | Pick::ByInput) {
            return None;
        }
        let r = self.batch_recipe().or(self.chosen()).or_else(|| self.held_recipe())?;
        // Work a tick at full power, as `work` counts it.
        let work = self.stats().speed * self.boost / FULL_SPEED;
        let a_minute = |list: &[(ItemId, u32)]| list.iter().map(|&(i, n)| (i, tenths_a_minute(r, n, work))).collect();
        (work > 0).then(|| (a_minute(r.inputs), a_minute(r.outputs)))
    }

    /// `rates` as one short line for the readout: "Full speed: 30 Iron Ore → 30 Iron Ingot a minute". The panel draws
    /// the same numbers as a table, a minute and a second (`Factory::rate_rows`).
    #[cfg(test)]
    pub fn rates_text(&self) -> Option<String> {
        let (ins, outs) = self.rates()?;
        let words = |list: &Rates| -> String {
            let each: Vec<String> = list.iter().map(|&(i, t)| format!("{} {}", tenths(t), item::name(i))).collect();
            each.join(" + ")
        };
        Some(format!("Full speed: {} → {} a minute", words(&ins), words(&outs)))
    }

    /// Seconds one batch of recipe `r` takes in this machine at full power (its tier and research's bonus), counted in
    /// whole ticks as `work.rs` does: the time the recipe card shows, so a Mk3 smelter reads shorter than a Mk1.
    pub fn batch_seconds(&self, r: &MachineRecipe) -> f64 {
        let work = self.stats().speed * self.boost / FULL_SPEED;
        batch_ticks(r, work) as f64 / f64::from(TICK_RATE)
    }

    pub fn panel(&self) -> Panel {
        let chosen = self.spec.pick == Pick::Chosen;
        let progress = self.batch_recipe().map_or(0, |r| self.progress / ticks(r.seconds));
        let mut slots: Vec<(u8, Stack)> = self.input.slots.iter().map(|&s| (ROLE_INPUT, s)).collect();
        slots.extend(self.fuel.slots.iter().map(|&s| (ROLE_FUEL, s)));
        let store = if self.spec.pick == Pick::Depot { ROLE_STORE } else { ROLE_OUTPUT };
        slots.extend(self.out.slots.iter().map(|&s| (store, s)));
        slots.extend(self.side.slots.iter().map(|&s| (ROLE_OUTPUT, s)));
        Panel {
            block: self.spec.block,
            recipe: if chosen { self.recipe } else { self.batch },
            choosable: chosen,
            progress,
            fire: self.fire_seconds(),
            slots,
            status: self.status_text(),
            filter: None,
        }
    }

    /// Visual status pill (`@status|tone|text`) for the top-of-screen HUD readout: concise label + colour tone,
    /// leaving verbose tutorial wording out of hover cards.
    fn hud_status_line(&self) -> String {
        use super::super::describe::hud_status;
        if self.status == Status::Off {
            return hud_status("off", super::super::switch::SWITCHED_OFF);
        }
        if let super::Extra::Study(study) = &self.extra {
            use super::super::lab::LabStatus;
            return match study.status {
                None | Some(LabStatus::NoResearch) => {
                    format!("{}\n\tT=Research", hud_status("warn", "No research chosen"))
                }
                Some(LabStatus::Working) => {
                    hud_status(if self.speed < FULL_SPEED { "warn" } else { "ok" }, &self.status_text())
                }
                Some(LabStatus::NoPacks | LabStatus::AllTaken | LabStatus::NeedsCenter | LabStatus::NeedsAi) => {
                    hud_status("warn", &self.status_text())
                }
                Some(LabStatus::NoPower) => {
                    hud_status("err", if self.is_ai_lab() { "No power or compute" } else { "No power" })
                }
                Some(LabStatus::Off) => hud_status("off", super::super::switch::SWITCHED_OFF),
            };
        }
        if self.spec.pick == Pick::Store {
            let used = self.out.slots.iter().filter(|s| !s.is_empty()).count();
            let cap = self.out.slots.len();
            let tone = if used == cap { "warn" } else { "accent" };
            return super::super::describe::hud_bar(
                "Slots",
                used as u64,
                cap as u64,
                &format!("{used} / {cap} stacks"),
                tone,
            );
        }
        let special = self
            .pods_text()
            .or_else(|| self.steam_text())
            .or_else(|| self.solar_text())
            .or_else(|| self.hangar_text())
            .or_else(|| self.diesel_text())
            .or_else(|| self.gas_text())
            .or_else(|| self.hydro_text())
            .or_else(|| self.rectenna_text())
            .or_else(|| self.hoist_text())
            .or_else(|| self.reactor_text())
            .or_else(|| self.recycle_text())
            .or_else(|| self.eject_text())
            .or_else(|| self.launch_text())
            .or_else(|| self.half_fed_text())
            .or_else(|| self.datacenter_text())
            .or_else(|| self.tower_text())
            .or_else(|| self.optimizer_text())
            .or_else(|| self.hub_text())
            .or_else(|| self.ground_text())
            .or_else(|| self.forest_text())
            .or_else(|| self.laser_text())
            .or_else(|| self.pump_text());
        if let Some(text) = special {
            let tone = match self.status {
                Status::Working => {
                    if self.speed > 0 && self.speed < FULL_SPEED {
                        "warn"
                    } else {
                        "ok"
                    }
                }
                Status::NoPower | Status::NoFuel | Status::NoWater | Status::Overheated => "err",
                Status::OutputFull | Status::NoInput | Status::NoRecipe | Status::NoDeposit | Status::Exhausted => {
                    "warn"
                }
                Status::Off => "off",
            };
            return hud_status(tone, &text);
        }
        match (self.status, self.batch_recipe().or(self.chosen())) {
            (Status::Working, Some(r)) => {
                let share = self.stats().speed as f64 * self.speed as f64 * self.boost as f64
                    / (FULL_SPEED * FULL_SPEED * FULL_SPEED) as f64;
                let rate = (r.main().1 as f64 * 60.0 / r.seconds * share).round() as u32;
                let slow =
                    if self.speed < FULL_SPEED { format!(" · low power {}%", self.speed / 10) } else { String::new() };
                let out = crate::wood::made_from(r.main().0, self.taken.iter().map(|s| s.item));
                let tone = if self.speed < FULL_SPEED { "warn" } else { "ok" };
                hud_status(tone, &format!("{} {} · {rate}/m{slow}", self.spec.verb, item::name(out)))
            }
            (Status::NoFuel, _) => hud_status("err", "Out of fuel"),
            (Status::NoWater, _) => hud_status("err", "Out of water"),
            (Status::NoPower, _) => hud_status("err", "No power"),
            (Status::OutputFull, r) => match r.or_else(|| self.held_recipe()).and_then(|r| self.full_output(r)) {
                Some((item, true)) => hud_status("warn", &format!("Side output full ({})", item::name(item))),
                _ => hud_status("warn", "Output full"),
            },
            (Status::NoRecipe, _) => hud_status("warn", "No recipe chosen"),
            _ => hud_status("warn", &self.status_text()),
        }
    }

    /// Structured visual recipe flow row (`@recipe|in_id:tenths,...|out_id:tenths,...`) for `target-card.ts`.
    fn hud_recipe_line(&self) -> Option<String> {
        let (ins, outs) = self.rates()?;
        let fmt = |list: &Rates| list.iter().map(|&(i, t)| format!("{}:{t}", i.0)).collect::<Vec<_>>().join(",");
        Some(format!("@recipe|{}|{}", fmt(&ins), fmt(&outs)))
    }

    /// Status, visual recipe flow, icon inventory slots, and fire/controls.
    pub(super) fn readout(&self) -> String {
        use super::super::describe::{hud_items, hud_stats};
        let mut lines = vec![self.hud_status_line()];
        lines.extend(self.hud_recipe_line());
        let stacks = |b: &Buffer| -> Vec<(ItemId, u32)> {
            let mut merged: Vec<(ItemId, u32)> = Vec::new();
            for s in b.contents() {
                match merged.iter_mut().find(|m| m.0 == s.item) {
                    Some(m) => m.1 += s.count,
                    None => merged.push((s.item, s.count)),
                }
            }
            merged
        };
        // A depot's store has its own line (`Holding`); its perk takes in and gives out the front.
        let (into, side) = if self.spec.pick == Pick::Depot { ("Perk in", "Front out") } else { ("In", "Side") };
        lines.extend(hud_items(into, &stacks(&self.input)));
        lines.extend(hud_items("Fuel", &stacks(&self.fuel)));
        lines.extend(hud_items("Out", &stacks(&self.out)));
        lines.extend(hud_items(side, &stacks(&self.side)));
        if self.burn > 0 {
            lines.extend(hud_stats(&[format!("Fire {} s", self.fire_seconds())]));
        }
        lines.extend(self.launch_line());
        if self.spec.pick != Pick::Eject
            && !matches!(
                self.energy(),
                Energy::Turbine | Energy::Solar | Energy::Accumulator | Energy::Hydro | Energy::Rectenna
            )
        {
            lines.push("\tRMB=Open".to_string());
        }
        lines.join("\n")
    }

    /// What a machine with a chosen recipe still lacks for its next batch, and how many of each: empty unless it is
    /// waiting for input.
    pub fn lacking(&self) -> Vec<(ItemId, u32)> {
        let Some(r) = self.chosen().filter(|_| self.status == Status::NoInput && self.spec.pick == Pick::Chosen) else {
            return Vec::new();
        };
        r.inputs
            .iter()
            .filter_map(|&(i, n)| n.checked_sub(self.input.count_like(i)).filter(|&m| m > 0).map(|m| (i, m)))
            .collect()
    }

    /// Whether a faster Mk would do more: a machine with a chosen recipe, working at full pace, with every input piled up
    /// (two batches' worth, or a full stack). A machine short of power, starved or full gains nothing from a kit.
    pub fn is_bottleneck(&self) -> bool {
        let Some(r) = self.chosen().filter(|_| self.status == Status::Working && self.spec.pick == Pick::Chosen) else {
            return false;
        };
        let at_pace = !self.draws_power() || self.speed >= FULL_SPEED;
        at_pace && r.inputs.iter().all(|&(i, n)| self.input.count_like(i) >= (2 * n).min(item::stack_size(i)))
    }

    /// The product or byproduct it has stopped for, having nowhere to put it.
    pub fn blocked_output(&self) -> Option<ItemId> {
        if self.status != Status::OutputFull {
            return None;
        }
        let r = self.batch_recipe().or(self.chosen()).or_else(|| self.held_recipe())?;
        self.full_output(r).map(|(item, _)| item)
    }

    /// The recipe the first input it holds would run (a machine that picks by input), whatever research allows.
    fn held_recipe(&self) -> Option<&'static MachineRecipe> {
        let held = self.input.slots.iter().find(|s| !s.is_empty())?;
        self.spec.recipe(self.spec.recipe_using(held.item, &[true; MACHINE_RECIPES.len()])?)
    }

    /// Seconds the fire lasts at this tier's full speed.
    fn fire_seconds(&self) -> u32 {
        let per_tick = (self.stats().speed * self.stats().fuel / 1000).max(1);
        self.burn.div_ceil(per_tick * TICK_RATE)
    }
}

/// Tenths of an item a minute that `n` a batch of `r` comes to at `work` thousandths of a Mk1 tick each tick (a
/// tier's `speed` at full power), a batch taking whole ticks as `work.rs` counts them.
pub fn tenths_a_minute(r: &MachineRecipe, n: u32, work: u32) -> u32 {
    let batch = batch_ticks(r, work);
    ((u64::from(n) * 600 * u64::from(TICK_RATE) + batch / 2) / batch) as u32
}

/// Ticks one batch of `r` takes at `work` thousandths of a Mk1 tick each tick.
fn batch_ticks(r: &MachineRecipe, work: u32) -> u64 {
    u64::from((ticks(r.seconds) * FULL_SPEED).div_ceil(work.max(1)))
}

/// Tenths as words: "7.5", or "30" when whole (integers only: float formatting costs wasm size).
pub fn tenths(t: u32) -> String {
    if t.is_multiple_of(10) {
        (t / 10).to_string()
    } else {
        format!("{}.{}", t / 10, t % 10)
    }
}
