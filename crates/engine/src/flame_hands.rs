//! The flamethrower (FIRE2, the local player's hands, presentation only): holding the use button (right-click) with
//! a flamethrower in the selected slot burns a steady continuous torch flame and sends `Action::Flame` every
//! `FLAME_SECONDS` along the player's aim ray. Liquid fuel (naphtha or diesel barrels from the refinery or cracker —
//! never coal or charcoal) is checked and consumed in the core (`sim/fire/`); when empty, the hands play `DENIED` once
//! and send nothing. No camera shake or FOV effects (motion-sickness rule).
//!
//! To tune: `FLAME_SECONDS` here; range, ray count and fuel ticks per barrel live in `sim/fire/mod.rs`.

use crate::action::Action;
use crate::item::FLAMETHROWER;
use crate::math::Vec3;
use crate::sim::fire::{flame_fuel_barrels, FLAME_TICKS_PER_BARREL};
use crate::sound;
use crate::Game;

/// Seconds between core flame ray ticks while the use button is held (visual torch plume streams continuously).
pub const FLAME_SECONDS: f32 = 0.20;

impl Game {
    /// Whether a flamethrower is in the selected hotbar slot.
    pub(crate) fn flamethrower_held(&self) -> bool {
        self.inventory().selected_stack().item == FLAMETHROWER
    }

    /// True while the local player is actively streaming a continuous flamethrower torch flame.
    pub(crate) fn flamethrower_burning(&self) -> bool {
        self.using && self.flamethrower_held() && self.flame_fuel_available() > 0
    }

    /// Runs the use button for a held flamethrower for one tick. Returns true when it handles the button.
    pub(crate) fn update_flamethrower(&mut self, dt: f32) -> bool {
        if !self.using || !self.flamethrower_held() {
            return false;
        }
        let fuel = self.flame_fuel_available();
        self.use_cooldown -= dt;
        if self.use_cooldown > 0.0 {
            return true;
        }
        if fuel == 0 {
            if self.refuse_cooldown <= 0.0 {
                self.sounds.push(sound::DENIED, 0, Vec3::new(0.0, -0.2, 0.0), 0.7);
                self.refuse_cooldown = 0.4;
            }
            self.use_cooldown = FLAME_SECONDS;
            return true;
        }
        self.use_cooldown = FLAME_SECONDS;
        let origin = self.body().eye();
        let dir = self.body().look_dir();
        self.act(Action::Flame { origin, dir });
        true
    }

    /// The HUD label with a flamethrower in hand; `None` otherwise.
    pub(crate) fn flamethrower_label(&self) -> Option<String> {
        if !self.flamethrower_held() {
            return None;
        }
        let keys = crate::factory::hud_keys;
        let barrels = flame_fuel_barrels(self.inventory());
        let tank = self.sim.fire.tank(self.local);
        if barrels == 0 && tank == 0 {
            return Some("Flamethrower\nNo fuel · carry naphtha or diesel barrels".into());
        }
        let total = u32::from(tank) + barrels * u32::from(FLAME_TICKS_PER_BARREL);
        let secs = ((total as f32) * FLAME_SECONDS).ceil() as u32;
        Some(format!(
            "Flamethrower\n{secs} s of fuel ({barrels} naphtha/diesel) · weed torch & tree burner\n{}",
            keys(&[("Hold RMB", "Torch")])
        ))
    }

    /// Total fuel ticks available from the canister plus naphtha/diesel barrels in the pack.
    fn flame_fuel_available(&self) -> u32 {
        let tank = u32::from(self.sim.fire.tank(self.local));
        let pack = flame_fuel_barrels(self.inventory()) * u32::from(FLAME_TICKS_PER_BARREL);
        tank + pack
    }
}
