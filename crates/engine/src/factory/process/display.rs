//! The product display: every processor that makes items shows what it makes on top of its model (`Look::Product`, a
//! square slot standing on a roof, seen from every side; tested in `model/tests.rs`). A light plate (`tex::SIGN`) lies
//! at the slot's floor and the product's own model (`item_models`, as on a belt) stands on it, scaled to fit the slot
//! as it turns; a flat item (a plate, a tool) stands on its edge. Plate and product turn slowly while the machine works
//! and stop where they are when it doesn't. The product is the chosen recipe's (`Pick::Chosen`), else the last batch's
//! (`Processor::shown`), so a machine that picks by input keeps showing it between batches; with none the plate turns
//! empty. Drawn within `NEAR` only. Presentation only: the turn lives in a `Cell` and is neither saved nor hashed.

use std::f32::consts::{FRAC_PI_2, TAU};

use crate::block::tex;
use crate::item::{self, ItemId};
use crate::item_models::{self, Form, Part};
use crate::math::Vec3;
use crate::recipes::MACHINE_RECIPES;

use super::super::render::{push_box, push_round, push_shape, Axis};
use super::{Pick, Processor, Status};

#[cfg(test)]
mod tests;

/// How fast the display turns while its machine works, in radians a second (a turn in about ten seconds).
const SPIN: f32 = 0.6;
/// The longest step one frame may turn it, in seconds (a machine back in sight doesn't jump).
const MAX_STEP: f64 = 0.1;
/// The plate's thickness, its side as a share of the slot's (its corners stay inside the slot as it turns), and the
/// share of the slot the product fills.
const PLATE: f32 = 0.04;
const PLATE_SIDE: f32 = 0.7;
const FILL: f32 = 0.9;
/// An item whose height is under this share of its narrower side lies flat and is stood on its edge.
const FLAT: f32 = 0.5;
/// The display is drawn within this many blocks of the camera: a small display is 5 px across there (1080p).
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

    /// The display's turn at `time` (seconds): it moves on only while the machine works.
    fn display_turn(&self, time: f64) -> f32 {
        let (last, mut turn) = self.display_turn.get();
        if self.status == Status::Working {
            turn = (turn + (time - last).clamp(0.0, MAX_STEP) as f32 * SPIN) % TAU;
        }
        self.display_turn.set((time, turn));
        turn
    }
}

/// Draws `p`'s display in the slot centred at `at` (relative to the camera) of `size` (in the machine's frame), the
/// machine turned by `yaw`, at `time` (seconds).
pub(super) fn draw(p: &Processor, out: &mut Vec<f32>, at: Vec3, yaw: f32, size: [f32; 3], time: f64) {
    if at.x * at.x + at.y * at.y + at.z * at.z > NEAR * NEAR {
        return;
    }
    let turn = yaw + p.display_turn(time);
    let (s, c) = (turn.sin() as f64, turn.cos() as f64);
    let place =
        |v: [f32; 3]| at + Vec3::new(c * v[0] as f64 - s * v[2] as f64, v[1] as f64, s * v[0] as f64 + c * v[2] as f64);
    let floor = -size[1] / 2.0;
    let plate = [size[0] * PLATE_SIDE, PLATE, size[2] * PLATE_SIDE];
    push_box(out, place([0.0, floor + PLATE / 2.0, 0.0]), turn, plate, 0.0, [tex::SIGN, tex::FRAME, tex::FRAME], false);
    let Some(item) = p.product() else { return };
    let plain;
    let parts = match item_models::parts(item) {
        [] => {
            let Some(def) = item::def(item) else { return };
            plain = [Part { center: [0.0; 3], size: def.size, yaw: 0.0, tex: def.tex, form: Form::Box }];
            &plain[..]
        }
        parts => parts,
    };
    let stand = lies_flat(parts);
    let (lo, hi) = bounds(parts, stand);
    let radius = (hi[0] - lo[0]).hypot(hi[2] - lo[2]) / 2.0;
    let k = (size[0] * FILL / 2.0 / radius.max(1e-3)).min((size[1] - PLATE) * FILL / (hi[1] - lo[1]).max(1e-3));
    let mid = [(lo[0] + hi[0]) / 2.0, (lo[2] + hi[2]) / 2.0];
    for part in parts {
        let t = if stand { up(part.center) } else { part.center };
        let v = [(t[0] - mid[0]) * k, floor + PLATE + (t[1] - lo[1]) * k, (t[2] - mid[1]) * k];
        let size = part.size.map(|x| x * k);
        match (stand, part.form) {
            (false, form) => push_shape(out, form, place(v), turn + part.yaw, size, part.tex, false),
            (true, Form::Box | Form::Lean(_)) => {
                push_box(out, place(v), turn + part.yaw, size, 0.0, part.tex, false);
                let n = out.len();
                out[n - 4] = FRAC_PI_2; // the box's pitch (`render.rs`): stood up about its own centre, as `up` does
            }
            (true, Form::Round(axis, t)) => {
                // Stood up as `up` turns it: +y goes to +z, +z to -y, x stays; the box it fills turns with it.
                let (axis, t, across) = match axis {
                    Axis::Up => (Axis::Z, t, 1.0),
                    Axis::Z => (Axis::Up, 1.0 / t, t),
                    Axis::X => (Axis::X, t, 1.0),
                };
                let stood = match axis {
                    Axis::Up => [size[0] * across, size[2], size[1] * across],
                    _ => [size[0], size[2], size[1]],
                };
                push_round(out, place(v), turn + part.yaw, axis, stood, t, part.tex, false);
            }
        }
    }
}

/// Whether an item lies flat (its height well under its narrower side), measured untipped.
fn lies_flat(parts: &[Part]) -> bool {
    let (lo, hi) = bounds(parts, false);
    let d = [0, 1, 2].map(|a| hi[a] - lo[a]);
    d[1] < FLAT * d[0].min(d[2])
}

/// A point of the item stood on its edge: turned a quarter about x, as the box shader turns a box by a pitch of a
/// quarter turn (`boxVert`).
fn up(v: [f32; 3]) -> [f32; 3] {
    [v[0], -v[2], v[1]]
}

/// The bounds of `parts`, stood up or not (each part's own turn left out: few items have one, and it changes the
/// bounds little).
fn bounds(parts: &[Part], stand: bool) -> ([f32; 3], [f32; 3]) {
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in parts {
        let h = p.size.map(|s| s / 2.0);
        let (t, reach) = if stand { (up(p.center), [h[0], h[2], h[1]]) } else { (p.center, h) };
        for a in 0..3 {
            lo[a] = lo[a].min(t[a] - reach[a]);
            hi[a] = hi[a].max(t[a] + reach[a]);
        }
    }
    (lo, hi)
}
