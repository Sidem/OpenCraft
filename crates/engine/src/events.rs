//! How the game reacts to the core's `SimEvent`s, every tick right after `Sim::step`: the authority
//! spawns loose items (block drops, throws from the right player's body), and the view plays sounds
//! (anyone's, heard from the local player's position) and queues toasts for the local player only.
//! Nothing here changes the core.
//!
//! To react to a new event: add its arm to `handle_sim_events`.

use crate::block;
use crate::item::ItemId;
use crate::math::Vec3;
use crate::sim::SimEvent;
use crate::sound;
use crate::Game;

/// Working miners are heard within this many blocks of the camera.
const MINER_SOUND_RANGE: f64 = 24.0;
/// Decaying leaves rustle softly within this many blocks.
const LEAF_SOUND_RANGE: f64 = 20.0;
/// Seconds before a broken block's drops can be picked up.
const DROP_PICKUP_DELAY: f32 = 0.25;

impl Game {
    pub(crate) fn handle_sim_events(&mut self) {
        let mut events = std::mem::take(&mut self.sim.events);
        let (mut picked_up, local) = (false, self.local);
        for &event in &events {
            match event {
                SimEvent::MinerWorking { pos } => {
                    let at = pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
                    if (at - self.body().eye()).length() < MINER_SOUND_RANGE {
                        self.play(sound::DIG, block::sound::STONE, at, 0.35);
                    }
                }
                SimEvent::BlockBroken { pos, block, .. } => {
                    self.play(sound::BREAK, block::def(block).sound, pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 1.0);
                }
                SimEvent::BlockPlaced { pos, block, .. } => {
                    self.play(sound::PLACE, block::def(block).sound, pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 1.0);
                }
                SimEvent::LeafDecayed { pos } => {
                    let at = pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
                    if (at - self.body().eye()).length() < LEAF_SOUND_RANGE {
                        self.play(sound::BREAK, block::sound::LEAVES, at, 0.2);
                    }
                }
                SimEvent::QuarryDug { pos, block } => {
                    let at = pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
                    if (at - self.body().eye()).length() < MINER_SOUND_RANGE {
                        self.play(sound::BREAK, block::def(block).sound, at, 0.7);
                    }
                }
                SimEvent::TreeGrew { pos } => {
                    let at = pos.as_vec3() + Vec3::new(0.5, 1.5, 0.5);
                    if (at - self.body().eye()).length() < LEAF_SOUND_RANGE {
                        self.play(sound::PLACE, block::sound::LEAVES, at, 0.6);
                    }
                }
                SimEvent::ToolWornOut { player, item } => {
                    if let Some(body) = self.bodies.get(player.0 as usize).and_then(Option::as_ref) {
                        self.play(sound::BREAK, block::sound::WOOD, body.eye(), 0.8);
                    }
                    if player == local {
                        self.pickups.push_back((item, 0)); // a count of 0 tells the HUD it wore out
                    }
                }
                SimEvent::Gained { player, item, count } if player == local => {
                    self.toast(item, count);
                    picked_up = true;
                }
                SimEvent::Crafted { player, item, count } if player == local => self.toast(item, count),
                SimEvent::Gained { .. } | SimEvent::Crafted { .. } => {}
                SimEvent::Dropped { pos, vel, item, count } => {
                    self.spawn_item(pos, vel, item, count, DROP_PICKUP_DELAY)
                }
                SimEvent::Thrown { player, item, count } => self.throw(player, item, count),
            }
        }
        if picked_up {
            self.sounds.push(sound::PICKUP, 0, Vec3::new(0.0, -0.6, 0.0), 1.0);
        }
        events.clear();
        self.sim.events = events;
    }

    /// Queues a pickup notification, merging with the previous one for the same item.
    fn toast(&mut self, item: ItemId, count: u32) {
        match self.pickups.back_mut() {
            Some((last, n)) if *last == item => *n += count,
            _ => self.pickups.push_back((item, count)),
        }
    }
}
