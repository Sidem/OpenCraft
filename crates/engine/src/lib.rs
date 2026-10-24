//! OpenCraft engine: the whole simulation (world streaming, terrain generation, meshing, physics,
//! interaction, inventory, ore deposits and factory machines) compiled to WebAssembly.
//!
//! The JavaScript host only owns the platform: WebGL2, input and DOM UI. Bulk data (chunk meshes,
//! box instances, textures) never gets copied across the boundary; the host reads it straight out
//! of wasm linear memory through `*_ptr` / `*_len` accessors.
//!
//! This file holds the `Game` struct, its constructor and the fixed tick (`run_tick`); the per-frame `update` is in
//! `frame.rs`. `Game` wraps the deterministic core (`sim.rs`) with the authority (every player's
//! body, loose items: `authority.rs`) and the local player's view (hands, camera, sounds, instances).
//! The JS-facing API lives in `api/*.rs` (one `#[wasm_bindgen] impl Game` block per area); every method
//! there only forwards to a module, and acts for the local player. `Game` never edits the core
//! directly: it queues `Action`s (`act`), applied at the next tick (in co-op, through the host's
//! frames: `net/mod.rs`). The hands (mining, right-click,
//! footsteps) live in `interaction.rs`, reactions to core events in `events.rs`.
//! To add a wasm method: put it in the matching `api/` file (see docs/CODEMAP.md).
//!
//! Invariant: game state advances only in `run_tick`, by exactly [`TICK`] seconds, so the same inputs
//! give the same results at any frame rate. `update` turns frame time into whole ticks and does the
//! per-frame presentation work (streaming, the interpolated camera, box instances). Look direction is
//! the one input applied per frame, for responsiveness.

mod action;
mod analytics;
mod api;
mod authority;
mod avatars;
mod belt_line;
mod block;
mod blueprint;
mod bytes;
mod camera;
mod cargo_tools;
mod chunk;
mod crafting;
mod daytime;
mod deposits;
mod drone_view;
mod drones;
mod entities;
mod equipment;
mod events;
mod factory;
mod footprint_preview;
mod frame;
mod ghost_mode;
mod ghosts;
mod hand_waste;
mod helper_hands;
mod helpers;
mod hints;
mod hitbox;
mod interaction;
mod inventory;
mod item;
mod item_models;
mod item_select;
mod join;
mod lessons;
mod light;
mod math;
mod mesher;
mod minimap;
mod mode;
mod net;
mod noise;
mod opening;
mod orbit_fx;
mod ore_guide;
mod perks;
mod physics;
mod player;
mod pods;
mod power_tools;
mod prospect;
mod quarry_preview;
mod rail_tools;
mod raycast;
mod recipes;
mod research;
mod satellites;
mod save;
mod settlements;
mod sickle;
mod sim;
mod site_hands;
mod sound;
mod stall_lesson;
mod strategy;
mod surroundings;
mod survey;
mod telescope;
mod textures;
mod tools;
mod town_preview;
mod train_tools;
mod upgrade_aim;
mod world;
mod worldgen;

use std::collections::VecDeque;

use wasm_bindgen::prelude::*;

use action::Action;
use avatars::Avatars;
use belt_line::BeltLine;
use camera::{CrouchGlide, ThirdPerson};
use deposits::DepositState;
use entities::Items;
use inventory::Inventory;
use item::ItemId;
use math::{IVec3, Vec3};
use minimap::Minimap;
use net::Role;
use player::Player;
use power_tools::PowerTools;
use prospect::Prospect;
use rail_tools::RailTools;
use raycast::RayHit;
use sim::{PlayerId, Sim};
use sound::Sounds;
use world::MeshData;
use worldgen::WorldGen;

/// Simulation ticks per second.
pub const TICK_RATE: u32 = 60;
/// Length of one tick, in seconds.
pub const TICK: f64 = 1.0 / TICK_RATE as f64;

