//! Logs as branches: the logs of a tree are drawn as one skeleton of round, tapering wood in the shaped list
//! (`shaped.rs`), not a cylinder per block. Each log is a node (`node.rs`: a single log, or a square of upright logs
//! drawn as one thick trunk) that joins its neighbours, across a corner too, so a branch laid as a staircase of blocks
//! reads as one slanting limb. A node draws its half of each join, from its middle to the join's middle; a thin
//! branch running into a much thicker node draws all of it, into that node's middle, out of sight.
//! Ends: a trunk on the ground flares at its foot; a free living limb's end tapers to a point, whichever way it points
//! (up, hanging down, out); a full-thickness free end is cut flat (end grain), as a stump is.
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
/// A limb's free end tapers to a point this much longer than half its radius.
const TIP: f32 = 0.32;
/// A join steeper than this (the sine of its slope) counts as upright: a trunk's, with a foot and a flat top.
const UPRIGHT: f32 = 0.8;
/// A run through a node bending less than this (the cosine between its two ways) is drawn as one straight stretch.
const STRAIGHT: f32 = -0.94;

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
        // Vertical trunk steps (e.g. a 2×2 redwood narrowing to 1×1) always taper across their midpoint, never skipped.
        let trunk_step = n.trunk && o.trunk && d[1].abs() > UPRIGHT;
        if !trunk_step && o.radius * THIN < n.radius {
            continue; // the thin side draws all the way into our centre
        }
        let seg = if !trunk_step && n.radius * THIN < o.radius {
            // Subtle branch collar where a living limb enters a much thicker trunk.
            let r1 = if n.leafy { (n.radius * 1.28).min(o.radius * 0.88) } else { n.radius };
            Seg { to: o.centre, r0: n.radius, r1, sides, tiles }
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
    // Ground root flare, or true free upright ends (never draw a bottom cut cap on an interior branch elbow!).
    if up && !down {
        let ground = OPAQUE[below as usize] && !is_log(below);
        if ground {
            let rings = [(bottom, r * FLARE), (bottom + FLARE_HEIGHT, r), (n.centre[1], r)];
            upright(&f, n, &rings, own_sides, own_tiles, out);
        } else if near.len() == 1 {
            if n.leafy {
                f.strip(n.centre, r, at_y(n.centre[1] - TIP - r * 0.5), 0.02, own_sides, own_tiles, out);
            } else {
                upright(&f, n, &[(bottom, r), (n.centre[1], r)], own_sides, own_tiles, out);
                if !HIDES[below as usize] {
                    f.cap(at_y(bottom), [0.0, -1.0, 0.0], r, own_sides, out);
                }
            }
        }
    }
    if down && !up && (near.len() == 1 || segs.len() == 1) {
        if n.leafy {
            f.strip(n.centre, r, at_y(n.centre[1] + TIP + r * 0.5), 0.02, own_sides, own_tiles, out);
        } else if near.len() == 1 {
            upright(&f, n, &[(n.centre[1], r), (top, r)], own_sides, own_tiles, out);
            if !HIDES[above as usize] {
                f.cap(at_y(top), [0.0, 1.0, 0.0], r, own_sides, out);
            }
        }
    }
    // Draw the node's segments cleanly: straight/mitered through a 2-way run, or main stem + embedded branches at forks.
    match &segs[..] {
        [] => {}
        [s] => {
            f.strip(n.centre, s.r0, s.to, s.r1, s.sides, s.tiles, out);
            // A free end of a slanting or level branch: living limbs always taper cleanly, bare logs are cut flat.
            if near.len() == 1 {
                let d = unit(sub(s.to, n.centre));
                if d[1].abs() <= UPRIGHT {
                    if n.leafy {
                        let tip = std::array::from_fn(|k| n.centre[k] - d[k] * (TIP + r * 0.5));
                        f.strip(n.centre, r, tip, 0.02, s.sides, s.tiles, out);
                    } else {
                        let end = std::array::from_fn(|k| n.centre[k] - d[k] * 0.5);
                        f.strip(n.centre, r, end, r, s.sides, s.tiles, out);
                        f.cap(end, d.map(|v| -v), r, s.sides, out);
                    }
                }
            }
        }
        [a, b] => draw_pair(&f, n, a, b, out),
        many => {
            let mut best = (0usize, 1usize, 1.0f32);
            for (i, si) in many.iter().enumerate() {
                let di = unit(sub(si.to, n.centre));
                for (j, sj) in many.iter().enumerate().skip(i + 1) {
                    let c = dot(di, unit(sub(sj.to, n.centre)));
                    if c < best.2 {
                        best = (i, j, c);
                    }
                }
            }
            if best.2 < -0.5 {
                draw_pair(&f, n, &many[best.0], &many[best.1], out);
                for (k, s) in many.iter().enumerate() {
                    if k != best.0 && k != best.1 {
                        f.strip(n.centre, s.r0, s.to, s.r1, s.sides, s.tiles, out);
                    }
                }
            } else {
                let back = r * 0.15;
                for s in many {
                    let d = unit(sub(s.to, n.centre));
                    let from = std::array::from_fn(|k| n.centre[k] - d[k] * back);
                    f.strip(from, s.r0, s.to, s.r1, s.sides, s.tiles, out);
                }
            }
        }
    }
}

/// Draws two segments meeting at `n.centre`: one direct strip when nearly straight, else two mitered strips flush on
/// the bisecting plane so curved branches have no protruding cylinder butts or open elbow gaps.
fn draw_pair(f: &Ring, n: &Node, a: &Seg, b: &Seg, out: &mut Shaped) {
    let (da, db) = (unit(sub(a.to, n.centre)), unit(sub(b.to, n.centre)));
    let cos = dot(da, db);
    if cos < STRAIGHT && a.sides == b.sides && a.tiles == b.tiles {
        f.strip(a.to, a.r1, b.to, b.r1, a.sides, a.tiles, out);
    } else if cos < 0.25 {
        let miter = unit(sub(db, da));
        f.strip_miter(a.to, a.r1, n.centre, a.r0, miter, a.sides, a.tiles, out);
        f.strip_miter(b.to, b.r1, n.centre, b.r0, miter, b.sides, b.tiles, out);
    } else {
        let back = n.radius * 0.15;
        for (s, d) in [(a, da), (b, db)] {
            let from = std::array::from_fn(|k| n.centre[k] - d[k] * back);
            f.strip(from, s.r0, s.to, s.r1, s.sides, s.tiles, out);
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

/// The node's light: the mean of the open cells beside its corner log (a square's corner has two open sides), or,
/// with wood all round (a whorl of branches), its own cell's: a grown tree's wood lets light through (`light.rs`).
fn node_light(pad: &[crate::block::BlockId], light: &[u8], c: [i32; 3]) -> u8 {
    let p = pidx(c[0] as usize + 1, c[1] as usize + 1, c[2] as usize + 1);
    let open = STRIDE.iter().flat_map(|&s| [p as isize + s, p as isize - s]).map(|i| i as usize);
    super::shaped::mean_light(open.filter(|&i| !OPAQUE[pad[i] as usize]).map(|i| light[i])).unwrap_or(light[p])
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
