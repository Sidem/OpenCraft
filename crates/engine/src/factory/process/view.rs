//! What a processor says: its status line (the readout's first line and the panel's), the readout, and
//! its panel (`panel.rs`). Presentation only.

use crate::inventory::Stack;
use crate::item;
use crate::recipes::{MachineRecipe, MACHINE_RECIPES};
use crate::TICK_RATE;

use super::super::buffer::Buffer;
use super::super::panel::{Panel, ROLE_FUEL, ROLE_INPUT, ROLE_OUTPUT};
use super::super::power::{FULL_SPEED, NOT_WIRED};
use super::super::ticks;
use super::{Energy, Pick, Processor, Status};

impl Processor {
    /// The first readout line, also the panel's status.
    pub fn status_text(&self) -> String {
        let special = self
            .steam_text()
            .or_else(|| self.solar_text())
            .or_else(|| self.hangar_text())
            .or_else(|| self.diesel_text())
            .or_else(|| self.hydro_text())
            .or_else(|| self.hoist_text())
            .or_else(|| self.reactor_text())
            .or_else(|| self.center_text());
        if let Some(text) = special.or_else(|| self.pump_text()) {
            return text;
        }
        let s = self.spec;
        match (self.status, self.batch_recipe().or(self.chosen())) {
            (Status::Working, Some(r)) => {
                let share = self.stats().speed as f64 * self.speed as f64 / (FULL_SPEED * FULL_SPEED) as f64;
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

    pub fn panel(&self) -> Panel {
        let chosen = self.spec.pick == Pick::Chosen;
        let progress = self.batch_recipe().map_or(0, |r| self.progress / ticks(r.seconds));
        let mut slots: Vec<(u8, Stack)> = self.input.slots.iter().map(|&s| (ROLE_INPUT, s)).collect();
        slots.extend(self.fuel.slots.iter().map(|&s| (ROLE_FUEL, s)));
        slots.extend(self.out.slots.iter().chain(&self.side.slots).map(|&s| (ROLE_OUTPUT, s)));
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
        let held = |label: &str, b: &Buffer| {
            let each: Vec<String> =
                b.contents().iter().map(|s| format!("{} {}", s.count, item::name(s.item))).collect();
            (!each.is_empty()).then(|| format!("{label} {}", each.join(", ")))
        };
        let mut parts: Vec<String> =
            [held("In:", &self.input), held("Fuel:", &self.fuel)].into_iter().flatten().collect();
        if self.burn > 0 {
            parts.push(format!("fire for {} s", self.fire_seconds()));
        }
        parts.extend(held("Out:", &self.out));
        parts.extend(held("Side:", &self.side));
        if !parts.is_empty() {
            lines.push(parts.join(" · "));
        }
        if !matches!(
            self.energy(),
            Energy::Turbine | Energy::Solar | Energy::Accumulator | Energy::Hydro | Energy::Hoist
        ) {
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
