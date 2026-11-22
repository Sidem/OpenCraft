//! 45° pitched slope / roof geometry for `mesher/blocks`: straight wedges, outer hip pyramid corners,
//! and inner valley corners (both upright and inverted).

use crate::shape::{dir_face, Corner};

use super::Ctx;

const S45: f32 = std::f32::consts::FRAC_1_SQRT_2;
const FULL_UV: [[f32; 2]; 4] = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

/// Emits a 45° pitched slope / roof in canonical `(r, y, b)` space (`inverted` mirrors vertically).
pub(super) fn emit_slope(mut ctx: Ctx<'_>, dir: u8, inv: bool, corner: Corner) {
    let f_back = dir_face(dir) as usize;
    let f_front = f_back ^ 1;
    let f_right = dir_face((dir + 1) & 3) as usize;
    let f_left = f_right ^ 1;
    let f_cap = if inv { 2 } else { 3 };
    let f_slope = if inv { 3 } else { 2 };
    let l_in = ctx.light[ctx.p];

    if !ctx.hidden(f_cap) {
        let l = ctx.face_light(f_cap, true);
        ctx.quad(
            dir,
            inv,
            [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
            FULL_UV,
            [0.0, -1.0, 0.0],
            f_cap,
            l,
        );
    }

    let full_back = |ctx: &mut Ctx<'_>| {
        if !ctx.hidden(f_back) {
            let l = ctx.face_light(f_back, true);
            ctx.quad(
                dir,
                inv,
                [[1.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [1.0, 1.0, 1.0]],
                FULL_UV,
                [0.0, 0.0, 1.0],
                f_back,
                l,
            );
        }
    };
    let tri_right = |ctx: &mut Ctx<'_>| {
        if !ctx.hidden(f_right) {
            let l = ctx.face_light(f_right, true);
            ctx.tri(
                dir,
                inv,
                [[1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0]],
                [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
                [1.0, 0.0, 0.0],
                f_right,
                l,
            );
        }
    };
    let tri_left = |ctx: &mut Ctx<'_>| {
        if !ctx.hidden(f_left) {
            let l = ctx.face_light(f_left, true);
            ctx.tri(
                dir,
                inv,
                [[0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [0.0, 1.0, 1.0]],
                [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]],
                [-1.0, 0.0, 0.0],
                f_left,
                l,
            );
        }
    };

    match corner {
        Corner::Straight => {
            ctx.quad(
                dir,
                inv,
                [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]],
                FULL_UV,
                [0.0, S45, -S45],
                f_slope,
                l_in,
            );
            full_back(&mut ctx);
            tri_right(&mut ctx);
            tri_left(&mut ctx);
        }
        Corner::OuterLeft => {
            ctx.tri(
                dir,
                inv,
                [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 1.0]],
                [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]],
                [0.0, S45, -S45],
                f_slope,
                l_in,
            );
            ctx.tri(
                dir,
                inv,
                [[1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]],
                [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
                [S45, S45, 0.0],
                f_slope,
                l_in,
            );
            if !ctx.hidden(f_back) {
                let l = ctx.face_light(f_back, true);
                ctx.tri(
                    dir,
                    inv,
                    [[1.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0]],
                    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
                    [0.0, 0.0, 1.0],
                    f_back,
                    l,
                );
            }
            tri_left(&mut ctx);
        }
        Corner::OuterRight => {
            ctx.tri(
                dir,
                inv,
                [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
                [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
                [0.0, S45, -S45],
                f_slope,
                l_in,
            );
            ctx.tri(
                dir,
                inv,
                [[0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
                [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]],
                [-S45, S45, 0.0],
                f_slope,
                l_in,
            );
            if !ctx.hidden(f_back) {
                let l = ctx.face_light(f_back, true);
                ctx.tri(
                    dir,
                    inv,
                    [[1.0, 0.0, 1.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0]],
                    [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]],
                    [0.0, 0.0, 1.0],
                    f_back,
                    l,
                );
            }
            tri_right(&mut ctx);
        }
        Corner::InnerLeft => {
            ctx.tri(
                dir,
                inv,
                [[1.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]],
                [[1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                [0.0, S45, -S45],
                f_slope,
                l_in,
            );
            ctx.tri(
                dir,
                inv,
                [[1.0, 0.0, 0.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0]],
                [[0.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                [S45, S45, 0.0],
                f_slope,
                l_in,
            );
            full_back(&mut ctx);
            if !ctx.hidden(f_left) {
                let l = ctx.face_light(f_left, true);
                ctx.quad(
                    dir,
                    inv,
                    [[0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 1.0]],
                    FULL_UV,
                    [-1.0, 0.0, 0.0],
                    f_left,
                    l,
                );
            }
            if !ctx.hidden(f_front) {
                let l = ctx.face_light(f_front, true);
                ctx.tri(
                    dir,
                    inv,
                    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                    [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]],
                    [0.0, 0.0, -1.0],
                    f_front,
                    l,
                );
            }
            tri_right(&mut ctx);
        }
        Corner::InnerRight => {
            ctx.tri(
                dir,
                inv,
                [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]],
                [[0.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                [0.0, S45, -S45],
                f_slope,
                l_in,
            );
            ctx.tri(
                dir,
                inv,
                [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0]],
                [[1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                [-S45, S45, 0.0],
                f_slope,
                l_in,
            );
            full_back(&mut ctx);
            if !ctx.hidden(f_right) {
                let l = ctx.face_light(f_right, true);
                ctx.quad(
                    dir,
                    inv,
                    [[1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0]],
                    FULL_UV,
                    [1.0, 0.0, 0.0],
                    f_right,
                    l,
                );
            }
            if !ctx.hidden(f_front) {
                let l = ctx.face_light(f_front, true);
                ctx.tri(
                    dir,
                    inv,
                    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
                    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
                    [0.0, 0.0, -1.0],
                    f_front,
                    l,
                );
            }
            tri_left(&mut ctx);
        }
    }
}
