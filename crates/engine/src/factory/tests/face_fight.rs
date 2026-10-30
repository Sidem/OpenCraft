//! The standing check that models draw no two faces in one plane facing one way: such faces fight over each pixel and
//! flicker as the camera moves (joined pipes once did). `face_fight` looks at every axis-aligned box and cylinder of a
//! scene, faces of one instance against another's, the same cell or a neighbour's. Back faces are culled, so faces
//! facing apart never fight and pass. Applied to pipework: every set of joined faces, each kind of piece around each.

use crate::block::{BlockId, OUTLET, PIPE, PUMP};
use crate::factory::render::{push_round, ROUND};
use crate::factory::{push_box, Axis, Factory, FACES, INSTANCE_FLOATS};
use crate::math::{IVec3, Vec3};
use crate::world::World;

/// Faces nearer than this (blocks) share a plane for the depth buffer at play distances.
const PLANE: f32 = 0.002;
/// Overlaps thinner than this are edges touching, not faces sharing area.
const AREA: f32 = 0.002;
// The instance layout (`render::INSTANCE_FLOATS`): mode bits, pitch, roll, taper.
const MODE: usize = 11;
const PITCH: usize = 12;
const ROLL: usize = 13;
const TAPER: usize = 14;

/// A flat face: its plane (axis, outward sign, where) and its outline in the other two axes.
#[derive(Debug)]
struct Face {
    axis: usize,
    out: bool,
    at: f32,
    shape: Outline,
}

#[derive(Debug)]
enum Outline {
    Rect([f32; 2], [f32; 2]),
    Disc([f32; 2], f32),
}

/// A cylinder's curved wall: its axis, centre line, radius and span.
#[derive(Debug)]
struct Wall {
    axis: usize,
    centre: [f32; 2],
    r: f32,
    span: [f32; 2],
}

/// The first two instances of `boxes` with faces that fight, or none. Instances turned off the axes are skipped.
pub fn face_fight(boxes: &[f32]) -> Option<(usize, usize)> {
    let shapes: Vec<_> =
        boxes.chunks_exact(INSTANCE_FLOATS).enumerate().filter_map(|(i, r)| shape(r).map(|s| (i, s))).collect();
    for (n, (i, (fa, wa))) in shapes.iter().enumerate() {
        for (j, (fb, wb)) in &shapes[n + 1..] {
            let flat = fa.iter().any(|a| fb.iter().any(|b| faces_fight(a, b)));
            let round = wa.iter().any(|a| wb.iter().any(|b| walls_fight(a, b)));
            if flat || round {
                return Some((*i, *j));
            }
        }
    }
    None
}

/// An instance's faces and wall in world axes, or none if it is turned off the axes or tapered.
fn shape(r: &[f32]) -> Option<(Vec<Face>, Option<Wall>)> {
    let (yaw, pitch) = (r[3], r[PITCH]);
    if r[ROLL].abs() > 1e-4 || (r[TAPER] - 1.0).abs() > 1e-4 {
        return None;
    }
    // The instance's own axes in the world (`render::Form` gives the turn: yaw about y after pitch about x).
    let (sy, cy, sp, cp) = (yaw.sin(), yaw.cos(), pitch.sin(), pitch.cos());
    let own = [[cy, 0.0, sy], [-sp * sy, cp, sp * cy], [-cp * sy, -sp, cp * cy]];
    let mut along = [0usize; 3];
    for (k, e) in own.iter().enumerate() {
        let axis = (0..3).find(|&i| (e[i].abs() - 1.0).abs() < 1e-3)?;
        if (0..3).any(|i| i != axis && e[i].abs() > 1e-3) {
            return None;
        }
        along[k] = axis;
    }
    let mut half = [0.0f32; 3];
    for k in 0..3 {
        half[along[k]] = r[4 + k] / 2.0;
    }
    let c = [r[0], r[1], r[2]];
    let other = |a: usize| [(a + 1) % 3, (a + 2) % 3];
    let rect = |a: usize| {
        let [u, v] = other(a);
        Outline::Rect([c[u] - half[u], c[v] - half[v]], [c[u] + half[u], c[v] + half[v]])
    };
    let mut faces = Vec::new();
    if r[MODE] as u32 & ROUND == 0 {
        for a in 0..3 {
            for out in [false, true] {
                faces.push(Face { axis: a, out, at: c[a] + if out { half[a] } else { -half[a] }, shape: rect(a) });
            }
        }
        return Some((faces, None));
    }
    // A cylinder's own axis is its y.
    let a = along[1];
    let [u, v] = other(a);
    let (centre, radius) = ([c[u], c[v]], half[u].min(half[v]));
    for out in [false, true] {
        let at = c[a] + if out { half[a] } else { -half[a] };
        faces.push(Face { axis: a, out, at, shape: Outline::Disc(centre, radius) });
    }
    let wall = Wall { axis: a, centre, r: radius, span: [c[a] - half[a], c[a] + half[a]] };
    Some((faces, Some(wall)))
}

