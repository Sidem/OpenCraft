//! What trees cost to mesh, and how square they still are (ignored; a bench, not a check):
//!   cargo test --release -q bench_forest_mesh -- --ignored --nocapture
//! Loads seed 1337 around its lowland spawn (broadleaf and oak woods), then remeshes every chunk holding a tree block
//! a few times and prints the chunks, their quads by list, the vertex bytes and the time per chunk. Compare before
//! and after a change to how trees are meshed (`mesher/`).
//! Squareness: the share of the canopies' surface (and of the trunks' sides) that faces within 10° of an axis, by
//! area. Leaves and logs drawn as cubes are 100% by construction.

use super::run_until_ready;
use crate::world::MeshData;
use crate::Game;

/// Area of the triangles of `quads` (shaped list, 12 words a quad) whose kind passes `keep`, and of those facing
/// within 10° of an axis.
fn square_share(m: &MeshData, from: usize, to: usize, keep: impl Fn(u32) -> bool) -> (f64, f64) {
    let (mut all, mut square) = (0.0, 0.0);
    for q in m.shaped[from * 12..to * 12].chunks(12) {
        if !keep(q[0] >> 24 & 7) {
            continue;
        }
        let p: [[f64; 3]; 4] = std::array::from_fn(|i| {
            let w = &q[i * 3..i * 3 + 3];
            [w[0] & 4095, w[0] >> 12 & 4095, w[1] & 4095].map(f64::from)
        });
        for [a, b, c] in [[p[0], p[1], p[2]], [p[0], p[2], p[3]]] {
            let (u, v) =
                (std::array::from_fn::<f64, 3, _>(|k| b[k] - a[k]), std::array::from_fn::<f64, 3, _>(|k| c[k] - a[k]));
            let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len == 0.0 {
                continue;
            }
            all += len;
            if n.iter().any(|&c| c.abs() / len > 10f64.to_radians().cos()) {
                square += len;
            }
        }
    }
    (all, square)
}

#[test]
#[ignore]
fn bench_forest_mesh() {
    let mut g = Game::new(1337, 6);
    run_until_ready(&mut g);
    let mut best = f64::MAX;
    let mut last = (0, Vec::new());
    for _ in 0..5 {
        let (chunks, meshes, secs) = g.sim.world.remesh_trees();
        best = best.min(secs / f64::from(chunks.max(1)));
        last = (chunks, meshes);
    }
    let (chunks, meshes) = last;
    let sum = |f: &dyn Fn(&MeshData) -> u32| meshes.iter().map(|m| u64::from(f(m))).sum::<u64>();
    let [opaque, cutout, liquid, s_opaque, s_cutout] = [
        sum(&|m| m.opaque_quads),
        sum(&|m| m.cutout_quads),
        sum(&|m| m.liquid_quads),
        sum(&|m| m.shaped_opaque),
        sum(&|m| m.shaped_cutout),
    ];
    let per = |q: u64| q as f64 / f64::from(chunks.max(1));
    println!(
        "tree chunks {chunks}: quads per chunk opaque {:.0} cutout {:.0} liquid {:.0}",
        per(opaque),
        per(cutout),
        per(liquid)
    );
    // Bytes per chunk: a cube vertex is 4 bytes and a light byte, a shaped one 12 (`mesher/shaped.rs`).
    let bytes = (opaque + cutout + liquid) as f64 * 20.0 + (s_opaque + s_cutout) as f64 * 48.0;
    println!(
        "shaped quads per chunk opaque {:.0} cutout {:.0}; vertex bytes per chunk {:.0}",
        per(s_opaque),
        per(s_cutout),
        bytes / f64::from(chunks.max(1))
    );
    println!("mesh time per tree chunk (best of 5): {:.3} ms", best * 1000.0);
    let share = |cutout: bool, keep: &dyn Fn(u32) -> bool| {
        let (all, square) = meshes.iter().fold((0.0, 0.0), |(a, s), m| {
            let (from, to) = if cutout {
                (m.shaped_opaque as usize, (m.shaped_opaque + m.shaped_cutout) as usize)
            } else {
                (0, m.shaped_opaque as usize)
            };
            let (x, y) = square_share(m, from, to, keep);
            (a + x, s + y)
        });
        100.0 * square / all.max(1.0)
    };
    println!(
        "facing an axis: canopy shell {:.1}%, trunk sides {:.1}% (cubes: 100%)",
        share(true, &|k| k == 0),
        share(false, &|k| (1..=3).contains(&k))
    );
}
