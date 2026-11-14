//! Logs as branches: the logs of a tree are drawn as one skeleton of round, tapering wood in the shaped list
//! (`shaped.rs`), not a cylinder per block. Each log is a node (`node.rs`: a single log, or a square of upright logs
//! drawn as one thick trunk) that joins its neighbours, across a corner too, so a branch laid as a staircase of blocks
//! reads as one slanting limb. A node draws its half of each join, from its middle to the join's middle; a thin
//! branch running into a much thicker node draws all of it, into that node's middle, out of sight.
//! Ends: a trunk on the ground flares at its foot; a free upright end is cut flat (end grain), as a stump is; a free
//! living limb's end tapers to a point; a full-thickness free end is cut flat.
//! Invariants: logs hide no neighbour's faces (`HIDES` in the mesher), so the ground shows round a trunk; light and
//! shade are still those of an opaque block. Two chunks drawing the halves of one join compute its middle ring from
//! the same blocks, so they meet. To give a log block its own thickness: a row in `node::radii`.

use super::shaped::{Shaped, HIDES};
use super::tree_grid::TreeGrid;
use super::{pidx, N, STRIDE};
use crate::block::{is_leaves, is_log, FACE_TEX, OPAQUE};

mod node;
mod ring;
use node::{joins, Node, Nodes};
use ring::Ring;

/// How much wider than the trunk its foot is, and how high the flare reaches.
const FLARE: f32 = 1.35;
const FLARE_HEIGHT: f32 = 0.3;
/// A join where one side is this much thicker than the other is drawn by the thin side alone.
const THIN: f32 = 1.5;
/// A limb's free end tapers to a point this much longer than it is thick.
const TIP: f32 = 0.4;
/// A join steeper than this (the sine of its slope) counts as upright: a trunk's, with a foot and a flat top.
const UPRIGHT: f32 = 0.8;
/// A run through a node bending less than this (the cosine between its two ways) is drawn as one stretch.
const STRAIGHT: f32 = -0.9;

/// Every log node owned by the chunk (by its lowest corner) as wood, into `out`'s opaque list.
pub fn mesh_logs(g: &TreeGrid, pad: &[crate::block::BlockId], light: &[u8], out: &mut Shaped) {
    let (mut near, mut nodes) = (Vec::new(), Nodes::default());
    for y in 0..N as i32 {
        for z in 0..N as i32 {
            for x in 0..N as i32 {
                if !is_log(g.at(x, y, z)) {
                    continue;
                }
                let n = nodes.of(g, [x, y, z]);
                if n.corner == [x, y, z] {
                    joins(g, &n, &mut nodes, &mut near);
                    draw(g, pad, light, &n, &near, out);
                }
            }
        }
    }
}

/// One stretch of wood from a node: from its middle to `to`, its radius going from `r0` to `r1`.
struct Seg {
    to: [f32; 3],
    r0: f32,
    r1: f32,
    sides: usize,
    tiles: f32,
}

