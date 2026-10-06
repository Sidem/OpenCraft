//! The analytics screen (`web/src/ui/analytics.ts`) and the efficiency lines of machines: graph data from
//! `analytics/`, read-only.

use wasm_bindgen::prelude::*;

use crate::math::IVec3;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// The last `secs` seconds of the graphs as `[points, rows, then points values per row, oldest first]`. Rows:
    /// kW the grids could give, kW used, kW asked for, then one per item in `analytics_items` order (made a minute).
    /// `NaN` marks time before the world was opened.
    pub fn analytics_graphs(&self, secs: u32, points: u32) -> Vec<f32> {
        self.analytics.export(secs, points as usize)
    }

    /// The newest value of every graph row, in the same order.
    pub fn analytics_now(&self) -> Vec<f32> {
        self.analytics.latest()
    }

    /// The items with a graph row (after the three power rows), as item ids.
    pub fn analytics_items(&self) -> Vec<u32> {
        self.analytics.item_ids()
    }

    /// Machine counts: `[working, at full speed, held back by power, input, output, fuel, water, deposit, other]`.
    pub fn analytics_machines(&self) -> Vec<u32> {
        self.analytics.machines.summary().to_vec()
    }

    /// How well the machine at a position has run lately, in words (empty if it has no efficiency).
    pub fn machine_efficiency(&self, x: i32, y: i32, z: i32) -> String {
        self.efficiency_line(IVec3::new(x, y, z))
    }

    /// The same as a percentage, or -1 when it is idle or has none.
    pub fn machine_efficiency_percent(&self, x: i32, y: i32, z: i32) -> i32 {
        let anchor = self.sim.factory.anchor_of(IVec3::new(x, y, z));
        anchor.and_then(|a| self.analytics.machines.percent(a)).map_or(-1, |p| p as i32)
    }
}

impl Game {
    /// The efficiency line of the machine at `cell`, for any of its cells.
    pub(crate) fn efficiency_line(&self, cell: IVec3) -> String {
        self.sim.factory.anchor_of(cell).map_or_else(String::new, |a| self.analytics.machines.line(a))
    }
}
