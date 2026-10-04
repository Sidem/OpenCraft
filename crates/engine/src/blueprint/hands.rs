//! The hands of blueprints (the local player's, presentation only): in ghost mode Z marks a box's two
//! corners (a third press clears it), Enter copies the machines in it as a new blueprint and takes it in hand,
//! and with one in hand the use button stamps its ghosts where the crosshair points (`Action::PlantGhost`
//! per machine; nothing is built or used), R turns it a quarter and Z puts it away. Outlines are appended
//! to `placement_box` (`api/hud.rs`) and the label replaces the ghost label (`ghost_mode.rs`).

use super::{Blueprint, Library, MAX_BLUEPRINTS};
use crate::action::Action;
use crate::item;
use crate::math::IVec3;
use crate::Game;

/// Selection box outline colour (gold) and the stamp preview's (pale cyan, its box green).
const SELECT_GOLD: i32 = 0xffd166;
const PREVIEW_PALE: i32 = 0xcaf0f8;
const PREVIEW_GREEN: i32 = 0x80ed99;
/// Machine outlines drawn in a preview at most.
const MAX_PREVIEW: usize = 300;
/// Shortages listed in the label.
const LABEL_MISSING: usize = 3;

impl Game {
    /// The block the crosshair is on, in ghost mode.
    fn aimed_block(&self) -> Option<IVec3> {
        self.target.filter(|_| self.ghost_mode).map(|h| h.block)
    }

    /// The cell a stamp's lowest corner would take: the free cell against the aimed face.
    fn stamp_origin(&self) -> Option<IVec3> {
        self.target.filter(|h| h.normal != IVec3::ZERO && self.ghost_mode).map(|h| h.block + h.normal)
    }

    /// Z: mark a corner, finish the box, or clear it; with a blueprint in hand, put it away.
    pub(crate) fn mark_blueprint_corner(&mut self) {
        if !self.ghost_mode {
            return;
        }
        let aimed = self.aimed_block();
        let lib = &mut self.library;
        if lib.held.take().is_some() {
            lib.note = "Blueprint put away".into();
        } else if lib.selection.take().is_some() {
            lib.note = "Selection cleared".into();
        } else if let (Some(a), Some(b)) = (lib.corner, aimed) {
            lib.corner = None;
            lib.selection = Some(span(a, b));
            lib.note = "Box selected".into();
        } else if let Some(a) = lib.corner.take() {
            lib.note = format!("Corner at {}, {}, {} dropped: aim at a block first", a.x, a.y, a.z);
        } else if let Some(b) = aimed {
            lib.corner = Some(b);
            lib.note = "Corner marked: aim at the opposite corner and press Z".into();
        }
    }

    /// Enter: copy the machines in the selected box as a new blueprint and take it in hand.
    pub(crate) fn copy_blueprint(&mut self) {
        let Some((lo, hi)) = self.library.selection.filter(|_| self.ghost_mode) else { return };
        if self.library.list.len() >= MAX_BLUEPRINTS {
            self.library.note = "The blueprint library is full: delete one first".into();
            return;
        }
        let name = format!("Blueprint {}", self.library.list.len() + 1);
        let lib = &mut self.library;
        match Blueprint::copy(&self.sim, lo, hi, name) {
            Ok(bp) => {
                lib.note = format!("Copied {} machines", bp.entries.len());
                lib.list.push(bp);
                lib.held = Some(lib.list.len() - 1);
                lib.turns = 0;
                lib.selection = None;
                lib.version += 1;
            }
            Err(why) => lib.note = why.into(),
        }
    }

    /// The use button with a blueprint in hand: plant its ghosts. False when none is in hand.
    pub(crate) fn stamp_held(&mut self) -> bool {
        let Some(bp) = self.library.held.and_then(|i| self.library.list.get(i)) else { return false };
        let Some(origin) = self.stamp_origin() else { return true };
        let ghosts = bp.ghosts_at(origin, self.library.turns);
        self.library.note = format!("Stamped {} ghosts", ghosts.len());
        for g in ghosts {
            self.act(Action::PlantGhost { pos: g.pos, block: g.block, facing: g.facing, tier: g.tier });
        }
        true
    }

