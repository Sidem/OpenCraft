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
    assert_eq!(mode_light(&boxes, 0), 2.0 * DAYLIGHT as f32);
    assert_eq!(mode_light(&boxes, 1), 1.0 + 2.0 * DAYLIGHT as f32, "the uv mode rides in the low bit");

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
    assert_eq!(mode_light(&boxes, 0), 2.0 * 15.0);
    assert_eq!(mode_light(&boxes, 1), 1.0 + 2.0 * 74.0);
    assert_eq!(mode_light(&boxes, 2), 2.0 * 74.0, "the same cell as the box before it");
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
    let lit = |i: usize| buried[i * INSTANCE_FLOATS + MODE_LIGHT] / 2.0;
    assert_eq!(lit(0) as u8, 0x2c);
    assert_eq!(lit(1) as u8, 0, "a box in a dark cave stays dark");
}