#[inline(never)]
fn draw(g: &TreeGrid, pad: &[crate::block::BlockId], light: &[u8], n: &Node, near: &[Node], out: &mut Shaped) {
    let mut segs: Vec<Seg> = Vec::with_capacity(near.len());
    let (mut up, mut down) = (false, false);
    let c = n.corner;
    let walled = n.width == 1
        && STRIDE.iter().all(|&s| {
            let p = pidx(c[0] as usize + 1, c[1] as usize + 1, c[2] as usize + 1) as isize;
            HIDES[pad[(p + s) as usize] as usize] && HIDES[pad[(p - s) as usize] as usize]
        });
    if walled {
        return;
    }
    for o in near {
        let d = unit(sub(o.centre, n.centre));
        up |= d[1] > UPRIGHT;
        down |= d[1] < -UPRIGHT;
        let (sides, tiles) = ring::detail(n.radius.max(o.radius));
        if o.radius * THIN < n.radius {
            continue; // the thin side draws it
        }
        let seg = if n.radius * THIN < o.radius {
            Seg { to: o.centre, r0: n.radius, r1: n.radius, sides, tiles }
        } else {
            let mid = std::array::from_fn(|k| (n.centre[k] + o.centre[k]) * 0.5);
            Seg { to: mid, r0: n.radius, r1: (n.radius + o.radius) * 0.5, sides, tiles }
        };
        segs.push(seg);
    }
    let f = Ring {
        cell: c,
        light: node_light(pad, light, c),
        occlusion: 7 - (leaves_round(g, c) / 5).min(4),
        side_layer: FACE_TEX[n.block as usize][0],
        end_layer: FACE_TEX[n.block as usize][2],
    };
    let (own_sides, own_tiles) = ring::detail(n.radius);
    let below = g.at(c[0], c[1] - 1, c[2]);
    let above = g.at(c[0], c[1] + 1, c[2]);
    let (bottom, top) = (n.centre[1] - 0.5, n.centre[1] + 0.5);
    let at_y = |y: f32| [n.centre[0], y, n.centre[2]];
    let r = n.radius;
    // A lone log: a short upright trunk, capped where it shows.
    if near.is_empty() {
        let foot = OPAQUE[below as usize] && !is_log(below);
        let rings: &[(f32, f32)] =
            if foot { &[(bottom, r * FLARE), (bottom + FLARE_HEIGHT, r), (top, r)] } else { &[(bottom, r), (top, r)] };
        upright(&f, n, rings, own_sides, own_tiles, out);
        if !HIDES[below as usize] {
            f.cap(at_y(bottom), [0.0, -1.0, 0.0], r, own_sides, out);
        }
        if !HIDES[above as usize] {
            f.cap(at_y(top), [0.0, 1.0, 0.0], r, own_sides, out);
        }
        return;
    }
    // A trunk's foot on the ground, flared; an upright free end, cut flat.
    if up && !down {
        let ground = OPAQUE[below as usize] && !is_log(below);
        if ground {
            let rings = [(bottom, r * FLARE), (bottom + FLARE_HEIGHT, r), (n.centre[1], r)];
            upright(&f, n, &rings, own_sides, own_tiles, out);
        } else {
            upright(&f, n, &[(bottom, r), (n.centre[1], r)], own_sides, own_tiles, out);
            if !HIDES[below as usize] {
                f.cap(at_y(bottom), [0.0, -1.0, 0.0], r, own_sides, out);
            }
        }
    }
    if down && !up && near.len() == 1 {
        upright(&f, n, &[(n.centre[1], r), (top, r)], own_sides, own_tiles, out);
        if !HIDES[above as usize] {
            f.cap(at_y(top), [0.0, 1.0, 0.0], r, own_sides, out);
        }
    }
    // A straight or gently bending run through the node: one stretch from end to end.
    if let [a, b] = &segs[..] {
        let (da, db) = (unit(sub(a.to, n.centre)), unit(sub(b.to, n.centre)));
        if dot(da, db) < STRAIGHT && a.sides == b.sides && a.tiles == b.tiles {
            f.strip(a.to, a.r1, b.to, b.r1, a.sides, a.tiles, out);
            return;
        }
    }
    let back = if segs.len() >= 2 { r * 0.7 } else { 0.0 };
    for s in &segs {
        let d = unit(sub(s.to, n.centre));
        let from = std::array::from_fn(|k| n.centre[k] - d[k] * back);
        f.strip(from, s.r0, s.to, s.r1, s.sides, s.tiles, out);
    }
    // A free end of a slanting or level branch: a living tree's limb (thinned among leaves) tapers to a point, longer
    // the thicker it is; full-thickness wood (a beam someone built, a dead tree) is cut flat.
    if let [s] = &segs[..] {
        let d = unit(sub(s.to, n.centre));
        if d[1].abs() <= UPRIGHT {
            if r < node::radii(n.block).1 * 0.99 {
                let tip = std::array::from_fn(|k| n.centre[k] - d[k] * (TIP + r));
                f.strip(n.centre, r, tip, 0.02, s.sides, s.tiles, out);
            } else {
                let end = std::array::from_fn(|k| n.centre[k] - d[k] * 0.5);
                f.strip(n.centre, r, end, r, s.sides, s.tiles, out);
                f.cap(end, d.map(|v| -v), r, s.sides, out);
            }
        }
    }
}

/// An upright stretch through the node's middle, ring by ring: (height, radius).
fn upright(f: &Ring, n: &Node, rings: &[(f32, f32)], sides: usize, tiles: f32, out: &mut Shaped) {
    for w in rings.windows(2) {
        let at = |y: f32| [n.centre[0], y, n.centre[2]];
        f.strip(at(w[0].0), w[0].1, at(w[1].0), w[1].1, sides, tiles, out);
    }
}

/// The node's light: the mean of the open cells beside its corner log (a square's corner has two open sides).
fn node_light(pad: &[crate::block::BlockId], light: &[u8], c: [i32; 3]) -> u8 {
    let p = pidx(c[0] as usize + 1, c[1] as usize + 1, c[2] as usize + 1);
    let open = STRIDE.iter().flat_map(|&s| [p as isize + s, p as isize - s]).map(|i| i as usize);
    super::shaped::mean_light(open.filter(|&i| !OPAQUE[pad[i] as usize]).map(|i| light[i])).unwrap_or(0)
}

/// Leaves in the 3×3×3 box round a cell: a branch deep in a crown is shaded.
fn leaves_round(g: &TreeGrid, c: [i32; 3]) -> u32 {
    let mut n = 0;
    for dy in -1..=1 {
        for dz in -1..=1 {
            for dx in -1..=1 {
                n += u32::from(is_leaves(g.at(c[0] + dx, c[1] + dy, c[2] + dz)));
            }
        }
    }
    n
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn unit(a: [f32; 3]) -> [f32; 3] {
    let l = dot(a, a).sqrt().max(1e-6);
    a.map(|v| v / l)
}

#[cfg(test)]
mod tests;
