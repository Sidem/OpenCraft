//! Version 8's shores (R1c): the outline of a lake or pond, so none is a circle. A shore is a radius bent by three
//! hashed harmonics round the centre (the second stretches it, the third and fifth give it bays and points); the
//! columns' own wobble (`rivers.rs`) roughens it further.
//!
//! Invariants: a pure function of the hash it is made from; `reach` never exceeds 1, so whatever a full radius
//! bounds (a pond's cell margin, the spawn clearing) still bounds the shore. To tune: `AMPS`.

use crate::math::{hash2, unit};

/// Each harmonic's (number round the centre, least and most strength).
const AMPS: [(f64, f64, f64); 3] = [(2.0, 0.14, 0.26), (3.0, 0.06, 0.13), (5.0, 0.03, 0.07)];

#[derive(Clone, Copy, Debug)]
pub(super) struct Shore {
    amp: [f64; 3],
    phase: [f64; 3],
    /// The sum of the strengths, which scales the largest reach to 1.
    total: f64,
}

impl Shore {
    /// The shore drawn from hash `h`.
    pub fn new(h: u32) -> Shore {
        let mut amp = [0.0; 3];
        let mut phase = [0.0; 3];
        for (i, &(_, lo, hi)) in AMPS.iter().enumerate() {
            amp[i] = lo + (hi - lo) * unit(hash2(h, i as i32, 0));
            phase[i] = std::f64::consts::TAU * unit(hash2(h, i as i32, 1));
        }
        Shore { amp, phase, total: amp.iter().sum() }
    }

    /// The share of the full radius (about 0.4 to 1) the shore reaches in direction (dx, dz) from the centre.
    pub fn reach(&self, dx: f64, dz: f64) -> f64 {
        let a = dz.atan2(dx);
        let bend: f64 = (0..3).map(|i| self.amp[i] * (AMPS[i].0 * a + self.phase[i]).cos()).sum();
        (1.0 + bend) / (1.0 + self.total)
    }
}

/// The farthest and nearest the edge of a shape lies from (x, z) over 72 directions; `inside` says what it covers.
#[cfg(test)]
pub(super) fn spread(x: i32, z: i32, inside: impl Fn(i32, i32) -> bool) -> (f64, f64) {
    let edge = |k: i32| {
        let a = k as f64 * std::f64::consts::TAU / 72.0;
        (0..200).find(|&d| !inside(x + (a.cos() * d as f64) as i32, z + (a.sin() * d as f64) as i32)).unwrap_or(200)
            as f64
    };
    (0..72).map(edge).fold((0.0f64, f64::MAX), |(hi, lo), e| (hi.max(e), lo.min(e)))
}