    /// R in ghost mode with a blueprint in hand: turn it a quarter. False when none is in hand.
    pub(crate) fn turn_held_blueprint(&mut self) -> bool {
        let lib = &mut self.library;
        let held = lib.held.is_some();
        if held {
            lib.turns = (lib.turns + 1) % 4;
        }
        held
    }

    /// Outlines (7 numbers each): the box being marked or selected, and a held blueprint's stamp preview.
    pub(crate) fn blueprint_boxes(&self) -> Vec<i32> {
        let mut out = Vec::new();
        let lib = &self.library;
        let marking = lib.corner.zip(self.aimed_block()).map(|(a, b)| span(a, b));
        if let Some((lo, hi)) = lib.selection.or(marking) {
            out.extend([lo.x, lo.y, lo.z, hi.x, hi.y, hi.z, SELECT_GOLD]);
        } else if let Some(a) = lib.corner {
            out.extend([a.x, a.y, a.z, a.x, a.y, a.z, SELECT_GOLD]);
        }
        let (Some(bp), Some(origin)) = (lib.held.and_then(|i| lib.list.get(i)), self.stamp_origin()) else {
            return out;
        };
        let (size, _) = bp.turned(lib.turns);
        let hi = origin + size - IVec3::new(1, 1, 1);
        out.extend([origin.x, origin.y, origin.z, hi.x, hi.y, hi.z, PREVIEW_GREEN]);
        for g in bp.ghosts_at(origin, lib.turns).iter().take(MAX_PREVIEW) {
            crate::ghost_mode::push_box(&mut out, &g.cells(), PREVIEW_PALE);
        }
        out
    }

    /// The ghost mode label while a blueprint is being marked, selected or held; `None` otherwise.
    pub(crate) fn blueprint_label(&self) -> Option<String> {
        let lib = &self.library;
        let note = |default: &str| if lib.note.is_empty() { default.to_string() } else { lib.note.clone() };
        if let Some(bp) = lib.held.and_then(|i| lib.list.get(i)) {
            let title = format!("Blueprint Â· {} Â· {} machines", bp.name, bp.entries.len());
            let inv = self.inventory();
            let short: Vec<String> = bp
                .needs()
                .iter()
                .filter(|&&(it, n)| inv.count(it) < n)
                .map(|&(it, n)| format!("{} {}", n - inv.count(it), item::name(it)))
                .collect();
            let more = if short.len() > LABEL_MISSING { ", ..." } else { "" };
            let missing = if short.is_empty() {
                "You have everything it needs".to_string()
            } else {
                format!("Missing {}{more}", short.iter().take(LABEL_MISSING).cloned().collect::<Vec<_>>().join(", "))
            };
            let hint = note("Right-click stamps ghosts Â· R turns Â· Z puts it away");
            return Some(format!("{title}\n{hint}\n{missing}"));
        }
        if lib.selection.is_some() {
            return Some(format!("Box selected\n{} Â· Enter copies its machines Â· Z clears", note("")));
        }
        lib.corner.map(|_| format!("Marking a box\n{}", note("Aim at the opposite corner and press Z")))
    }
}

impl Library {
    /// Renames blueprint `i` (an empty name is ignored).
    pub fn rename(&mut self, i: usize, name: &str) {
        let name = super::clean_name(name);
        if let Some(bp) = self.list.get_mut(i).filter(|_| !name.is_empty()) {
            bp.name = name;
            self.version += 1;
        }
    }

    /// Deletes blueprint `i`, keeping the one in hand if it was another.
    pub fn delete(&mut self, i: usize) {
        if i >= self.list.len() {
            return;
        }
        self.list.remove(i);
        self.held = match self.held {
            Some(h) if h == i => None,
            Some(h) if h > i => Some(h - 1),
            other => other,
        };
        self.version += 1;
    }
}

/// The lowest and highest cell of the box between two corners.
fn span(a: IVec3, b: IVec3) -> (IVec3, IVec3) {
    (IVec3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)), IVec3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)))
}
