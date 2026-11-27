//! Up and gravity (PLANET-UP): the one place that says which way is up at a position and which way things fall,
//! path-open rule 1 (`docs/DECISIONS.md` 2026-11-16, `crates/engine/CLAUDE.md`). A flat world's up is +y everywhere;
//! a round planet (`docs/ROADMAP_SPACE.md`) answers here from the planet's centre, and what reads these follows:
//! bodies (`player.rs`) and loose items (`entities.rs`) fall along it, the companion drone leans against it
//! (`companion_view.rs`), the camera eases the eye along it (`camera.rs`), and the authority looks below a body
//! along it (`authority.rs`). Block neighbours in the grid (`const UP: IVec3`) are the grid's own axis, not this.
//!
//! Invariant: on a flat world `up_at` is exactly +y and `gravity_at` exactly `-strength` on y, so every caller computes
//! the same bits it did with a hard-coded y (no golden hash moves). Each caller keeps its own strength (how floaty it
//! feels); only the direction lives here. To add a world shape: answer it in `up_at`.

use crate::math::Vec3;

/// The unit vector pointing up at `p`.
pub fn up_at(_p: Vec3) -> Vec3 {
    Vec3::new(0.0, 1.0, 0.0)
}

/// The acceleration (blocks per second squared) of something at `p` that falls at `strength`: along down.
pub fn gravity_at(p: Vec3, strength: f64) -> Vec3 {
    up_at(p) * -strength
}

/// How much of `v` points up at `p`.
pub fn along_up(p: Vec3, v: Vec3) -> f64 {
    let u = up_at(p);
    v.x * u.x + v.y * u.y + v.z * u.z
}

#[cfg(test)]
mod tests;
