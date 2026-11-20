//! Factory machines: belts, miners, storage, processors (`process/`), routers, power, labs, pipework, quarries,
//! terraforming sites, rails and fibre (`fibre.rs`). Tiers index `tiers.rs`.
//!
//! Machines occupy one voxel each (multi-block processors several: `footprint/`), keyed in `at`, and keep running
//! when streamed out. Each kind implements [`Machine`] in its own `Vec`, indexed via `Slot`. Core state (DEV_PLAN §3.4):
//! `update` (`tick.rs`) runs one fixed tick and reports via `SimEvent`s; links/order rebuild in `relink` when `dirty`.
//! Processors are spec rows (`process/specs.rs`); new kinds add a `Kind`/`Slot`/`MACHINES` row and `Vec`.

mod belt;
mod belt_chain;
mod belt_curve;
mod belt_items;
mod belt_model;
mod belt_order;
mod belt_shape;
pub mod box_label;
mod buffer;
mod cable;
mod capsule;
mod describe;
pub mod efficiency;
mod facts;
mod fibre;
pub mod filter_rules;
pub mod footprint;
mod generator;
pub mod grid_colour;
mod hitbox;
mod lab;
mod laser;
mod links;
mod machine;
mod miner;
mod miner_burner;
mod miner_model;
mod panel;
mod pipes;
mod placed;
mod plume;
mod pole;
mod pole_hang;
pub mod pole_mount;
mod pole_tower;
pub mod port_marks;
mod ports;
mod power;
pub mod power_mix;
mod preview;
mod process;
mod provenance;
mod pumping;
mod quarry;
mod rail;
mod render;
mod router;
mod router_model;
mod sensor;
mod sites;
pub(crate) mod smoke;
mod state;
mod static_view;
pub mod stock;
mod storage;
mod switch;
mod table;
mod tick;
mod tier_mark;
pub mod tiers;
pub(crate) mod trains;
pub mod underpass;
pub mod upgrades;
mod wiring;

use rustc_hash::FxHashMap;

use crate::block::{BlockId, CABLE, COAL_MINER, FACE_BOTTOM, FAST_BELT, LONG_POLE, MINER_MK2, POLE, SOLID};
use crate::deposits::{DepositKey, Deposits};
use crate::inventory::Stack;
#[cfg(test)]
use crate::item::ItemId;
use crate::math::IVec3;
#[cfg(test)]
use crate::math::Vec3;
use crate::pods::Pods;
use crate::research::Research;
use crate::satellites::Satellites;
use crate::sim::SimEvent;
use crate::world::World;
use crate::TICK_RATE;

use belt::Belt;
#[cfg(test)]
pub(crate) use belt::BELT_HEIGHT;
pub(crate) use belt::BELT_TIERS;
use fibre::{Data, Node};
use generator::Generator;
use lab::Lab;
use links::Slot;
use machine::{add_to, swap_out, Machine};
use miner::Miner;
use pipes::Pipework;
use pole::Pole;
use power::Power;
use process::Processor;
use quarry::Quarry;
use rail::Rail;
use router::{Router, RouterKind};
use sensor::Sensor;
use storage::Storage;
use trains::Train;
use wiring::Hook;

