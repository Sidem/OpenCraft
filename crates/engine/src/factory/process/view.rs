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
        let special = self
            .pods_text()
            .or_else(|| self.steam_text())
            .or_else(|| self.solar_text())
            .or_else(|| self.hangar_text())
            .or_else(|| self.diesel_text())
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
                format!("{} {} · {rate} a minute{slow}", s.verb, item::name(r.main().0))
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
                let each: Vec<String> = r.inputs.iter().map(|&(i, n)| format!("{n} {}", item::name(i))).collect();
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

    /// `rates` in words: "Full speed: takes 30 Iron Ore, makes 30 Iron Ingot a minute".
    pub fn rates_text(&self) -> Option<String> {
        let (ins, outs) = self.rates()?;
        let words = |list: Rates| -> String {
            let each: Vec<String> = list.iter().map(|&(i, t)| format!("{} {}", tenths(t), item::name(i))).collect();
            each.join(" and ")
        };
        Some(format!("Full speed: takes {}, makes {} a minute", words(ins), words(outs)))
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

    /// Status and rate, what it holds and its fire.
    pub(super) fn readout(&self) -> String {
        let mut lines = vec![self.status_text()];
        lines.extend(self.rates_text());
        let held = |label: &str, b: &Buffer| {
            let each: Vec<String> =
                b.contents().iter().map(|s| format!("{} {}", s.count, item::name(s.item))).collect();
            (!each.is_empty()).then(|| format!("{label} {}", each.join(", ")))
        };
        // A depot's store has its own line (`Holding`); its perk takes in and gives out the front.
        let (into, side) =
            if self.spec.pick == Pick::Depot { ("Perk in:", "Out the front:") } else { ("In:", "Side:") };
        let mut parts: Vec<String> =
            [held(into, &self.input), held("Fuel:", &self.fuel)].into_iter().flatten().collect();
        if self.burn > 0 {
            parts.push(format!("fire for {} s", self.fire_seconds()));
        }
        parts.extend(held("Out:", &self.out));
        parts.extend(held(side, &self.side));
        if !parts.is_empty() {
            lines.push(parts.join(" · "));
        }
        lines.extend(self.launch_line());
        if self.spec.pick != Pick::Eject
            && !matches!(
                self.energy(),
                Energy::Turbine
                    | Energy::Solar
                    | Energy::Accumulator
                    | Energy::Hydro
                    | Energy::Rectenna
                    | Energy::Hoist
                    | Energy::Optimizer
            )
        {
            lines.push("Right-click to open".to_string());
        }
        lines.join("\n")
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
    let batch = u64::from((ticks(r.seconds) * FULL_SPEED).div_ceil(work.max(1)));
    ((u64::from(n) * 600 * u64::from(TICK_RATE) + batch / 2) / batch) as u32
}

/// Tenths as words: "7.5", or "30" when whole (integers only: float formatting costs wasm size).
pub fn tenths(t: u32) -> String {
    if t.is_multiple_of(10) {
        (t / 10).to_string()
    } else {
        format!("{}.{}", t / 10, t % 10)
    }
}
