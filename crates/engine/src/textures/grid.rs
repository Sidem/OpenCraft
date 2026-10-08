//! The power grids' accent tiles: one solid colour per palette entry of `factory/grid_colour.rs` (wires, pole collars and
//! cable knots wear them), with a light grain so a long wire isn't flat. Placeholder looks until the art pass.

use crate::block::tex;
use crate::factory::grid_colour::PALETTE;

use super::{n, rgb};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    let [r, g, b] = PALETTE[(layer - tex::GRID_FIRST) as usize].1;
    rgb([r as f64, g as f64, b as f64], 0.94 + 0.12 * n(620 + layer as u32, x, y))
}