#[cfg(test)]
pub use belt::{FLOW_SECS, FLOW_WINDOWS};
pub use belt_curve::{belt_facing, bend_of, clean_facing, BENDS, BEND_LEFT, BEND_RIGHT, BEND_STRAIGHT};
pub use belt_items::{ItemAim, ItemKey};
pub use buffer::Buffer;
pub(crate) use capsule::push_capsule;
pub use describe::{fmt_int, hud_bar, hud_keys, hud_stats, hud_status};
pub use facts::facts;
#[cfg(test)]
pub use miner::MinerStatus;
pub use miner::MINER_TIERS;
pub use miner_burner::CoalMinerView;
pub use panel::{turns, ROLE_FUEL, ROLE_INPUT, ROLE_OUTPUT, ROLE_STORE};
pub use pole::{
    long_top, preview_pole, preview_wire, tier_stats, LongLink, LONG_LOCAL_LINK, LONG_STATS, LONG_TIER, POLE_TIERS,
    POLE_UNLINKED, POLE_WIRE_Y,
};
pub use pole_hang::ghost_hang;
pub use ports::PortInfo;
pub use power::{sag_point, wire};
pub(crate) use process::EJECTOR_ICON;
pub use process::{
    channel_colour, channel_name, first_maker, forester_survey, makes, spec as process_spec, tenths, tenths_a_minute,
    ForesterPatch, ForesterSurvey, Want,
};
#[cfg(test)]
pub use process::{ProcessSpec, Status as ProcessStatus, SPECS};
pub use quarry::{survey, DigBox, DEFAULT_DEPTH, DEFAULT_WIDTH, DEPTHS, WIDTHS};
pub use rail::{
    dir_of, fit as fit_track, is_dock, rail_faults, rail_node_stands, write_node, write_track, yaw_of, Curve, DockFit,
    Fault, Fit, SplitWhy, Track, MAX_LINKS, MAX_SPAN,
};
pub use render::{
    copy_near, is_round, light_boxes, make_round, mark_beacon, mark_beacon_tinted, push_box, push_cone, push_framed,
    push_round, push_shape, shape_of, split_round, Axis, Form, Framed, INSTANCE_FLOATS, ROUND,
};
pub use sensor::RULES as SENSOR_RULES;
pub use sites::{cut_takes, survey_site, survey_tunnel, touches_water, Job, Site, SiteSurvey, Sites, Tunnel, SECTIONS};
pub(crate) use static_view::{bin_boxes, ChunkIndex, ChunkMembers};
pub use trains::{car_at, car_box, CarAim, Spot, DROP_ALL, MAX_CARS};
pub use wiring::Hookup;
pub use {belt::belt_preview, belt::lift_preview, belt_shape::Shape, cable::preview_cable};

/// Horizontal directions in player-yaw quarter turns: 0 = -Z (north), 1 = +X, 2 = +Z, 3 = -X.
pub const DIRS: [IVec3; 4] = [IVec3::new(0, 0, -1), IVec3::new(1, 0, 0), IVec3::new(0, 0, 1), IVec3::new(-1, 0, 0)];
/// Block face normals in mesher order: +X, -X, +Y, -Y, +Z, -Z.
pub const FACES: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

/// The horizontal direction a player facing `yaw` looks along.
pub fn dir_from_yaw(yaw: f64) -> u8 {
    ((yaw / std::f64::consts::FRAC_PI_2).round() as i32).rem_euclid(4) as u8
}

pub fn face_of(v: IVec3) -> Option<u8> {
    FACES.iter().position(|&f| f == v).map(|i| i as u8)
}

#[cfg(test)]
use table::MACHINES;
pub use table::{machine, Kind};

#[derive(Default)]
pub struct Factory {
    belts: Vec<Belt>,
    miners: Vec<Miner>,
    storages: Vec<Storage>,
    processors: Vec<Processor>,
    routers: Vec<Router>,
    generators: Vec<Generator>,
    poles: Vec<Pole>,
    labs: Vec<Lab>,
    pipework: Vec<Pipework>,
    quarries: Vec<Quarry>,
    sensors: Vec<Sensor>,
    rails: Vec<Rail>,
    /// The data grid's fibre nodes, saved (`fibre.rs`).
    nodes: Vec<Node>,
    /// The track between rail nodes, saved (`rail.rs`).
    tracks: Vec<Track>,
    /// The locomotives on the track, saved (`trains.rs`).
    trains: Vec<Train>,
    /// The player's power wires, saved (`wiring.rs`).
    hooks: Vec<Hook>,
    /// Anchors of machines manually switched off (`switch.rs`, sorted, saved from version 80).
    manual_off: Vec<IVec3>,
    /// Tests only: leave wires to `connect` (else every relink wires by range, `hook_by_reach`).
    #[cfg(test)]
    pub(crate) by_hand: bool,
    /// Grids and last tick's supply and demand (derived, see `power.rs`).
    power: Power,
    /// The data grids and last tick's compute (derived, see `fibre.rs`).
    data: Data,
    /// The processors of each kind with a pass of its own (derived, see `process/kinds.rs`).
    kinds: process::Kinds,
    /// The laser beams between emitters and receivers (derived, see `laser.rs`).
    beams: laser::Beams,
    at: FxHashMap<IVec3, Slot>,
    /// Belt indices, downstream first.
    order: Vec<u32>,
    dirty: bool,
    /// Bumped on any structural or static-visual change (derived, not saved or hashed: `static_boxes`).
    pub(crate) layout_gen: u64,
    pub deposits: Deposits,
    /// The world's research, which labs advance.
    pub research: Research,
    /// The world's terraforming sites (`sites.rs`).
    pub sites: Sites,
    /// The satellites in orbit, which launch sites fill (`satellites.rs`).
    pub orbit: Satellites,
    /// The drop pods in flight between pod launchers and landing pads (`pods.rs`).
    pub pods: Pods,
    /// Blocks the machines changed this tick, with the one each replaced (not state: `Sim::step` drains it).
    pub changed: Vec<(IVec3, BlockId)>,
}

