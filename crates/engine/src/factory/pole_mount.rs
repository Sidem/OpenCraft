//! Where a power pole may stand and what it hangs from. A pole needs something solid under it or beside it: on the
//! ground it stands as it always did; with no ground under it but a solid block at one of its four sides it is
//! mounted on that wall (`Pole::wall`, drawn as a plate and two struts reaching the wall); with neither, the core
//! refuses it (`Sim::pole_fits`, `action/blocks.rs`). Pure helpers over a solidity query, so the placing preview
//! (`power_tools.rs`) and the core share one rule. The wall is chosen when the pole is placed and saved in the
//! high bits of the pole's tier byte (`Pole::write_state`), so older saves read as before: no wall.

use crate::block::tex;
use crate::math::{IVec3, Vec3};

use super::render::push_box;

const UP: IVec3 = IVec3::new(0, 1, 0);

/// The four sides a pole can hang on; `Pole::wall` is an index into this plus one (0: no wall).
pub const SIDES: [IVec3; 4] = [IVec3::new(1, 0, 0), IVec3::new(-1, 0, 0), IVec3::new(0, 0, 1), IVec3::new(0, 0, -1)];

/// The `Pole::wall` of a pole put at `pos`: 0 when it stands on something solid or has no wall, else the side with
/// a solid block, preferring the side of `against` (the block clicked) when that is one.
pub fn wall_for(mut solid: impl FnMut(IVec3) -> bool, pos: IVec3, against: IVec3) -> u8 {
    if solid(pos - UP) {
        return 0;
    }
    let walled = SIDES.map(|s| solid(pos + s));
    let clicked = SIDES.iter().position(|&s| pos + s == against).filter(|&i| walled[i]);
    clicked.or_else(|| walled.iter().position(|&w| w)).map_or(0, |i| i as u8 + 1)
}

/// Whether a pole at `pos` has something to stand on or hang from.
pub fn is_held_up(mut solid: impl FnMut(IVec3) -> bool, pos: IVec3) -> bool {
    solid(pos - UP) || SIDES.iter().any(|&s| solid(pos + s))
}

/// The mounting hardware of a pole on wall `wall` (1 to 4): a plate flat on the wall and two struts to the post.
/// `rel` is the cell centre relative to the camera.
pub fn mount_model(out: &mut Vec<f32>, rel: Vec3, wall: u8) {
    let Some(&side) = SIDES.get(wall as usize - 1) else { return };
    let along_x = side.x != 0;
    let dir = Vec3::new(f64::from(side.x), 0.0, f64::from(side.z));
    let frame = [tex::FRAME; 3];
    // Axis order is [x, y, z]: `reach` runs from the post to the wall, `across` is the width along the wall.
    let sized = |reach: f32, y: f32, across: f32| if along_x { [reach, y, across] } else { [across, y, reach] };
    push_box(out, rel + dir * 0.46 + Vec3::new(0.0, 0.4, 0.0), 0.0, sized(0.08, 1.3, 0.24), 0.0, frame, false);
    for y in [-0.12, 0.85] {
        push_box(out, rel + dir * 0.25 + Vec3::new(0.0, y, 0.0), 0.0, sized(0.36, 0.07, 0.07), 0.0, frame, false);
    }
}

#[cfg(test)]
impl super::Factory {
    /// The wall of the ordinary pole anchored at `pos`.
    pub(crate) fn pole_wall(&self, pos: IVec3) -> Option<u8> {
        self.poles.iter().find(|p| p.pos == pos).map(|p| p.wall)
    }
}

#[cfg(test)]
mod tests;
