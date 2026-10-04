//! Ghost mode (the local player's hands and view, presentation only; the ghosts themselves are core
//! state: `ghosts.rs`). B toggles it. In ghost mode the held block or machine is planted as a ghost where
//! the crosshair points (`Action::PlaceGhost`, nothing is used; aiming at a ghost again removes it), R
//! turns what will be planted, the reach is longer, and mining is off. Ghosts within [`DRAW_RANGE`] are
//! outlined cyan through the terrain (`ghost_boxes`, appended to `placement_box`), and the HUD label lists
//! what building them all needs. Building the real block on a ghost is just placing it (`Sim::place_block`).
//! Left-click (held to drag) marks the aimed block for tear-down instead of mining it (`Action::MarkRemoval`, a
//! red outline; clicking a marked block clears the mark): drone ports break marked blocks (`drones/`).

use crate::action::Action;
use crate::block;
use crate::item;
use crate::math::IVec3;
use crate::Game;

/// How far the hands reach in ghost mode (normally `interaction::REACH`).
pub(crate) const GHOST_REACH: f64 = 16.0;
/// Ghosts further than this from the eye are not outlined.
const DRAW_RANGE: f64 = 64.0;
/// Outline boxes drawn at most.
const MAX_DRAWN: usize = 600;
/// Ghost outline colour (cyan).
const GHOST_CYAN: i32 = 0x4cc9f0;
/// Tear-down mark outline colour (red).
const MARK_RED: i32 = 0xff4d4d;
/// Needs listed in the HUD label.
const LABEL_NEEDS: usize = 3;

impl Game {
    /// Reach of the hands now.
    pub(crate) fn reach(&self) -> f64 {
        if self.ghost_mode {
            GHOST_REACH
        } else {
            crate::interaction::REACH
        }
    }

    /// One use-button press in ghost mode: remove the ghost in the aimed cell, else plant the held block's.
    pub(crate) fn update_ghosts(&mut self) {
        if !self.using {
            return;
        }
        self.using = false;
        if self.stamp_held() {
            return;
        }
        let Some(hit) = self.target.filter(|h| h.normal != IVec3::ZERO) else { return };
        let pos = hit.block + hit.normal;
        if self.sim.ghosts.covering(pos).is_some() {
            self.act(Action::RemoveGhost { pos });
            return;
        }
        let inv = self.inventory();
        let stack = inv.selected_stack();
        if stack.is_empty() || stack.item.places().is_none() {
            return;
        }
        let slot = inv.selected as u8;
        self.act(Action::PlaceGhost { pos, slot, facing: self.placing_facing() });
    }

    /// Left button in ghost mode: mark the aimed block for tear-down, or clear its mark. One block per press
    /// (`mine_block` remembers the last, so holding and sweeping marks each block once).
    pub(crate) fn update_marking(&mut self) {
        let Some(hit) = self.target.filter(|_| self.mining) else {
            self.mine_block = None;
            return;
        };
        if self.mine_block == Some(hit.block) {
            return;
        }
        self.mine_block = Some(hit.block);
        let pos = self.sim.factory.footprint_at(hit.block).map_or(hit.block, |f| f.0);
        if self.sim.ghosts.covering(pos).is_some_and(|g| g.block == block::AIR) {
            self.act(Action::RemoveGhost { pos });
        } else if block::def(hit.id).break_time >= 0.0 {
            self.act(Action::MarkRemoval { pos });
        }
    }

    /// Outlines of the ghosts near the eye: 7 numbers each (lowest and highest cell, colour).
    pub(crate) fn ghost_boxes(&self) -> Vec<i32> {
        let eye = self.body().eye();
        let mut out = Vec::new();
        for g in self.sim.ghosts.iter() {
            if out.len() >= MAX_DRAWN * 7 {
                break;
            }
            if (g.pos.as_vec3() - eye).length() > DRAW_RANGE {
                continue;
            }
            if g.block == block::AIR {
                let cells = self.sim.factory.footprint_at(g.pos).map_or_else(|| vec![g.pos], |f| f.2);
                push_box(&mut out, &cells, MARK_RED);
            } else {
                push_box(&mut out, &g.cells(), GHOST_CYAN);
            }
        }
        out
    }

    /// The plan line the HUD shows in ghost mode: a title, then how to use it and what the ghosts need.
    pub(crate) fn ghost_label(&self) -> String {
        if let Some(label) = self.blueprint_label() {
            return label;
        }
        let inv = self.inventory();
        let stack = inv.selected_stack();
        let held = stack.item.places().filter(|_| !stack.is_empty());
        let title = match held {
            Some(b) => format!("Ghost mode · {}", block::def(b).name),
            None => "Ghost mode · hold a block or machine".to_string(),
        };
        let mut lines = vec![format!(
            "Right-click plants a ghost, left-click marks a block for tear-down · R turns · B leaves · {} blocks reach",
            self.reach() as i32
        )];
        let needs = self.sim.ghosts.needs();
        let marks = self.sim.ghosts.iter().filter(|g| g.block == block::AIR).count();
        if marks > 0 {
            lines.push(format!("{marks} marked for tear-down"));
        }
        if needs.is_empty() && marks == 0 {
            lines.push("No ghosts yet".to_string());
        } else if !needs.is_empty() {
            let list: Vec<String> = needs
                .iter()
                .take(LABEL_NEEDS)
                .map(|&(it, n)| format!("{n} {} (have {})", item::name(it), inv.count(it)))
                .collect();
            let more = if needs.len() > LABEL_NEEDS { ", ..." } else { "" };
            lines.push(format!("{} ghosts need {}{more}", self.sim.ghosts.len() - marks, list.join(", ")));
        }
        format!("{title}\n{}", lines.join("\n"))
    }

    /// The amber box of the cell a single block would be planted in (multi-block machines have their own).
    pub(crate) fn ghost_cell_box(&self) -> Vec<i32> {
        let hit = self.target.filter(|h| h.normal != IVec3::ZERO && self.ghost_mode);
        let held = self.held_footprint();
        match (hit, held) {
            (Some(h), None) => {
                let p = h.block + h.normal;
                vec![p.x, p.y, p.z, p.x, p.y, p.z, 0]
            }
            _ => Vec::new(),
        }
    }
}

/// Appends the outline of the box around cells (7 numbers: lowest cell, highest cell, colour).
pub(crate) fn push_box(out: &mut Vec<i32>, cells: &[IVec3], colour: i32) {
    let min = |f: fn(&IVec3) -> i32| cells.iter().map(f).min().unwrap_or(0);
    let max = |f: fn(&IVec3) -> i32| cells.iter().map(f).max().unwrap_or(0);
    out.extend([min(|c| c.x), min(|c| c.y), min(|c| c.z), max(|c| c.x), max(|c| c.y), max(|c| c.z), colour]);
}
