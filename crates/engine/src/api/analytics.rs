//! The analytics hub (`web/src/ui/analytics*.ts`) and the efficiency lines of machines: graph data from `analytics/`,
//! read-only. Item rows come two to an item (made, then used); power rows are the totals, the sources, then the users.

use wasm_bindgen::prelude::*;

use crate::analytics::{self, GROUPS};
use crate::math::IVec3;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// The last `secs` seconds of the item graphs as `[points, rows, then points values per row, oldest first]`: for
    /// each item of `analytics_items`, a row of how many were made a minute, then a row of how many were used up.
    /// `NaN` marks time before the world was opened.
    pub fn analytics_item_graphs(&self, secs: u32, points: u32) -> Vec<f32> {
        self.analytics.export_items(secs, points as usize)
    }

    /// The items with graph rows, as item ids.
    pub fn analytics_items(&self) -> Vec<u32> {
        self.analytics.item_ids()
    }

    /// The group of each item of `analytics_items` (an index into `analytics_groups`).
    pub fn analytics_item_groups(&self) -> Vec<u32> {
        self.analytics.item_groups()
    }

    /// Per item of `analytics_items`: `[made a minute, used a minute, made in all, used in all]` (the totals since the
    /// world was opened).
    pub fn analytics_item_now(&self) -> Vec<f32> {
        self.analytics.item_now()
    }

    /// The names of the item groups, one a line.
    pub fn analytics_groups(&self) -> String {
        GROUPS.map(|g| g.name).join("\n")
    }

    /// The colour of each item group as `0xRRGGBB`.
    pub fn analytics_group_colours(&self) -> Vec<u32> {
        GROUPS.iter().map(|g| g.colour).collect()
    }

    /// The last `secs` seconds of the power graphs, laid out like the item graphs (kW): the grids' capacity, power
    /// given and power asked for, then a row per source (`analytics_power_sources`), then a row per kind of machine
    /// asking for power (`analytics_power_users`).
    pub fn analytics_power_graphs(&self, secs: u32, points: u32) -> Vec<f32> {
        self.analytics.export_power(secs, points as usize)
    }

    /// The newest value of every power row.
    pub fn analytics_power_now(&self) -> Vec<f32> {
        self.analytics.power_now()
    }

    /// What each power source could give right now (kW).
    pub fn analytics_power_could(&self) -> Vec<u32> {
        self.analytics.power_could()
    }

    /// The power sources' names, one a line, in row order.
    pub fn analytics_power_sources(&self) -> String {
        analytics::power_sources()
    }

    /// The power sources' colours as `0xRRGGBB`.
    pub fn analytics_power_source_colours(&self) -> Vec<u32> {
        analytics::power_source_colours()
    }

    /// The block of each kind of machine that has asked for power (rows after the sources), usable as an item id.
    pub fn analytics_power_users(&self) -> Vec<u32> {
        self.analytics.power_users()
    }

    /// Machine counts: `[working, at full speed, held back by power, input, output, fuel, water, deposit, other]`.
    pub fn analytics_machines(&self) -> Vec<u32> {
        self.analytics.machines.summary().to_vec()
    }

    /// Machines by kind, each tier its own (the kind is the item id of the machine's tier, its block's for a machine without
    /// tiers): rows of `analytics_type_fields` numbers, `[kind, working, at full speed, average percent,
    /// idle, then a count held back by each cause]` (the causes in the order of `analytics_machines`).
    pub fn analytics_machine_types(&self) -> Vec<u32> {
        self.analytics.machines.by_type()
    }

    /// How many numbers a row of `analytics_machine_types` has.
    pub fn analytics_type_fields(&self) -> u32 {
        analytics::TYPE_FIELDS as u32
    }

    /// The `n` slowest working machines: `[x, y, z, kind, percent, cause]` each (the cause indexes
    /// `analytics_machines` after its first two counts).
    pub fn analytics_worst_machines(&self, n: u32) -> Vec<i32> {
        self.analytics.machines.worst(n as usize)
    }

    /// Every machine of `kind` (an item id) counted in `analytics_machine_types`, `analytics_kind_fields` numbers each:
    /// `[x, y, z, percent, cause]` (percent -1 when idle; cause -1 when idle or at full speed, else an index as in
    /// `analytics_worst_machines`).
    pub fn analytics_machines_of(&self, kind: u32) -> Vec<i32> {
        self.analytics.machines.of_kind(kind as u16)
    }

    /// How many numbers a machine of `analytics_machines_of` has.
    pub fn analytics_kind_fields(&self) -> u32 {
        analytics::KIND_FIELDS as u32
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
