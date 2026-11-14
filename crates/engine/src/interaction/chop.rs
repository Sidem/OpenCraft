//! The hands at a grown tree (presentation; the core decides what a swing does: `sim/felling.rs`). Held at a tree's
//! trunk near the ground they swing every `SWING_SECONDS`, sending `Action::ChopTree` with where the swing lands and
//! the side the player stands on (the tree falls the other way). The trunk under the crosshair is worked out once
//! per target block (`chop_trunk`), from the loaded blocks; the HUD shows how far the felling has got
//! (`chop_progress`) and where its mark is (`chop_mark`).

use super::aim;
use crate::action::Action;
use crate::block::{self, AIR};
use crate::math::{IVec3, Vec3};
use crate::raycast::RayHit;
use crate::sim::{mark_of, trunk, work_needed, Trunk, LOW, SWING_SECONDS};
use crate::Game;

impl Game {
    /// The trunk of the grown tree the crosshair is on, worked out again when the targeted block changes.
    pub(crate) fn chop_trunk(&mut self) -> Option<&Trunk> {
        let hit = self.target.filter(|h| block::is_tree_wood(h.id))?;
        if self.chop.as_ref().is_none_or(|(at, _)| *at != hit.block) {
            let world = &self.sim.world;
            let found = trunk(|p| world.get_block(p).unwrap_or(AIR), hit.block)?;
            self.chop = Some((hit.block, found));
        }
        self.chop.as_ref().map(|(_, t)| t)
    }

    /// Swings at the tree `hit` while the button is held. A swing up the trunk, out of reach of the cut, does
    /// nothing but sound.
    pub(super) fn update_chopping(&mut self, hit: RayHit, dt: f32) {
        let Some(t) = self.chop_trunk().cloned() else { return };
        self.mine_progress += dt / SWING_SECONDS;
        if self.mine_progress < 1.0 {
            return;
        }
        self.mine_progress = 0.0;
        if hit.block.y >= t.low + LOW {
            self.play(crate::sound::DIG, block::sound::WOOD, hit.block.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 0.3);
            return;
        }
        let (eye, dir) = (self.body().eye(), self.body().look_dir());
        let at = aim::surface(self.third_person.aim_eye(eye, dir), dir, hit, None);
        let feet = self.body().pos;
        let angle = ((feet.z as f32 - t.centre[1]).atan2(feet.x as f32 - t.centre[0])) / std::f32::consts::TAU;
        let from = ((angle * 16.0).round() as i32).rem_euclid(16) as u8;
        self.act(Action::ChopTree { pos: hit.block, at, from });
    }

    /// How far the felling of the tree under the crosshair has got, 0..1 (`None`: no grown tree there).
    pub(crate) fn chop_progress(&mut self) -> Option<f32> {
        let t = self.chop_trunk()?.clone();
        let log = self.target.map_or(AIR, |h| h.id);
        Some((self.sim.felling.work(t.foot) as f32 / work_needed(&t, log).max(1) as f32).min(1.0))
    }

    /// Where the mark on the tree under the crosshair is, once it has been struck at (`None` before).
    pub(crate) fn chop_mark(&mut self) -> Option<Vec3> {
        let t = self.chop_trunk()?.clone();
        let k = self.sim.felling.marks(t.foot)?;
        Some(mark_of(self.sim.world.generator().seed(), &t, k))
    }

    /// Whether the crosshair is on a grown tree too high up its trunk for a swing to count.
    pub(crate) fn chop_too_high(&mut self) -> bool {
        let block = self.target.map_or(IVec3::ZERO, |h| h.block);
        self.chop_trunk().is_some_and(|t| block.y >= t.low + LOW)
    }
}
