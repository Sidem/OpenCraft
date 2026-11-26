//! Placing a quarry (the local player's, presentation only): while one is held and aimed at a free cell,
//! `quarry_preview` is the box it would dig there (outlined by the host), its label says how big it is and what is in
//! it, and R turns it a quarter (`turn_placement`, from `rotate_target`; multi-block machines and the forester,
//! `forester_preview.rs`, share the turn: `footprint_preview.rs`). The turn goes into the placing action's facing
//! (`interaction.rs`). Reads loaded chunks only; never touches the core.

use crate::block::{self, BlockId, FORESTER, OUTLET, PIPE, PUMP, QUARRY};
use crate::factory::{self, survey, DigBox, DEFAULT_DEPTH, DEFAULT_WIDTH, DEPTHS, DIRS, FACES};
use crate::item;
use crate::math::IVec3;
use crate::Game;

impl Game {
    /// Whether the selected slot holds a quarry.
    pub(crate) fn holds_quarry(&self) -> bool {
        self.holds(QUARRY)
    }

    /// Whether the selected slot holds a pipe outlet, which shows a rotatable ghost and turns with R.
    pub(crate) fn holds_outlet(&self) -> bool {
        self.holds(OUTLET)
    }

    /// Whether the selected slot holds a machine that works a box in front of it, which R turns.
    pub(crate) fn holds_boxed(&self) -> bool {
        self.holds(QUARRY) || self.holds(FORESTER)
    }

    fn holds(&self, block: BlockId) -> bool {
        let stack = self.inventory().selected_stack();
        !stack.is_empty() && stack.item.places() == Some(block)
    }

    /// The way a quarry or multi-block machine placed now would face: the player's facing, turned by R.
    pub(crate) fn placing_facing(&self) -> u8 {
        (factory::dir_from_yaw(self.body().yaw) + self.place_turn) % 4
    }

    /// The way a held outlet placed against a face with `normal` would pour: defaults to pointing outward along a
    /// horizontal clicked face (so attaching to the end of a pipe points away from the pipe instead of into it),
    /// otherwise along the player's facing, then turned clockwise by R (`place_turn`).
    pub(crate) fn outlet_facing(&self, normal: IVec3) -> u8 {
        let base =
            DIRS.iter().position(|&d| d == normal).map_or_else(|| factory::dir_from_yaw(self.body().yaw), |i| i as u8);
        (base + self.place_turn) % 4
    }

    /// Preview state of a held outlet: `(pos, facing, adjacent_pipe_arms, blocked)`. `blocked` is true when the
    /// target cell itself or the cell directly in front of the spout is solid.
    pub(crate) fn outlet_ghost(&self) -> Option<(IVec3, u8, u8, bool)> {
        let hit = self.target.filter(|h| h.normal != IVec3::ZERO && self.holds_outlet())?;
        let pos = hit.block + hit.normal;
        let cell_free = self.sim.world.get_block(pos).is_some_and(block::replaceable) && !self.body_blocks(OUTLET, pos);
        let facing = self.outlet_facing(hit.normal);
        let front = pos + DIRS[facing as usize];
        let front_free = self.sim.world.get_block(front).is_some_and(block::replaceable);
        let is_pipe = |c: IVec3| matches!(self.sim.world.get_block(c), Some(PIPE | PUMP | OUTLET));
        let arms = (0..6).filter(|&f| is_pipe(pos + FACES[f])).fold(0u8, |a, f| a | (1 << f));
        Some((pos, facing, arms, !cell_free || !front_free))
    }

    /// Placement card for a held outlet: shows spout direction, whether its front is clear or blocked, and R to turn.
    pub(crate) fn outlet_label(&self) -> String {
        let Some((pos, facing, _, _)) = self.outlet_ghost() else { return String::new() };
        const COMPASS: [&str; 4] = ["North", "East", "South", "West"];
        let front = pos + DIRS[facing as usize];
        let front_free = self.sim.world.get_block(front).is_some_and(block::replaceable);
        let status = if front_free {
            format!("Pours {} · clear", COMPASS[facing as usize])
        } else {
            format!("Pours {} · blocked by block in front (press R to turn)", COMPASS[facing as usize])
        };
        let keys = factory::hud_keys(&[("RMB", "Place"), ("R", "Turn")]);
        format!("Outlet\n{status}\n{keys}")
    }

    pub(crate) fn turn_placement(&mut self) {
        self.place_turn = (self.place_turn + 1) % 4;
    }

    /// The box a held quarry would dig if placed now, if it can be placed.
    pub(crate) fn quarry_preview(&self) -> Option<DigBox> {
        let hit = self.target.filter(|h| h.normal != IVec3::ZERO && self.holds_quarry())?;
        let pos = hit.block + hit.normal;
        self.sim.world.get_block(pos).filter(|&b| block::replaceable(b))?;
        Some(DigBox::new(pos, self.placing_facing(), DEFAULT_WIDTH, DEFAULT_DEPTH))
    }

    /// "Quarry" and "7×7, 16 deep · about 600 blocks: stone, dirt · R turns it" on two lines ("" when
    /// not placing one).
    pub(crate) fn quarry_label(&self) -> String {
        let Some(dig) = self.quarry_preview() else { return String::new() };
        let (n, drops) = survey(&dig, 0, &self.sim.world);
        let about = if n >= 100 { (n + 5) / 10 * 10 } else { n };
        let names: Vec<String> = drops.iter().map(|&d| item::name(d).to_ascii_lowercase()).collect();
        let what = if names.is_empty() { String::new() } else { format!(": {}", names.join(", ")) };
        let (w, depth) = (dig.width, DEPTHS[DEFAULT_DEPTH as usize].label());
        let keys = factory::hud_keys(&[("RMB", "Place"), ("R", "Turn")]);
        format!("Quarry\n{w}×{w}, {depth} · about {about} blocks{what}\n{keys}")
    }
}
