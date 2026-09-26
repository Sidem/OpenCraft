//! Co-op plumbing for the host page's session code (`web/src/net/`): choose a role, move action bytes,
//! body states and item views between peers. The logic lives in `net/` (lockstep: `mod.rs`, bodies:
//! `players.rs`, items: `items.rs`).

use wasm_bindgen::prelude::*;

use crate::net::Role;
use crate::sim::PlayerId;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Hosts a co-op session from now on: every action goes through the host's frames.
    pub fn start_host(&mut self) {
        self.role = Role::host();
    }

    /// Plays as player `local` in the host's world from this core's tick on. False for an id past 255.
    pub fn start_client(&mut self, local: u32) -> bool {
        let Ok(id) = u8::try_from(local) else { return false };
        self.become_client(PlayerId(id));
        true
    }

    /// A client playing as `local` from a host's `snapshot`. Throws a message a player can read if
    /// the bytes can't be loaded.
    pub fn from_snapshot(bytes: &[u8], local: u32, view_radius: u32) -> Result<Game, String> {
        let local = u8::try_from(local).map_err(|_| "The host sent a player id that doesn't exist.")?;
        Game::read_snapshot(bytes, PlayerId(local), view_radius)
    }

    /// Client: replaces the core with a fresh host `snapshot` (taken like a joiner's), keeping this
    /// player's body and view. Throws a message a player can read if the bytes can't be loaded.
    pub fn resync(&mut self, snapshot: &[u8]) -> Result<(), String> {
        self.resync_from(snapshot)
    }

    pub fn is_client(&self) -> bool {
        matches!(self.role, Role::Client(_))
    }

    /// Host: lets in a player whose browser keeps `key` (they get back what they had if they were
    /// here before) and returns their id. Send them `snapshot()` next, then every frame. Throws a
    /// message a player can read if the world is full.
    pub fn host_join(&mut self, key: u64) -> Result<u32, String> {
        let id = self.join(key).ok_or_else(|| "This world is full.".to_string())?;
        self.role.set_peer(id, true);
        Ok(id.0 as u32)
    }

    /// Host: a player left or lost the connection. Their things wait under their key.
    pub fn host_leave(&mut self, player: u32) {
        if let Ok(id) = u8::try_from(player) {
            self.leave(PlayerId(id));
            self.role.set_peer(PlayerId(id), false);
        }
    }

    /// Host: the world for a joiner, taken between frames (see `host_join`).
    pub fn snapshot(&self) -> Vec<u8> {
        self.snapshot_bytes()
    }

    /// Host: collects a peer's actions (bytes from its `take_outbox`). False if refused.
    pub fn host_stamp(&mut self, player: u32, bytes: &[u8]) -> bool {
        u8::try_from(player).is_ok_and(|id| self.role.host_stamp(PlayerId(id), bytes))
    }

    /// Host: the frames of every tick run since the last call, for every client.
    pub fn take_frames(&mut self) -> Vec<u8> {
        self.role.take_frames()
    }

    /// Host and client: (tick, state hash) pairs, flat, noted every `CHECKSUM_TICKS` since the last call.
    /// The client compares its own with the host's.
    pub fn take_checksums(&mut self) -> Vec<u64> {
        self.role.take_checksums()
    }

    /// Client: the local player's actions since the last call, for the host.
    pub fn take_outbox(&mut self) -> Vec<u8> {
        self.role.take_outbox()
    }

    /// Client: queues frames from the host. False if they are damaged or out of order.
    pub fn push_frames(&mut self, bytes: &[u8]) -> bool {
        self.role.push_frames(&mut self.sim, bytes)
    }

    /// The player this game plays as.
    pub fn local_player(&self) -> u32 {
        self.local.0 as u32
    }

    /// Host: every body's latest state, for every client; client: its own, for the host. Empty when
    /// nothing new was noted (20 times a second).
    pub fn take_states(&mut self) -> Vec<u8> {
        self.role.take_states()
    }

    /// Host: a peer's body state (bytes from its `take_states`). False if refused.
    pub fn host_state(&mut self, player: u32, bytes: &[u8]) -> bool {
        u8::try_from(player).is_ok_and(|id| self.host_state_of(PlayerId(id), bytes))
    }

    /// Client: every body's state from the host. False if damaged.
    pub fn push_states(&mut self, bytes: &[u8]) -> bool {
        self.push_body_states(bytes)
    }

    /// Host: the loose items near peer `player`, for that peer only. Empty when nothing new was noted
    /// (10 times a second).
    pub fn take_item_view(&mut self, player: u32) -> Vec<u8> {
        u8::try_from(player).map_or_else(|_| Vec::new(), |id| self.item_view_for(PlayerId(id)))
    }

    /// Client: the host's items near this player. False if damaged.
    pub fn push_items(&mut self, bytes: &[u8]) -> bool {
        self.push_item_view(bytes)
    }

    /// Ticks the core has run.
    pub fn core_tick(&self) -> f64 {
        self.sim.tick as f64
    }

    /// Ticks the host has sent frames for (`core_tick` outside a client).
    pub fn confirmed_tick(&self) -> f64 {
        self.role.confirmed(self.sim.tick) as f64
    }
}
