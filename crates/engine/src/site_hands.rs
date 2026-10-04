//! The planner's hands (the local player's, presentation only): with a planner in hand the use button marks a
//! terraforming site (`factory/sites.rs`) from two corner blocks up to `PLANNER_REACH` away. The first click
//! sets a corner (clicking the same column again drops it), the second asks the host's site panel (`ui/site.ts`)
//! to choose the job and level (`take_site_request`: `[0, ax, az, bx, bz, ay, by]`); a click on a marked
//! site with no corner set asks for that site instead (`[1, id]`, to look at it or remove it). The panel
//! queues `mark_site` / `remove_site`, so nothing here touches the core. Outlines of the sites, the corner and
//! the box being marked are appended to `placement_box` (`api/hud.rs`), the label to `line_label`.

use wasm_bindgen::prelude::*;

use crate::item::PLANNER;
use crate::math::IVec3;
use crate::Game;

/// How far the hands reach with a planner in hand.
pub(crate) const PLANNER_REACH: f64 = 64.0;
/// Outline colours: a corner being marked (cyan), a dig site (orange), a fill site (green), a flatten site (gold)
/// and the level of every site (white).
const CORNER_CYAN: i32 = 0x4cc9f0;
const SITE_COLOURS: [i32; 3] = [0xff9f1c, 0x80ed99, 0xffd166];
const LEVEL_WHITE: i32 = 0xf1f5f9;

/// The planner's state: the first corner while one is set, and the request waiting for the host.
#[derive(Default)]
pub struct Planner {
    corner: Option<IVec3>,
    request: Option<Vec<i32>>,
}

impl Game {
    /// Whether a planner is in hand.
    pub(crate) fn planner_held(&self) -> bool {
        self.inventory().selected_stack().item == PLANNER
    }

    /// Runs the planner for one tick. Returns whether one is in hand (the use button is then its own).
    pub(crate) fn update_planner(&mut self) -> bool {
        if !self.planner_held() {
            self.planner.corner = None;
            return false;
        }
        if !self.using {
            return true;
        }
        self.using = false;
        let Some(b) = self.target.map(|h| h.block) else { return true };
        match self.planner.corner.take() {
            Some(a) if (a.x, a.z) == (b.x, b.z) => {}
            Some(a) => self.planner.request = Some(vec![0, a.x, a.z, b.x, b.z, a.y, b.y]),
            None => match self.sim.factory.sites.list.iter().find(|s| s.has_column(b)) {
                Some(s) => self.planner.request = Some(vec![1, s.id as i32]),
                None => self.planner.corner = Some(b),
            },
        }
        true
    }

    /// Outlines (7 numbers each) while a planner is in hand: every site (its extent and its level) and the
    /// box being marked.
    pub(crate) fn site_boxes(&self) -> Vec<i32> {
        let mut out = Vec::new();
        if !self.planner_held() {
            return out;
        }
        for s in &self.sim.factory.sites.list {
            let (bottom, top) = (s.low.min(s.level), s.high.max(s.level));
            let colour = SITE_COLOURS[s.job as usize];
            out.extend([s.lo.0, bottom, s.lo.1, s.hi.0, top, s.hi.1, colour]);
            out.extend([s.lo.0, s.level, s.lo.1, s.hi.0, s.level, s.hi.1, LEVEL_WHITE]);
        }
        if let Some(a) = self.planner.corner {
            let b = self.target.map_or(a, |h| h.block);
            out.extend([
                a.x.min(b.x),
                a.y.min(b.y),
                a.z.min(b.z),
                a.x.max(b.x),
                a.y.max(b.y),
                a.z.max(b.z),
                CORNER_CYAN,
            ]);
        }
        out
    }

    /// The HUD label with a planner in hand; `None` otherwise.
    pub(crate) fn planner_label(&self) -> Option<String> {
        if !self.planner_held() {
            return None;
        }
        Some(match self.planner.corner {
            Some(a) => format!(
                "Planner\nCorner at {}, {}, {} · right-click the opposite corner (or this column to drop it)",
                a.x, a.y, a.z
            ),
            None => "Planner\nRight-click a corner block to mark a site · right-click a marked site to edit it".into(),
        })
    }
}

#[wasm_bindgen]
impl Game {
    /// What the planner wants the site panel to show, once: `[0, ax, az, bx, bz, ay, by]` for a new area,
    /// `[1, id]` for a marked site; empty when nothing is waiting.
    pub fn take_site_request(&mut self) -> Vec<i32> {
        self.planner.request.take().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests;
