//! Where an item's icon sits in its box: the screen bounds of its model as the icon draws it, so every icon, a thin
//! tool as much as a block, is scaled to fill its box (`web/src/ui/hud.ts` `itemIcon` uses `scale` and `centre`).
//! The projection is the icon's: x runs right-down, z left-down, y up; a full cube spans 1 across and 1 down,
//! centred on 0, and keeps the size it always had (`ICON_FILL`). Invariant: `span` is positive for every item.
//! A plant-drawn block (torch, saplings) is a picture with see-through edges, so its box says nothing of its size:
//! it is `flat`, and the icon draws its picture trimmed to its visible pixels instead (`web/src/ui/icon-sprite.ts`).

use super::{parts, Part};
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
        let (s, c) = p.yaw.sin_cos();
        for corner in 0..8 {
            let sign = |bit: u32| if corner & bit == 0 { -0.5 } else { 0.5 };
            let (lx, ly, lz) = (sign(1) * p.size[0], sign(2) * p.size[1], sign(4) * p.size[2]);
            let (x, y, z) = (p.center[0] + c * lx - s * lz, p.center[1] + ly, p.center[2] + s * lx + c * lz);
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
    vec![Part { center: [0.0; 3], size, yaw: 0.0, tex: [0; 3] }]
}

/// A point on the icon, in cube units right and down from the centre of a full cube's icon.
pub fn project(x: f32, y: f32, z: f32) -> [f32; 2] {
    [(x - z) * 0.5, (x + z) * 0.25 - y * 0.5]
}

#[cfg(test)]
mod tests;