#[wasm_bindgen]
pub struct Game {
    /// The deterministic core: world blocks, factory, deposits, inventories, tick.
    sim: Sim,
    /// The player this game shows and takes input for (0 in single-player).
    local: PlayerId,
    /// Solo, or host or client of a co-op session: where actions go (net/mod.rs).
    role: Role,
    /// Every player's body, indexed by `PlayerId` (authority, see authority.rs).
    bodies: Vec<Option<Player>>,
    items: Items,
    /// Other players as drawn here (avatars.rs).
    avatars: Avatars,
    spawn: Vec3,
    /// Frame time not yet run as ticks, in seconds.
    accumulator: f64,
    /// Player eye at the start of the latest tick, and this frame's camera, interpolated between it
    /// and the current eye (presentation only).
    prev_eye: Vec3,
    render_eye: Vec3,
    /// Eases the eye between standing and crouching height (camera.rs).
    crouch_glide: CrouchGlide,
    /// The optional third-person camera and the avatar it shows (camera.rs).
    third_person: ThirdPerson,
    strategy: strategy::Strategy,
    /// The local player's hands (interaction.rs). Other players run their own on their machines.
    target: Option<RayHit>,
    /// Where in the target's cell its model is (lowest corner, size), for a pole or belt; None for the whole cell.
    target_fit: Option<[f32; 6]>,
    /// The belt item the crosshair is on, if the target is a belt (`factory/belt_items.rs`).
    target_item: Option<factory::ItemAim>,
    /// The belt item a click selected (`item_select.rs`).
    selected_item: Option<item_select::Selection>,
    mining: bool,
    mine_block: Option<IVec3>,
    mine_progress: f32,
    mine_cooldown: f32,
    using: bool,
    use_cooldown: f32,
    /// A machine the player right-clicked whose panel the host should open (`take_panel_request`).
    panel_request: Option<IVec3>,
    /// Ghost mode (B): the use button plants ghosts instead of building (`ghost_mode.rs`).
    ghost_mode: bool,
    /// Whether the jetpack thrust was last sent on (`helper_hands.rs`).
    jet_sent: bool,
    /// Whether the hover was last sent on, and the pole the pack was last sent to charge from (`helper_hands.rs`).
    hover_sent: bool,
    charge_sent: Option<IVec3>,
    /// Whether the hover pack was last heard charging, for the power-up and power-down sounds (`helper_hands.rs`).
    charge_heard: bool,
    /// Whether the three above have been taken from the core yet (a loaded save may be mid-thrust).
    hands_synced: bool,
    /// The first port of the cargo route being set, and whether the use button was down (`cargo_tools.rs`).
    cargo_from: Option<IVec3>,
    cargo_down: bool,
    /// The player's blueprints and what the hands are doing with them (`blueprint/`).
    library: blueprint::Library,
    /// The planner's corner and the request for the site panel (`site_hands.rs`).
    planner: site_hands::Planner,
    /// The telescope's zoom, aim and the pins it draws (`telescope.rs`).
    scope: telescope::Telescope,
    dig_timer: f32,
    step_distance: f64,
    sounds: Sounds,
    textures: Vec<u8>,
    minimap: Minimap,
    /// Box instances (dropped items, belt items, machine parts) for the current frame.
    instances: Vec<f32>,
    pickups: VecDeque<(ItemId, u32)>,
    /// Ore the local player wasted by hand mining this session (`hand_waste.rs`).
    waste: hand_waste::HandWaste,
    /// The machine stall lesson being watched for the local player (`stall_lesson.rs`); never saved.
    stall: stall_lesson::StallWatch,
    /// Launch and landing effects, and launches the HUD has yet to toast (`orbit_fx.rs`).
    orbit_fx: orbit_fx::OrbitFx,
    cur_pickup: (ItemId, u32),
    cur_mesh: Option<MeshData>,
    cur_event_pos: IVec3,
    /// Figures of the last untracked deposit `target_detail` showed, so looking stays cheap
    /// without making the core track it.
    surveyed: Option<DepositState>,
    /// The scanner's and core drill's latest reading (prospect.rs).
    prospect: Prospect,
    /// A belt line being dragged out or built (belt_line.rs).
    line: BeltLine,
    /// Quarter turns R added to a held quarry's or multi-block machine's facing (quarry_preview.rs).
    place_turn: u8,
    /// Pole and cable placing (`power_tools.rs`).
    tools: PowerTools,
    /// Track laying (`rail_tools.rs`).
    rails: RailTools,
    /// Power and production history and machine efficiency (`analytics/`; presentation, not saved).
    analytics: analytics::Analytics,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, view_radius: u32) -> Game {
        Game::with_generator(WorldGen::new(seed), view_radius)
    }
}

