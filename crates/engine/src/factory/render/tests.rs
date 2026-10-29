use super::*;

fn boxes_at(cells: &[(f64, f64, f64)]) -> Vec<f32> {
    let mut out = Vec::new();
    for (i, &(x, y, z)) in cells.iter().enumerate() {
        push_box(&mut out, Vec3::new(x, y, z), 0.0, [1.0; 3], 0.0, [1, 2, 3], i % 2 == 1);
    }
    out
}

#[test]
fn boxes_start_in_daylight_and_light_boxes_sets_each_cells_light() {
    let mut boxes = boxes_at(&[(0.5, 0.5, 0.5), (3.5, 0.5, 0.5), (3.9, 0.9, 0.1)]);
    let mode_light = |b: &[f32], i: usize| b[i * INSTANCE_FLOATS + MODE_LIGHT];
    assert_eq!(mode_light(&boxes, 0), 4.0 * DAYLIGHT as f32);
    assert_eq!(mode_light(&boxes, 1), 1.0 + 4.0 * DAYLIGHT as f32, "the uv mode rides in the low bit");

    // The eye is 10 blocks off: centres are relative to it.
    let eye = Vec3::new(10.0, 0.0, 0.0);
    let mut asked = Vec::new();
    light_boxes(&mut boxes, eye, |cell| {
        asked.push(cell);
        if cell.x == 13 {
            0x4a
        } else {
            0x0f
        }
    });
    assert_eq!(asked, [IVec3::new(10, 0, 0), IVec3::new(13, 0, 0)], "a cell is asked once for boxes in it");
    assert_eq!(mode_light(&boxes, 0), 4.0 * 15.0);
    assert_eq!(mode_light(&boxes, 1), 1.0 + 4.0 * 74.0);
    assert_eq!(mode_light(&boxes, 2), 4.0 * 74.0, "the same cell as the box before it");
}

#[test]
fn a_box_inside_solid_ground_borrows_the_light_beside_it() {
    let mut buried = boxes_at(&[(0.5, 0.5, 0.5), (5.5, 0.5, 0.5)]);
    let air = |c: IVec3| match (c.x, c.y) {
        (0, 1) => 0x2c, // above the first box: sky 12, block light 2
        (0, 0) => 0,    // the buried cell itself
        (0, _) | (1, _) => 0x0a,
        _ => 0, // the second box sits in a real cave: nothing around it either
    };
    light_boxes(&mut buried, Vec3::ZERO, air);
    let lit = |i: usize| buried[i * INSTANCE_FLOATS + MODE_LIGHT] / 4.0;
    assert_eq!(lit(0) as u8, 0x2c);
    assert_eq!(lit(1) as u8, 0, "a box in a dark cave stays dark");
}

#[test]
fn round_bodies_keep_their_mark_through_lighting_and_split_from_the_boxes() {
    let mut out = boxes_at(&[(0.5, 0.5, 0.5)]);
    push_round(&mut out, Vec3::new(1.5, 0.5, 0.5), 0.3, Axis::X, [0.4, 0.8, 0.4], 0.5, [4, 5, 6], true);
    push_round(&mut out, Vec3::new(2.5, 0.5, 0.5), 0.0, Axis::Up, [0.4, 0.8, 0.4], 1.0, [4, 5, 6], false);
    light_boxes(&mut out, Vec3::ZERO, |_| 0x3b);
    let mut round = Vec::new();
    split_round(&out, &mut round);
    assert_eq!(round.len(), 2 * INSTANCE_FLOATS, "the box stays behind");
    assert_eq!(round[..INSTANCE_FLOATS], out[INSTANCE_FLOATS..2 * INSTANCE_FLOATS]);
    let mode = |r: &[f32]| r[MODE_LIGHT] as u32;
    assert_eq!(mode(&round) & 3, 1 | ROUND, "lighting keeps the uv mode and the round bit");
    assert_eq!(mode(&round[INSTANCE_FLOATS..]) & 3, ROUND);
    assert_eq!(mode(&round) >> 2, 0x3b);
    // Laid along x: a quarter pitch, and its yaw a quarter less than asked; the taper rides along.
    assert!((round[3] - (0.3 - std::f32::consts::FRAC_PI_2)).abs() < 1e-6);
    assert!((round[PITCH] - std::f32::consts::FRAC_PI_2).abs() < 1e-6 && round[TAPER] == 0.5);
}

#[test]
fn a_round_body_laid_along_x_points_its_far_end_along_its_turned_x() {
    // The box shader's turn (`boxVert`): yaw * roll * pitch applied to the cylinder's own +y.
    for yaw in [0.0f32, 0.7, -2.0] {
        let mut out = Vec::new();
        push_round(&mut out, Vec3::ZERO, yaw, Axis::X, [1.0; 3], 1.0, [0; 3], false);
        let (y, p) = (out[3], out[PITCH]);
        let tilted = [0.0, p.cos(), p.sin()]; // pitch takes +y towards +z
        let world = [y.cos() * tilted[0] - y.sin() * tilted[2], tilted[1], y.sin() * tilted[0] + y.cos() * tilted[2]];
        let want = [yaw.cos(), 0.0, yaw.sin()];
        assert!(world.iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-5), "{world:?} != {want:?}");
    }
}
