//! Local player's hands for shaped building blocks (`L-SLAB`, presentation only — acts only by queuing
//! `Action::PlaceShaped` or `Action::Rotate`): cycling placement shape (`Shift+R` / `Shift+Wheel`), resolving
//! top/bottom/inverted/stacking from the aimed surface, drawing the live see-through ghost preview (including
//! auto-connected stair and roof corners before clicking), exact sub-voxel aim hit boxes, and the HUD card.

use crate::block::{self, BlockId};
use crate::factory;
use crate::hitbox::Boxes;
use crate::math::{IVec3, Vec3};
use crate::raycast::RayHit;
use crate::shape::{self, Kind};
use crate::sound;
use crate::Game;

impl Game {
    /// The shapeable building block currently held in the selected hotbar slot, if any.
    pub(crate) fn held_shapeable(&self) -> Option<BlockId> {
        let stack = self.inventory().selected_stack();
        stack.item.places().filter(|&b| !stack.is_empty() && shape::is_shapeable(b))
    }

    /// Steps the held shape placement mode (`0..shape::MODE_COUNT`), returning true when holding a shapeable block.
    pub(crate) fn cycle_shape_mode(&mut self, delta: i32) -> bool {
        if self.held_shapeable().is_none() {
            return false;
        }
        let n = i32::from(shape::MODE_COUNT);
        self.held_shape_mode = (i32::from(self.held_shape_mode) + delta).rem_euclid(n) as u8;
        self.play(sound::PLACE, block::sound::STONE, self.body().eye(), 0.3);
        true
    }

    /// Turns a held directional shape (stairs, slope, vertical slab) by 90° clockwise before placing.
    pub(crate) fn turn_held_shape(&mut self) -> bool {
        let directional = matches!(
            self.held_shape_mode,
            shape::MODE_STAIR | shape::MODE_SLOPE | shape::MODE_VSLAB_HALF | shape::MODE_VSLAB_QUARTER
        );
        if !directional || self.held_shapeable().is_none() {
            return false;
        }
        self.turn_placement();
        self.play(sound::PLACE, block::sound::STONE, self.body().eye(), 0.35);
        true
    }

    /// Sub-voxel aim hit boxes for a placed shaped block at `p` (`None` for full cubes and air).
    pub(crate) fn shape_hit_boxes(&self, p: IVec3) -> Option<Boxes> {
        let s = self.sim.world.get_shape(p);
        if s == shape::FULL {
            return None;
        }
        shape::hit_boxes(s, self.sim.world.cell_corner(p, s))
    }

    /// Exact 3D surface point where the crosshair ray hits `hit` (accounting for sub-voxel hit boxes).
    pub(crate) fn hit_surface(&self, hit: RayHit) -> Vec3 {
        let (eye, dir) = (self.body().eye(), self.body().look_dir());
        let origin = if self.strategy.active() { self.strategy.eye() } else { self.third_person.aim_eye(eye, dir) };
        let dir = if self.strategy.active() { self.strategy.cursor_ray() } else { dir };
        let slim = self.sim.factory.aim_boxes(hit.block).or_else(|| self.shape_hit_boxes(hit.block));
        let min = hit.block.as_vec3();
        let mut enter = 0.0_f64;
        for axis in 0..3 {
            let d = dir.get(axis);
            if d.abs() > 1e-9 {
                let a = (min.get(axis) - origin.get(axis)) / d;
                let b = (min.get(axis) + 1.0 - origin.get(axis)) / d;
                enter = enter.max(a.min(b));
            }
        }
        if let Some(t) = slim.and_then(|b| b.enter(origin - min, dir)) {
            enter = enter.max(t);
        }
        origin + dir * enter
    }

    /// Whether placing `s` at `pos` would overlap the local player's body AABB.
    pub(crate) fn shape_hits_body(&self, pos: IVec3, s: u8) -> bool {
        let body = self.body().aabb();
        let corner = shape::corner_of(s, pos, |q| self.sim.world.get_shape(q));
        shape::boxes_for(pos, s, corner).slice().iter().any(|b| b.intersects(&body))
    }

    /// Resolves where and with what shape byte the held shapeable block would be placed right now:
    /// `(block, target_cell, shape_byte, blocked_by_body)`. Returns `None` in Cube mode or when no valid placement.
    pub(crate) fn shape_placement(&self) -> Option<(BlockId, IVec3, u8, bool)> {
        if self.held_shape_mode == shape::MODE_CUBE || self.ghost_mode {
            return None;
        }
        let block = self.held_shapeable()?;
        let hit = self.target.filter(|h| h.normal != IVec3::ZERO)?;
        // In-place slab stacking: clicking the open inner face of a same-material slab grows it (¼ -> ½ -> ¾ -> Full).
        if hit.id == block {
            let existing = self.sim.world.get_shape(hit.block);
            if let Some(grown) = shape::stack_slab(existing, self.held_shape_mode, hit.normal) {
                let blocked = self.shape_hits_body(hit.block, grown);
                return Some((block, hit.block, grown, blocked));
            }
        }
        let pos = hit.block + hit.normal;
        if !self.sim.world.get_block(pos).is_some_and(block::replaceable) {
            return None;
        }
        let upper = if hit.normal.y < 0 {
            true
        } else if hit.normal.y > 0 {
            false
        } else {
            let surf = self.hit_surface(hit);
            (surf.y - hit.block.y as f64) >= 0.5
        };
        let facing = (factory::dir_from_yaw(self.body().yaw) + self.place_turn) & 3;
        let s = shape::from_mode(self.held_shape_mode, facing, upper);
        let blocked = self.shape_hits_body(pos, s);
        Some((block, pos, s, blocked))
    }

    /// HUD card while holding a shapeable block in any non-cube shape mode (`api/hud.rs` `line_label`).
    pub(crate) fn shape_label(&self) -> Option<String> {
        if self.held_shape_mode == shape::MODE_CUBE {
            return None;
        }
        let block = self.held_shapeable()?;
        let name = block::def(block).name;
        let mode = shape::mode_name(self.held_shape_mode);
        let directional = matches!(
            self.held_shape_mode,
            shape::MODE_STAIR | shape::MODE_SLOPE | shape::MODE_VSLAB_HALF | shape::MODE_VSLAB_QUARTER
        );
        let keys = if directional {
            factory::hud_keys(&[("Shift+R", "Shape"), ("R", "Turn"), ("RMB", "Place")])
        } else {
            factory::hud_keys(&[("Shift+R", "Shape"), ("RMB", "Place")])
        };
        let hint = match self.shape_placement() {
            Some((_, _, s, true)) => format!("{} · Blocked (step back)", shape::shape_label(s)),
            Some((_, _, shape::FULL, false)) => "Stack into Full Cube".to_string(),
            Some((_, _, s, false)) => {
                let lbl = shape::shape_label(s);
                if matches!(shape::kind(s), Kind::Stair { .. } | Kind::Slope { .. }) {
                    format!("{lbl} · Corners connect automatically")
                } else {
                    lbl.to_string()
                }
            }
            None => mode.to_string(),
        };
        Some(format!("{name} · {mode}\n{hint}\n{keys}"))
    }
}
