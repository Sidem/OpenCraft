//! What the camera can see, as a cone round its look direction wide enough to hold the whole screen (its corners
//! included): per-frame instance writers skip what lies outside it, so boxes behind the camera are never built, lit
//! or uploaded. Presentation only, never core state.
//!
//! Invariant: conservative. Anything on screen is inside the cone (`sees` may say yes to a little off screen, never
//! no to anything on it), given the host's field of view (`Game::set_view_cone`) and the same yaw and pitch the host
//! draws with (`Game::camera_yaw` / `camera_pitch`). Until the host sends a field of view, the cone sees everything.

use crate::math::Vec3;

/// The camera's view cone (`Game::update` builds one a frame).
#[derive(Clone, Copy)]
pub struct ViewCone {
    forward: Vec3,
    cos: f64,
    sin: f64,
    all: bool,
}

impl ViewCone {
    /// Sees everything (no field of view known, or one too wide to be worth testing).
    pub const ALL: ViewCone = ViewCone { forward: Vec3::ZERO, cos: 0.0, sin: 1.0, all: true };

    /// The cone of a camera looking along `yaw` / `pitch` (`player::Body::look_dir`'s convention) with vertical field
    /// of view `fov_y` (radians) on a screen `aspect` wide per unit high. `fov_y <= 0` gives [`Self::ALL`].
    pub fn new(yaw: f64, pitch: f64, fov_y: f64, aspect: f64) -> Self {
        if fov_y <= 0.0 || aspect <= 0.0 {
            return Self::ALL;
        }
        // The half-angle to a screen corner: the widest the frustum reaches from its axis.
        let half = ((fov_y * 0.5).tan() * (1.0 + aspect * aspect).sqrt()).atan();
        if half >= 1.5 {
            return Self::ALL;
        }
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let forward = Vec3::new(sy * cp, sp, -cy * cp);
        ViewCone { forward, cos: half.cos(), sin: half.sin(), all: false }
    }

    /// Whether any of the ball of `radius` round camera-relative `rel` can be inside the cone.
    pub fn sees(&self, rel: Vec3, radius: f64) -> bool {
        if self.all {
            return true;
        }
        let len2 = rel.dot(rel);
        if len2 <= radius * radius {
            return true;
        }
        let along = rel.dot(self.forward);
        let across = (len2 - along * along).max(0.0).sqrt();
        // Nearest the apex itself (behind the camera): the ball is clear of it, checked above.
        if along * self.cos + across * self.sin < 0.0 {
            return false;
        }
        across * self.cos - along * self.sin <= radius
    }
}

#[cfg(test)]
mod tests;
