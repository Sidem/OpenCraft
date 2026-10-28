//! The product display: every processor that makes items shows what it makes in one slot of its model
//! (`Look::Product`, placed per model where it is seen from the front and above; tested in `model/tests.rs`). A dark
//! plate stands at the slot's back and the product's own model (`item_models`, as on a belt) in front of it, scaled to
//! fill the slot and tipped back `TILT`, so a flat item (a plate, an ingot) shows its face from the ground and from
//! above. The product is the chosen recipe's (`Pick::Chosen`), else the last batch's (`Processor::shown`), so a machine
//! that picks by input keeps showing it between batches; with none the plate stands empty. The display is drawn within
//! `NEAR` only, and the product only when seen from the front side of the plate. Presentation only.

use crate::block::tex;
use crate::item::{self, ItemId};
use crate::item_models::{self, Part};
use crate::math::Vec3;
use crate::recipes::MACHINE_RECIPES;

use super::super::render::push_box;
use super::{Pick, Processor};

#[cfg(test)]
mod tests;

/// How far the product is tipped back (its top turned towards the front), in radians, and its sine and cosine
/// (`tip` runs for every corner of every product each frame; a test checks they agree).
const TILT: f32 = 0.6;
const TILT_SIN: f32 = 0.564_642_5;
const TILT_COS: f32 = 0.825_335_6;
/// The plate's thickness, and the share of the slot the product fills.
const PLATE: f32 = 0.04;
const FILL: f32 = 0.9;
/// The display is drawn within this many blocks of the camera: a small sign is 5 px across there (1080p).
const NEAR: f64 = 48.0;

impl Processor {
    /// What its display shows: the chosen recipe's product, else the last batch's.
    pub fn product(&self) -> Option<ItemId> {
        let r = if self.spec.pick == Pick::Chosen { self.recipe } else { self.shown };
        self.spec.recipe(r?).map(|r| r.main().0)
    }

    /// What a loaded machine last made, as well as its state tells: the batch in progress, else a recipe of its making
    /// whose product it holds (`shown` itself is not saved).
    pub(super) fn guess_shown(&self) -> Option<u16> {
        if self.batch.is_some() {
            return self.batch;
        }
        let held = self.out.slots.iter().find(|s| !s.is_empty())?.item;
        (0..MACHINE_RECIPES.len() as u16).find(|&i| self.spec.recipe(i).is_some_and(|r| r.main().0 == held))
    }
}

/// Draws `p`'s display in the slot centred at `at` (relative to the camera) of `size` (in the machine's frame), the
/// machine turned by `yaw`.
pub(super) fn draw(p: &Processor, out: &mut Vec<f32>, at: Vec3, yaw: f32, size: [f32; 3]) {
    if at.x * at.x + at.y * at.y + at.z * at.z > NEAR * NEAR {
        return;
    }
    let (s, c) = (yaw.sin() as f64, yaw.cos() as f64);
    let turn =
        |v: [f32; 3]| at + Vec3::new(c * v[0] as f64 - s * v[2] as f64, v[1] as f64, s * v[0] as f64 + c * v[2] as f64);
    let back = turn([0.0, 0.0, (PLATE - size[2]) / 2.0]);
    push_box(out, back, yaw, [size[0], size[1], PLATE], 0.0, [tex::FRAME, tex::SOOT, tex::FRAME], false);
    // Seen from behind, the plate hides the product.
    let front = turn([0.0, 0.0, 1.0]) - at;
    if front.x * back.x + front.z * back.z >= 0.0 {
        return;
    }
    let Some(item) = p.product() else { return };
    let plain;
    let parts = match item_models::parts(item) {
        [] => {
            let Some(def) = item::def(item) else { return };
            plain = [Part { center: [0.0; 3], size: def.size, yaw: 0.0, tex: def.tex }];
            &plain[..]
        }
        parts => parts,
    };
    let (lo, hi) = tipped_bounds(parts);
    let room = [size[0] * FILL, size[1] * FILL, (size[2] - PLATE) * FILL];
    let k = (0..3).map(|a| room[a] / (hi[a] - lo[a]).max(1e-3)).fold(f32::MAX, f32::min);
    let mid = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
    for part in parts {
        let t = tip(part.center);
        let v = [(t[0] - mid[0]) * k, (t[1] - mid[1]) * k, (t[2] - mid[2]) * k + PLATE / 2.0];
        push_box(out, turn(v), yaw + part.yaw, part.size.map(|x| x * k), 0.0, part.tex, false);
        let n = out.len();
        out[n - 4] = TILT; // the box's pitch (`render.rs`), the same tip about its own centre
    }
}

/// A point of the item turned by `TILT` about x, as the box shader turns a box by its pitch (`boxVert`).
fn tip(v: [f32; 3]) -> [f32; 3] {
    let (s, c) = (TILT_SIN, TILT_COS);
    [v[0], c * v[1] - s * v[2], s * v[1] + c * v[2]]
}

/// The bounds of `parts` once tipped (each part's own turn left out: few items have one, and it changes the bounds
/// little).
fn tipped_bounds(parts: &[Part]) -> ([f32; 3], [f32; 3]) {
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in parts {
        let (t, reach) = (tip(p.center), reach(p.size));
        for a in 0..3 {
            lo[a] = lo[a].min(t[a] - reach[a]);
            hi[a] = hi[a].max(t[a] + reach[a]);
        }
    }
    (lo, hi)
}

/// How far a box of `size` reaches from its centre along each axis once tipped.
fn reach(size: [f32; 3]) -> [f32; 3] {
    let h = size.map(|s| s / 2.0);
    [h[0], TILT_COS * h[1] + TILT_SIN * h[2], TILT_SIN * h[1] + TILT_COS * h[2]]
}