impl Game {
    pub(crate) fn with_generator(generator: WorldGen, view_radius: u32) -> Game {
        let sim = Sim::with_generator(generator, view_radius as i32);
        let spawn = Vec3::new(0.5, sim.world.generator().height_at(0, 0) as f64 + 1.0, 0.5);
        let player = Player::new(spawn);
        let eye = player.eye();
        let textures = textures::generate();
        Game {
            sim,
            local: PlayerId(0),
            role: Role::Solo,
            bodies: vec![Some(player)],
            items: Items::default(),
            avatars: Avatars::default(),
            spawn,
            accumulator: 0.0,
            prev_eye: eye,
            render_eye: eye,
            crouch_glide: CrouchGlide::default(),
            third_person: ThirdPerson::default(),
            strategy: strategy::Strategy::default(),
            target: None,
            target_fit: None,
            target_item: None,
            selected_item: None,
            mining: false,
            mine_block: None,
            mine_progress: 0.0,
            mine_cooldown: 0.0,
            using: false,
            use_cooldown: 0.0,
            panel_request: None,
            ghost_mode: false,
            jet_sent: false,
            hover_sent: false,
            charge_sent: None,
            charge_heard: false,
            hands_synced: false,
            cargo_from: None,
            cargo_down: false,
            library: blueprint::Library::default(),
            planner: site_hands::Planner::default(),
            scope: telescope::Telescope::default(),
            dig_timer: 0.0,
            step_distance: 0.0,
            sounds: Sounds::default(),
            minimap: Minimap::new(&textures),
            textures,
            instances: Vec::new(),
            pickups: VecDeque::new(),
            waste: hand_waste::HandWaste::default(),
            stall: stall_lesson::StallWatch::default(),
            orbit_fx: orbit_fx::OrbitFx::default(),
            cur_pickup: (ItemId::NONE, 0),
            cur_mesh: None,
            cur_event_pos: IVec3::ZERO,
            surveyed: None,
            prospect: Prospect::default(),
            line: BeltLine::default(),
            tools: PowerTools::default(),
            rails: RailTools::default(),
            place_turn: 0,
            analytics: analytics::Analytics::default(),
        }
    }
}

impl Game {
    /// Advances the game by one tick of exactly [`TICK`] seconds: every body, the local player's hands,
    /// loose items, then the core (a co-op client only through the tick the host confirmed), then what
    /// co-op peers see of the bodies and items (`net_tick`).
    fn run_tick(&mut self) {
        self.prev_eye = self.body().eye();
        let feet = self.body().pos;
        self.step_navigation();
        self.update_jetpack();
        self.step_bodies();
        self.update_movement_sounds(feet);
        self.update_target();
        self.update_mining(TICK as f32);
        self.update_placing(TICK as f32);
        self.body_mut().gesture = u8::from(self.mining) | (u8::from(self.using) << 1);

        self.step_items();
        if self.role.may_step(self.sim.tick) {
            self.step_core();
        }
        self.net_tick();
    }

    /// One core tick (`Sim::step` applies its actions), the host's frame for it, then its events and the analytics.
    fn step_core(&mut self) {
        self.sim.step();
        self.role.end_tick(&mut self.sim);
        self.handle_sim_events();
        self.analytics.record(&self.sim.factory);
        self.watch_stalls();
    }

    /// Queues an action by the local player for the coming tick.
    fn act(&mut self, action: Action) {
        self.act_as(self.local, action);
    }

    /// Queues an action by any player (the authority's pickups, joins, tests): for the coming tick
    /// solo; in co-op through the host (net/mod.rs).
    fn act_as(&mut self, player: PlayerId, action: Action) {
        self.role.route(&mut self.sim, self.local, player, action);
    }

    /// The local player's body, which always exists.
    fn body(&self) -> &Player {
        self.bodies[self.local.0 as usize].as_ref().expect("local body")
    }

    fn body_mut(&mut self) -> &mut Player {
        self.bodies[self.local.0 as usize].as_mut().expect("local body")
    }

    /// The local player's inventory; empty until a co-op client's `Join` applies.
    fn inventory(&self) -> &Inventory {
        self.sim.player(self.local).map_or(&Inventory::EMPTY, |p| &p.inventory)
    }
}

#[cfg(test)]
mod tests;
