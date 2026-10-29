//! Where an item's icon sits in its box: the screen bounds of its model as the icon draws it, so every icon, a thin
//! tool as much as a block, is scaled to fill its box (`web/src/ui/hud.ts` `itemIcon` uses `scale` and `centre`).
//! The projection is the icon's: x runs right-down, z left-down, y up; a full cube spans 1 across and 1 down,
//! centred on 0, and keeps the size it always had (`ICON_FILL`). Invariant: `span` is positive for every item.
//! A round body's outline is the hull of its two rims (`outline_points`), as the icon paints it.
//! A plant-drawn block (torch, saplings) is a picture with see-through edges, so its box says nothing of its size:
//! it is `flat`, and the icon draws its picture trimmed to its visible pixels instead (`web/src/ui/icon-sprite.ts`).

use super::{parts, Axis, Form, Part};
use crate::item::{self, ItemId};

/// The share of the icon box an icon's longer side fills; the rest is an even margin.
pub const ICON_FILL: f32 = 0.86;

/// An icon's screen bounds in cube units: the longer of its width and height, and its centre (right, down).
#[derive(Clone, Copy, Debug)]
pub struct IconFrame {
    pub span: f32,
    pub centre: [f32; 2],
    /// Drawn as its side picture, not as boxes.
    pub flat: bool,
}

impl IconFrame {
    /// One cube unit as a share of the icon box's side.
    pub fn scale(&self) -> f32 {
        ICON_FILL / self.span
    }
}

/// The frame of an item's icon, from the boxes it draws (`icon_boxes`). A flat one spans 1, so its `scale` is the fill.
pub fn icon_frame(item: ItemId) -> IconFrame {
    if item.0 < 256 && crate::block::PLANT[usize::from(item.0)] && parts(item).is_empty() {
        return IconFrame { span: 1.0, centre: [0.0; 2], flat: true };
    }
    let mut lo = [f32::MAX; 2];
    let mut hi = [f32::MIN; 2];
    for p in &icon_boxes(item) {
        for [x, y, z] in outline_points(p) {
            let at = project(x, y, z);
            for k in 0..2 {
                lo[k] = lo[k].min(at[k]);
                hi[k] = hi[k].max(at[k]);
            }
        }
    }
    let span = (hi[0] - lo[0]).max(hi[1] - lo[1]).max(1e-3);
    IconFrame { span, centre: [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5], flat: false }
}

/// The boxes an item's icon draws: its model's parts, or for a block without one its single box of the def's size.
pub fn icon_boxes(item: ItemId) -> Vec<Part> {
    let model = parts(item);
    if !model.is_empty() {
        return model.to_vec();
    }
    let size = item::def(item).map_or([1.0; 3], |d| d.size);
    vec![Part { center: [0.0; 3], size, yaw: 0.0, tex: [0; 3], form: Form::Box }]
}

/// Points sampled round each rim of a round body (as `RIM_POINTS` in `web/src/ui/icon-geometry.ts`).
pub const RIM_POINTS: usize = 48;

/// The points whose hull is a part's outline on the icon: a box's eight corners, or the rims of both ends of a round
/// body (its size the box it fills before its yaw, its far end `taper` times as wide; `icon-geometry.ts` paints it).
pub fn outline_points(p: &Part) -> Vec<[f32; 3]> {
    let (s, c) = p.yaw.sin_cos();
    let turn = |l: [f32; 3]| [p.center[0] + c * l[0] - s * l[2], p.center[1] + l[1], p.center[2] + s * l[0] + c * l[2]];
    let [sx, sy, sz] = p.size;
    let Form::Round(axis, taper) = p.form else {
        let pitch = if let Form::Lean(pitch) = p.form { pitch } else { 0.0 };
        let (sp, cp) = pitch.sin_cos();
        return (0..8)
            .map(|corner| {
                let sign = |bit: u32| if corner & bit == 0 { -0.5 } else { 0.5 };
                let [x, y, z] = [sign(1) * sx, sign(2) * sy, sign(4) * sz];
                turn([x, y * cp - z * sp, y * sp + z * cp])
            })
            .collect();
    };
    // Along the axis, then the two directions across it, with their radii, in the part's own frame.
    let (along, u, v) = match axis {
        Axis::Up => ([0.0, sy / 2.0, 0.0], [sx / 2.0, 0.0, 0.0], [0.0, 0.0, sz / 2.0]),
        Axis::X => ([sx / 2.0, 0.0, 0.0], [0.0, 0.0, sz / 2.0], [0.0, sy / 2.0, 0.0]),
        Axis::Z => ([0.0, 0.0, sz / 2.0], [sx / 2.0, 0.0, 0.0], [0.0, sy / 2.0, 0.0]),
    };
    let mut out = Vec::with_capacity(2 * RIM_POINTS);
    for (end, k) in [(-1.0, 1.0), (1.0, taper)] {
        for i in 0..RIM_POINTS {
            let (sa, ca) = (std::f32::consts::TAU * i as f32 / RIM_POINTS as f32).sin_cos();
            out.push(turn([0, 1, 2].map(|d| end * along[d] + k * (ca * u[d] + sa * v[d]))));
        }
    }
    out
}

/// A point on the icon, in cube units right and down from the centre of a full cube's icon.
pub fn project(x: f32, y: f32, z: f32) -> [f32; 2] {
    [(x - z) * 0.5, (x + z) * 0.25 - y * 0.5]
}

#[cfg(test)]
mod tests;
