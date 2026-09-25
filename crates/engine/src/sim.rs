//! The deterministic core: the tick counter, the world's blocks (edits included), the factory with its
//! deposits, each player's inventory, and the core random stream.
//!
//! Invariants (DEV_PLAN section 3.4): `step` advances exactly one tick and depends only on this state.
//! It never takes frame time, the camera or `Sounds`, and never asks which chunks are loaded. Core code
//! reads and writes blocks through `World::block_anywhere_or_generate` / `set_block_anywhere`. `World`
//! also still holds the render cache (loaded chunks, meshes, streaming), which the core must not read.
//! Anything the presentation should react to leaves as a [`SimEvent`] in `events`; `Game::run_tick`
//! drains them every tick.
//!
//! To add core state: a field here (and, from step 1.6, in the save format). To tell the view about
//! something: a `SimEvent` variant and its arm in `Game::present_events` (lib.rs).

use crate::factory::Factory;
use crate::inventory::Inventory;
use crate::math::{hash2, IVec3, Rng};
use crate::world::World;

/// Something that happened in the core that the presentation may want to show or play.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SimEvent {
    /// A miner is drawing ore; `pos` is the block it drills. Sent every `MINER_PULSE_TICKS`.
    MinerWorking { pos: IVec3 },
}

/// A player's core state. The body (position, physics) is not core: it belongs to the authority.
#[derive(Default)]
pub struct PlayerCore {
    /// Slots, cursor stack and selected hotbar slot.
    pub inventory: Inventory,
}

pub struct Sim {
    /// Ticks run so far; game time is `tick as f64 * TICK`.
    pub tick: u64,
    pub world: World,
    pub factory: Factory,
    pub players: Vec<PlayerCore>,
    pub rng: Rng,
    /// Events from the ticks since the last drain.
    pub events: Vec<SimEvent>,
}

impl Sim {
    /// A fresh world with one player.
    pub fn new(seed: u32, view_radius: i32) -> Sim {
        Sim {
            tick: 0,
            world: World::new(seed, view_radius),
            factory: Factory::default(),
            players: vec![PlayerCore::default()],
            rng: Rng::new(hash2(seed, 17, 42) as u64),
            events: Vec::new(),
        }
    }

    /// Advances the core by one tick of `TICK` seconds.
    pub fn step(&mut self) {
        self.factory.update(&mut self.world, self.tick, &mut self.events);
        self.tick += 1;
    }
}