fn faces_fight(a: &Face, b: &Face) -> bool {
    if a.axis != b.axis || a.out != b.out || (a.at - b.at).abs() > PLANE {
        return false;
    }
    let dist = |p: [f32; 2], lo: [f32; 2], hi: [f32; 2]| {
        let d = |k: usize| (lo[k] - p[k]).max(p[k] - hi[k]).max(0.0);
        d(0).hypot(d(1))
    };
    match (&a.shape, &b.shape) {
        (Outline::Rect(alo, ahi), Outline::Rect(blo, bhi)) => {
            (0..2).all(|k| ahi[k].min(bhi[k]) - alo[k].max(blo[k]) > AREA)
        }
        (Outline::Disc(c, r), Outline::Rect(lo, hi)) | (Outline::Rect(lo, hi), Outline::Disc(c, r)) => {
            dist(*c, *lo, *hi) < r - AREA
        }
        (Outline::Disc(p, r), Outline::Disc(q, s)) => (p[0] - q[0]).hypot(p[1] - q[1]) < r + s - AREA,
    }
}

fn walls_fight(a: &Wall, b: &Wall) -> bool {
    let coaxial = a.axis == b.axis && (0..2).all(|k| (a.centre[k] - b.centre[k]).abs() < PLANE);
    coaxial && (a.r - b.r).abs() < PLANE && a.span[1].min(b.span[1]) - a.span[0].max(b.span[0]) > AREA
}

/// Every piece of pipework at the centre facing each way, with each kind of piece on every set of its faces.
#[test]
fn pipework_draws_no_faces_that_fight() {
    const PIECES: [BlockId; 3] = [PIPE, PUMP, OUTLET];
    let mut scenes = 0;
    for centre in PIECES {
        for around in PIECES {
            for facing in 0..4 {
                for joined in 0..64u8 {
                    let (mut world, mut f) = (World::new(1, 2), Factory { by_hand: true, ..Factory::default() });
                    let pos = IVec3::new(0, 64, 0);
                    f.place(&mut world, centre, pos, facing, pos - IVec3::new(0, 1, 0), 0);
                    for &n in FACES.iter().enumerate().filter(|&(k, _)| joined & 1 << k != 0).map(|(_, n)| n) {
                        f.place(&mut world, around, pos + n, facing, pos + n - IVec3::new(0, 1, 0), 0);
                    }
                    f.relink();
                    let mut out = Vec::new();
                    f.write_instances(&mut out, Vec3::new(0.5, 66.0, 0.5), 0.0, 40.0);
                    if let Some((a, b)) = face_fight(&out) {
                        let at = |i: usize| &out[i * INSTANCE_FLOATS..i * INSTANCE_FLOATS + 7];
                        panic!(
                            "block {centre} facing {facing} with {around} on faces {joined:06b}: {:?} and {:?} fight",
                            at(a),
                            at(b)
                        );
                    }
                    scenes += 1;
                }
            }
        }
    }
    assert_eq!(scenes, 3 * 3 * 4 * 64);
}

#[test]
fn the_detector_finds_shared_faces_and_passes_faces_facing_apart() {
    let o = Vec3::new(0.0, 0.0, 0.0);
    let x = |d: f64| Vec3::new(d, 0.0, 0.0);
    let fight = |a: &dyn Fn(&mut Vec<f32>), b: &dyn Fn(&mut Vec<f32>)| {
        let mut out = Vec::new();
        a(&mut out);
        b(&mut out);
        face_fight(&out).is_some()
    };
    let cube = |at: Vec3, s: f32| move |out: &mut Vec<f32>| push_box(out, at, 0.0, [s; 3], 0.0, [0; 3], false);
    let bar = |at: Vec3, len: f32, d: f32| {
        move |out: &mut Vec<f32>| push_round(out, at, 0.0, Axis::X, [len, d, d], 1.0, [0; 3], false)
    };
    assert!(fight(&cube(o, 0.5), &cube(x(0.1), 0.5)), "overlapping boxes share their top");
    assert!(!fight(&cube(o, 0.5), &cube(x(0.5), 0.5)), "boxes meeting face to face face apart");
    assert!(!fight(&cube(o, 0.5), &cube(o, 0.3)), "a box inside another");
    assert!(fight(&bar(o, 0.6, 0.24), &bar(x(0.4), 0.6, 0.24)), "overlapping cylinders share their wall");
    assert!(!fight(&bar(o, 0.5, 0.24), &bar(x(0.5), 0.5, 0.24)), "cylinders meeting end to end");
    assert!(!fight(&bar(o, 0.6, 0.24), &bar(x(0.4), 0.6, 0.3)), "a cylinder through a wider one");
    assert!(fight(&bar(o, 0.5, 0.24), &cube(o, 0.5)), "a cylinder's ends on a box's sides");
    // Turned a quarter, a box is the same box: still checked.
    let turned = |out: &mut Vec<f32>| push_box(out, x(0.1), std::f32::consts::FRAC_PI_2, [0.5; 3], 0.0, [0; 3], false);
    assert!(fight(&cube(o, 0.5), &turned), "a box turned a quarter");
}