impl Factory {
    /// How many machines of `kind` there are.
    pub fn count(&self, kind: Kind) -> usize {
        match kind {
            Kind::Belt => self.belts.len(),
            Kind::Miner => self.miners.len(),
            Kind::Storage => self.storages.len(),
            Kind::Process => self.processors.len(),
            Kind::Router => self.routers.len(),
            Kind::Generator => self.generators.len(),
            Kind::Pole => self.poles.len(),
            Kind::Lab => self.labs.len(),
            Kind::Pipe => self.pipework.len(),
            Kind::Quarry => self.quarries.len(),
            Kind::Sensor => self.sensors.len(),
            Kind::Rail => self.rails.len(),
            Kind::Node => self.nodes.len(),
        }
    }

    /// How many processors of `block` there are (smelters, constructors).
    pub fn processors_of(&self, block: BlockId) -> usize {
        self.processors.iter().filter(|p| p.spec.block == block).count()
    }

    /// Whether any `block` machine holds `item` to work on (a crusher fed slag: `hints::SIDE_HINTS`).
    pub fn any_fed(&self, block: BlockId, item: crate::item::ItemId) -> bool {
        self.processors.iter().any(|p| p.spec.block == block && p.input.count(item) > 0)
    }

    /// Adds the machine that `block` is at `pos` (`facing` horizontal direction, `against` clicked block).
    pub fn place(&mut self, world: &mut World, block: BlockId, pos: IVec3, facing: u8, against: IVec3, tier: u8) {
        let Some(def) = machine(block) else { return };
        self.remove(pos);
        let tier = match block {
            LONG_POLE => pole::LONG_TIER,
            COAL_MINER => miner_burner::COAL_TIER,
            _ => tier,
        };
        let at = &mut self.at;
        match def.kind {
            Kind::Belt => self.add_shaped_belt(pos, facing, Shape::of(block), tier.max((block == FAST_BELT) as u8)),
            Kind::Miner => {
                let drill = face_of(against - pos);
                let deposit = drill.and_then(|_| self.deposits.lookup(world, against));
                self.add_miner(pos, drill.unwrap_or(FACE_BOTTOM as u8), deposit, tier.max((block == MINER_MK2) as u8));
            }
            Kind::Storage => self.add_storage(pos),
            Kind::Process => {
                if let Some(spec) = process::spec(block) {
                    let mut p = Processor { dir: facing % 4, ..Processor::new(pos, spec, tier) };
                    p.sink_well(&mut self.deposits, world);
                    add_to(&mut self.processors, p, at, Slot::Process);
                }
            }
            Kind::Router => {
                add_to(&mut self.routers, Router::new(pos, facing, RouterKind::of(block)), at, Slot::Router)
            }
            Kind::Generator => add_to(&mut self.generators, Generator::new(pos), at, Slot::Generator),
            Kind::Pole => {
                let tier = match block {
                    CABLE => pole::CABLE_TIER,
                    LONG_POLE => pole::LONG_TIER,
                    _ => 0,
                };
                let wall = if block == POLE {
                    pole_mount::wall_for(|p| SOLID[world.block_anywhere_or_generate(p) as usize], pos, against)
                } else {
                    0
                };
                add_to(&mut self.poles, Pole { pos, tier, wall }, at, Slot::Pole)
            }
            Kind::Lab => add_to(&mut self.labs, Lab::new(pos), at, Slot::Lab),
            Kind::Pipe => add_to(&mut self.pipework, Pipework::new(pos, block, facing), at, Slot::Pipe),
            Kind::Quarry => add_to(&mut self.quarries, Quarry::new(pos, facing), at, Slot::Quarry),
            Kind::Sensor => add_to(&mut self.sensors, Sensor::new(pos, facing), at, Slot::Sensor),
            Kind::Rail => add_to(&mut self.rails, Rail::new(pos, facing), at, Slot::Rail),
            Kind::Node => add_to(&mut self.nodes, Node { pos }, at, Slot::Node),
        }
        if tier > 0 {
            self.set_tier(pos, tier);
        }
        self.dirty = true;
        self.layout_gen = self.layout_gen.wrapping_add(1);
        if (block == POLE || block == LONG_POLE) && facing & pole::POLE_UNLINKED == 0 {
            self.hook_new_pole(pos, tier);
        }
    }

