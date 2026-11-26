//! The space elevator for the host (queries only): the elevator part of its anchor's panel (`web/src/ui/elevator.ts`)
//! and its news, which the HUD toasts when a section is done or the climbers stall (`web/src/ui/hud.ts`).

use wasm_bindgen::prelude::*;

use crate::elevator::{Elevator, SECTIONS, SECTION_HEIGHT};
use crate::item::{name, ELEVATOR_ANCHOR_ITEM};
use crate::math::IVec3;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// The elevator anchor as an item, for the icon of its news.
    pub fn elevator_anchor_item(&self) -> u32 {
        u32::from(ELEVATOR_ANCHOR_ITEM.0)
    }

    /// The elevator whose anchor has a cell at (x, y, z) as `[sections built, sections in all, blocks a section,
    /// stalled, waiting for its town, order lines, then (item, delivered, wanted) per line of the open order, draw
    /// lines, then (item, a minute, held) per good the climbers use]`. Empty when no elevator's anchor is there.
    pub fn elevator_panel(&self, x: i32, y: i32, z: i32) -> Vec<u32> {
        let Some(e) = self.elevator_at(IVec3::new(x, y, z)) else { return Vec::new() };
        let waiting = self.sim.settlements.towns.get(e.town as usize).is_none_or(|t| t.stalled || t.dormant);
        let mut out =
            vec![u32::from(e.built), SECTIONS as u32, SECTION_HEIGHT as u32, u32::from(e.stalled), u32::from(waiting)];
        out.push(e.order().len() as u32);
        for (&(item, want), &got) in e.order().iter().zip(&e.delivered) {
            out.extend([u32::from(item.0), got, want]);
        }
        let held = self.sim.factory.depot_held(e.pos);
        let draw = if e.done() { &[][..] } else { crate::elevator::DRAW };
        out.push(draw.len() as u32);
        for &(item, n) in draw {
            out.extend([u32::from(item.0), n, held.map_or(0, |b| b.count_like(item))]);
        }
        out
    }

    /// The elevator's state for the HUD, which toasts when it changes: 0 for none or asleep, else 1 + 2 × sections
    /// built + 1 while stalled.
    pub fn elevator_state(&self) -> u32 {
        match &self.sim.elevator {
            Some(e) if !e.dormant => 1 + 2 * u32::from(e.built) + u32::from(e.stalled),
            _ => 0,
        }
    }

    /// Its news now: "The space elevator stands 192 blocks tall: section 3 of 8", "Phase 1 of the space elevator is
    /// done", or "The elevator's climbers are short of Plastic and Circuit"; empty with no elevator.
    pub fn elevator_news(&self) -> String {
        let Some(e) = &self.sim.elevator else { return String::new() };
        if e.stalled {
            let held = self.sim.factory.depot_held(e.pos);
            let short: Vec<&str> = (e.draw().iter())
                .filter(|&&(item, n)| held.is_none_or(|b| b.count_like(item) < n))
                .map(|&(item, _)| name(item))
                .collect();
            return format!("The elevator's climbers are short of {}", short.join(", "));
        }
        if e.done() {
            return "Phase 1 of the space elevator is done".to_string();
        }
        let tall = i32::from(e.built) * SECTION_HEIGHT;
        format!("The space elevator stands {tall} blocks tall: section {} of {SECTIONS}", e.built)
    }
}

impl Game {
    /// The elevator whose anchor has a cell at `cell`.
    fn elevator_at(&self, cell: IVec3) -> Option<&Elevator> {
        let anchor = self.sim.factory.footprint_at(cell)?.0;
        self.sim.elevator.as_ref().filter(|e| e.pos == anchor)
    }
}
