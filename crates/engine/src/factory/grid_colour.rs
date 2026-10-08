//! Colour-coded power grids: every grid wears one accent colour, so a glance tells whether a pole joins the main grid
//! or stands alone. Presentation only (derived at every relink, never saved, nothing in the core reads it).
//!
//! - `assign` gives each grid (`Power::pole_grid`) a palette index: the grid with the most poles is the main grid and
//!   always wears `PALETTE[0]`; the others take `PALETTE[1..]` by size, then lowest pole, repeating when there are more
//!   grids than colours. Joining two grids makes one grid, so both take one colour.
//! - `write_accents` draws a collar and a crossarm sleeve in the grid's colour on every pole; wires (`power.rs`) and
//!   cable knots (`cable.rs`) take the colour through `Power::pole_layer`.
//! - The palette is the Okabe-Ito set (readable with common colour blindness) and each entry has a name, which the
//!   pole readout uses (`Power::grid_label`). To add a colour: one row here and one in `tex::GRID_COLOURS`.

use crate::block::tex;
use crate::math::Vec3;

use super::power::Power;
use super::render::push_box;
use super::Factory;

/// Name and colour of each accent. The first is the main grid's.
pub const PALETTE: [(&str, [u8; 3]); tex::GRID_COLOURS] = [
    ("blue", [0, 114, 178]),
    ("orange", [230, 159, 0]),
    ("pink", [204, 121, 167]),
    ("green", [0, 158, 115]),
    ("yellow", [240, 228, 66]),
    ("red", [213, 40, 30]),
];

/// The palette index of each grid, from the grid of each pole (grids are numbered from 0).
pub(super) fn assign(pole_grid: &[u32]) -> Vec<u8> {
    let grids = pole_grid.iter().max().map_or(0, |&g| g as usize + 1);
    let mut size = vec![0u32; grids];
    for &g in pole_grid {
        size[g as usize] += 1;
    }
    // A grid's rank: how many grids come before it (bigger, or as big with a lower number).
    (0..grids)
        .map(|g| {
            let rank = (0..grids).filter(|&h| size[h] > size[g] || size[h] == size[g] && h < g).count();
            if rank == 0 {
                0
            } else {
                1 + ((rank - 1) % (PALETTE.len() - 1)) as u8
            }
        })
        .collect()
}

impl Power {
    /// The accent texture layer of pole `pole`'s grid (the main grid's when unknown).
    pub(super) fn pole_layer(&self, pole: u32) -> u16 {
        let colour = self.pole_grid.get(pole as usize).and_then(|&g| self.grid_colour.get(g as usize));
        tex::GRID_FIRST + u16::from(colour.copied().unwrap_or(0))
    }

    /// What a pole's readout calls its grid: "Main grid (blue)" or "Separate grid (orange)".
    pub(super) fn grid_label(&self, pole: u32) -> String {
        let colour = (self.pole_layer(pole) - tex::GRID_FIRST) as usize;
        let kind = if colour == 0 { "Main grid" } else { "Separate grid" };
        format!("{kind} ({})", PALETTE[colour].0)
    }
}

impl Factory {
    /// A collar on the post and a sleeve on the crossarm of every pole within `range`, in its grid's colour.
    pub(super) fn write_accents(&self, out: &mut Vec<f32>, eye: Vec3, range: f64) {
        for (i, p) in self.poles.iter().enumerate().filter(|(_, p)| !p.is_cable()) {
            let rel = p.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
            if rel.x * rel.x + rel.y * rel.y + rel.z * rel.z > range * range {
                continue;
            }
            let layer = [self.power.pole_layer(i as u32); 3];
            push_box(out, rel + Vec3::new(0.0, 0.16, 0.0), 0.0, [0.19, 0.12, 0.19], 0.0, layer, false);
            push_box(out, rel + Vec3::new(0.0, 0.34, 0.0), 0.0, [0.40, 0.10, 0.14], 0.0, layer, false);
        }
    }
}

#[cfg(test)]
mod tests;
