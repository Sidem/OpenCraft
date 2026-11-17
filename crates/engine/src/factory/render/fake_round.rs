//! The standing check that models draw round bodies with `push_round` rather than faking them with boxes (tests only).

use std::f32::consts::FRAC_PI_2;

use super::{INSTANCE_FLOATS, MODE_LIGHT, PITCH, ROUND};

/// A pair of boxes among `boxes` that fake one round body (the standing check that models use [`push_round`](super::push_round): the
/// tests of `factory/tests/fake_round.rs`, `item_models` and `trains`): two untilted boxes with one centre that are
/// the same box turned part of a quarter (an octagon, a star of strips) or the same box with two sides swapped (a
/// cross). Thin bars crossing (a plus, an X) are not round and pass.
pub fn fake_round(boxes: &[f32]) -> Option<(usize, usize)> {
    // Each untilted box as its yaw within a quarter turn and its size, x and z swapped for an odd quarter.
    let norm = |r: &[f32]| {
        let q = ((r[3] + 0.005) / FRAC_PI_2).floor();
        let s = if q.rem_euclid(2.0) == 1.0 { [r[6], r[5], r[4]] } else { [r[4], r[5], r[6]] };
        (r[3] - q * FRAC_PI_2, s)
    };
    let plain = |r: &&[f32]| r[MODE_LIGHT] as u32 & ROUND == 0 && r[PITCH].abs() < 1e-3 && r[13].abs() < 1e-3;
    let recs: Vec<(usize, &[f32])> =
        boxes.chunks_exact(INSTANCE_FLOATS).enumerate().filter(|(_, r)| plain(r)).collect();
    // Within a tenth: fakes stagger their copies a little so their faces don't fight.
    let near = |a: f32, b: f32| (a - b).abs() <= 0.1 * a.max(b) + 0.005;
    let squat = |a: f32, b: f32, share: f32| a.min(b) >= share * a.max(b);
    for (n, &(i, a)) in recs.iter().enumerate() {
        for &(j, b) in &recs[n + 1..] {
            if !(0..3).all(|k| (a[k] - b[k]).abs() <= 0.005) {
                continue;
            }
            let ((ya, sa), (yb, sb)) = (norm(a), norm(b));
            let turn = (ya - yb).abs();
            let same = near(sa[0], sb[0]) && near(sa[2], sb[2]) || near(sa[0], sb[2]) && near(sa[2], sb[0]);
            if same && near(sa[1], sb[1]) && turn > 0.05 && turn < FRAC_PI_2 - 0.05 && squat(sa[0], sa[2], 0.2) {
                return Some((i, j));
            }
            // Crossed about one axis: that axis's side shared, each box the wider on one of the other two, neither
            // a thin bar.
            let crossed = |k: usize| {
                let (u, v) = ((k + 1) % 3, (k + 2) % 3);
                let wider = |p: &[f32; 3], q: &[f32; 3]| p[u] > q[u] + 0.01 && q[v] > p[v] + 0.01;
                near(sa[k], sb[k])
                    && (wider(&sa, &sb) || wider(&sb, &sa))
                    && squat(sa[u], sa[v], 0.5)
                    && squat(sb[u], sb[v], 0.5)
            };
            if turn < 0.01 && (0..3).any(crossed) {
                return Some((i, j));
            }
        }
    }
    None
}
