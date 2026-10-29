use super::*;
use crate::item::ITEM_COUNT;

const GRID: usize = 128;

/// The icon as `hud.ts` draws it, fitted by its frame, on a `GRID`² grid over the icon box: every face of every box
/// as a parallelogram (their union is what the visible faces cover).
fn fitted_mask(item: ItemId) -> Vec<bool> {
    let frame = icon_frame(item);
    let boxes = icon_boxes(item);
    let to_box = |x: f32, y: f32, z: f32| {
        let p = project(x, y, z);
        [0.5 + (p[0] - frame.centre[0]) * frame.scale(), 0.5 + (p[1] - frame.centre[1]) * frame.scale()]
    };
    let mut faces = Vec::new();
    let mut hulls = Vec::new();
    for p in &boxes {
        if p.form != Form::Box {
            hulls.push(hull(outline_points(p).iter().map(|&[x, y, z]| to_box(x, y, z)).collect()));
            continue;
        }
        let (s, c) = p.yaw.sin_cos();
        let at = |lx: f32, ly: f32, lz: f32| {
            let (lx, ly, lz) = (lx * p.size[0], ly * p.size[1], lz * p.size[2]);
            to_box(p.center[0] + c * lx - s * lz, p.center[1] + ly, p.center[2] + s * lx + c * lz)
        };
        // Each face: a corner and the two corners beside it.
        for side in [-0.5, 0.5] {
            faces.push([at(-0.5, side, -0.5), at(0.5, side, -0.5), at(-0.5, side, 0.5)]);
            faces.push([at(side, -0.5, -0.5), at(side, 0.5, -0.5), at(side, -0.5, 0.5)]);
            faces.push([at(-0.5, -0.5, side), at(0.5, -0.5, side), at(-0.5, 0.5, side)]);
        }
    }
    let mut mask = vec![false; GRID * GRID];
    for (i, cell) in mask.iter_mut().enumerate() {
        let q = [((i % GRID) as f32 + 0.5) / GRID as f32, ((i / GRID) as f32 + 0.5) / GRID as f32];
        *cell = faces.iter().any(|&[o, a, b]| {
            let (u, v, d) = ([a[0] - o[0], a[1] - o[1]], [b[0] - o[0], b[1] - o[1]], [q[0] - o[0], q[1] - o[1]]);
            let det = u[0] * v[1] - u[1] * v[0];
            let (s, t) = ((d[0] * v[1] - d[1] * v[0]) / det, (u[0] * d[1] - u[1] * d[0]) / det);
            det.abs() > 1e-7 && (0.0..=1.0).contains(&s) && (0.0..=1.0).contains(&t)
        }) || hulls.iter().any(|h| inside(h, q));
    }
    mask
}

/// The convex hull of `pts`, counter-clockwise with y down (Andrew's monotone chain, as `icon-geometry.ts`).
fn hull(mut pts: Vec<[f32; 2]>) -> Vec<[f32; 2]> {
    pts.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    let cross = |o: [f32; 2], a: [f32; 2], b: [f32; 2]| (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
    let half = |list: &mut dyn Iterator<Item = [f32; 2]>| {
        let mut h: Vec<[f32; 2]> = Vec::new();
        for q in list {
            while h.len() >= 2 && cross(h[h.len() - 2], h[h.len() - 1], q) <= 0.0 {
                h.pop();
            }
            h.push(q);
        }
        h.pop();
        h
    };
    let mut out = half(&mut pts.iter().copied());
    out.extend(half(&mut pts.iter().rev().copied()));
    out
}

/// Whether `q` is inside the convex polygon `h` (wound as `hull` winds it).
fn inside(h: &[[f32; 2]], q: [f32; 2]) -> bool {
    (0..h.len()).all(|i| {
        let (a, b) = (h[i], h[(i + 1) % h.len()]);
        (b[0] - a[0]) * (q[1] - a[1]) - (b[1] - a[1]) * (q[0] - a[0]) >= 0.0
    })
}

/// The share of the box side the mask's bounds cover across and down, and whether it touches the box's edge.
fn extent(mask: &[bool]) -> (f32, f32, bool) {
    let (mut lo, mut hi) = ([GRID; 2], [0; 2]);
    for (i, _) in mask.iter().enumerate().filter(|(_, &m)| m) {
        let at = [i % GRID, i / GRID];
        for k in 0..2 {
            lo[k] = lo[k].min(at[k]);
            hi[k] = hi[k].max(at[k]);
        }
    }
    let share = |k: usize| (hi[k] + 1 - lo[k]) as f32 / GRID as f32;
    (share(0), share(1), lo[0] == 0 || lo[1] == 0 || hi[0] == GRID - 1 || hi[1] == GRID - 1)
}

#[test]
fn every_icon_fills_its_box_with_a_margin() {
    let mut worst = (f32::MAX, "");
    for id in 1..ITEM_COUNT as u16 {
        let Some(def) = item::def(ItemId(id)) else { continue };
        let (across, down, edge) = extent(&fitted_mask(ItemId(id)));
        let long = across.max(down);
        if long < worst.0 {
            worst = (long, def.name);
        }
        assert!(long >= 0.8, "{} fills {long:.2} of its icon box", def.name);
        assert!(!edge, "{} touches the edge of its icon box", def.name);
    }
    eprintln!("smallest icon: {} at {:.2}", worst.1, worst.0);
}

#[test]
fn a_full_block_keeps_its_size_and_sits_in_the_middle() {
    let frame = icon_frame(ItemId::block(crate::block::STONE));
    assert!((frame.span - 1.0).abs() < 1e-5 && frame.centre[0].abs() < 1e-5 && frame.centre[1].abs() < 1e-5);
    assert!((frame.scale() - ICON_FILL).abs() < 1e-5);
}

#[test]
fn plant_drawn_blocks_are_flat_pictures_and_nothing_else_is() {
    use crate::block::{SAPLING, STONE, TORCH};
    for plant in [TORCH, SAPLING] {
        let frame = icon_frame(ItemId::block(plant));
        assert!(frame.flat && (frame.scale() - ICON_FILL).abs() < 1e-5, "a flat icon's scale is the fill");
    }
    assert!(!icon_frame(ItemId::block(STONE)).flat && !icon_frame(crate::item::STONE_PICKAXE).flat);
}

#[test]
fn a_tool_is_drawn_as_large_as_a_block() {
    for tool in [crate::item::STONE_PICKAXE, crate::item::IRON_AXE, crate::item::STEEL_SHOVEL] {
        let (across, down, _) = extent(&fitted_mask(tool));
        assert!(across.max(down) >= 0.84, "{tool:?} spans {:.2}", across.max(down));
    }
}