    #[cfg(test)]
    pub fn add_belt(&mut self, pos: IVec3, dir: u8) {
        self.add_shaped_belt(pos, dir, Shape::Flat, 0);
    }

    /// A belt at `pos`; `facing` is its heading, with a flat belt's bend in its upper bits (`belt_facing`).
    fn add_shaped_belt(&mut self, pos: IVec3, facing: u8, shape: Shape, tier: u8) {
        self.remove(pos);
        let mut belt = Belt::new(pos, facing, shape, tier);
        if shape == Shape::Flat {
            belt.bend = bend_of(facing);
        }
        add_to(&mut self.belts, belt, &mut self.at, Slot::Belt);
        self.dirty = true;
        self.layout_gen = self.layout_gen.wrapping_add(1);
    }

    pub fn add_miner(&mut self, pos: IVec3, drill: u8, deposit: Option<DepositKey>, tier: u8) {
        self.remove(pos);
        add_to(&mut self.miners, Miner::new(pos, drill, deposit, tier), &mut self.at, Slot::Miner);
        self.dirty = true;
    }

    pub fn add_storage(&mut self, pos: IVec3) {
        self.remove(pos);
        add_to(&mut self.storages, Storage::new(pos), &mut self.at, Slot::Storage);
        self.dirty = true;
    }

    /// Removes the machine at `pos`, returning whatever it was holding or carrying.
    pub fn remove(&mut self, pos: IVec3) -> Vec<Stack> {
        let Some(slot) = self.at.remove(&pos) else { return Vec::new() };
        self.dirty = true;
        self.layout_gen = self.layout_gen.wrapping_add(1);
        let at = &mut self.at;
        let mut contents = match slot {
            Slot::Belt(i) => swap_out(&mut self.belts, i, at, Slot::Belt),
            Slot::Miner(i) => swap_out(&mut self.miners, i, at, Slot::Miner),
            Slot::Storage(i) => swap_out(&mut self.storages, i, at, Slot::Storage),
            Slot::Process(i) => swap_out(&mut self.processors, i, at, Slot::Process),
            Slot::Router(i) => swap_out(&mut self.routers, i, at, Slot::Router),
            Slot::Generator(i) => swap_out(&mut self.generators, i, at, Slot::Generator),
            Slot::Pole(i) => swap_out(&mut self.poles, i, at, Slot::Pole),
            Slot::Lab(i) => swap_out(&mut self.labs, i, at, Slot::Lab),
            Slot::Pipe(i) => swap_out(&mut self.pipework, i, at, Slot::Pipe),
            Slot::Quarry(i) => swap_out(&mut self.quarries, i, at, Slot::Quarry),
            Slot::Sensor(i) => swap_out(&mut self.sensors, i, at, Slot::Sensor),
            Slot::Rail(i) => swap_out(&mut self.rails, i, at, Slot::Rail),
            Slot::Node(i) => swap_out(&mut self.nodes, i, at, Slot::Node),
        };
        self.prune_hooks();
        self.prune_tracks();
        self.prune_manual_off(pos);
        contents.extend(self.derail_at(pos));
        contents
    }
}

#[inline]
fn opposite(dir: u8) -> u8 {
    (dir + 2) % 4
}

/// Whole ticks in `seconds` (machine work is counted in ticks).
fn ticks(seconds: f64) -> u32 {
    (seconds * TICK_RATE as f64).round() as u32
}

#[cfg(test)]
mod tests;
