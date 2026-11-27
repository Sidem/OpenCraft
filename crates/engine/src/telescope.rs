//! The telescope (the local player's hands and view, presentation only): with one in the left hand (`left_hand.rs`), `G`
//! toggles looking through it, the wheel steps its zoom (`ZOOMS`) and left-click pins the ground under the crosshair.
//! While looking, every pin of the world is a coloured column of light (`beacons.rs`), and a ruin whose ground the
//! crosshair meets goes on the map (`Known::remember_ruin`). Looking is first person only.
//!
//! Invariants: nothing here touches the core. The aim is a query over the loaded blocks and the generator
//! (`ground.rs`); a pin is a request for the host (`take_pin_request`: `[x, y, z]`, the ground's top block), which keeps
//! the pins in the browser (`ui/pins.ts`) and hands them back (`set_scope_pins`), so co-op peers and saves know nothing
//! of them. The zoom the host draws (`scope_zoom`) is eased here, so the view never jumps: it follows the zoom step at
//! a capped rate (`ZOOM_EASE`, `ZOOM_MAX_RATE`). Edge-detected clicks (a held button acts once). To add a zoom step:
//! a number in `ZOOMS`.

mod beacons;
pub(crate) use beacons::push_beacon;
pub(crate) mod ground;

use crate::item::TELESCOPE;
use crate::math::Vec3;
use crate::Game;

use ground::Aim;

/// The zoom steps, as a multiple of the normal view, and the one a telescope starts at.
const ZOOMS: [f32; 5] = [2.0, 3.0, 4.0, 6.0, 8.0];
const START_STEP: usize = 1;
/// The view eases towards its zoom at this rate (per second, of the remaining distance in log zoom), and never faster
/// than `ZOOM_MAX_RATE` (log zoom per second: 1.6 takes about a second to go from 1× to 4×).
const ZOOM_EASE: f32 = 5.0;
const ZOOM_MAX_RATE: f32 = 1.6;
/// Most pins kept (the host's list stops here too: `scope_pin_limit`).
pub(crate) const MAX_PINS: usize = 200;
/// A pin sent without a height (`i32::MIN`): its ground is looked up each frame.
pub(crate) const NO_HEIGHT: i32 = i32::MIN;
/// The crosshair's ground within this many blocks of a ruin's centre logs the ruin.
const RUIN_SPOT: i32 = 24;

/// One pin as the engine draws it: its column, the height of its ground when the player set it, and its colour.
struct PinMark {
    x: i32,
    z: i32,
    y: Option<i32>,
    rgb: u32,
}

/// The telescope's state: whether it is up, the zoom step, the eased zoom (log), the ground in the crosshair, the pin
/// the host has yet to take, the buttons' last state and the host's pins.
pub struct Telescope {
    on: bool,
    step: usize,
    shown: f32,
    aim: Option<Aim>,
    request: Option<[i32; 3]>,
    left_down: bool,
    mining_down: bool,
    pins: Vec<PinMark>,
}

impl Default for Telescope {
    fn default() -> Self {
        Telescope {
            on: false,
            step: START_STEP,
            shown: 0.0,
            aim: None,
            request: None,
            left_down: false,
            mining_down: false,
            pins: Vec::new(),
        }
    }
}

impl Game {
    /// Whether a telescope is in the left hand.
    pub(crate) fn telescope_held(&self) -> bool {
        self.left_item() == TELESCOPE
    }

    /// Whether the player is looking through a held telescope now.
    pub(crate) fn scope_active(&self) -> bool {
        self.scope.on && self.telescope_held() && !self.ghost_mode && self.first_person()
    }

    /// Runs the telescope's keys for one tick: `G` raises and lowers it, the left button pins while it is up.
    pub(crate) fn update_telescope(&mut self) {
        let (left, mining) = (self.left.using, self.mining);
        let (left_edge, mine_edge) = (left && !self.scope.left_down, mining && !self.scope.mining_down);
        (self.scope.left_down, self.scope.mining_down) = (left, mining);
        if !self.telescope_held() {
            self.scope.on = false;
            return;
        }
        if left_edge && self.first_person() {
            self.scope.on = !self.scope.on;
        }
        if mine_edge && self.scope_active() {
            if let Some(a) = self.scope.aim {
                self.scope.request = Some([a.x, a.y, a.z]);
                self.play(crate::sound::PLACE, crate::block::sound::METAL, self.body().eye(), 0.4);
            }
        }
    }

    /// The wheel, one notch: steps the zoom while looking through the telescope. False when it is not up (the wheel
    /// then changes the slot).
    pub(crate) fn scope_wheel(&mut self, delta: i32) -> bool {
        if !self.scope_active() {
            return false;
        }
        // Wheel up (negative) zooms in.
        let step = self.scope.step as i32 - delta.signum();
        self.scope.step = step.clamp(0, ZOOMS.len() as i32 - 1) as usize;
        true
    }

    /// Per frame while a telescope may be up: eases the zoom and finds the ground under the crosshair.
    pub(crate) fn update_scope(&mut self, dt: f64, eye: Vec3) {
        let active = self.scope_active();
        let to = if active { ZOOMS[self.scope.step].ln() } else { 0.0 };
        let dt = dt.max(0.0) as f32; // the host's first frame can be timed before the clock it started from
        let step =
            ((to - self.scope.shown) * (1.0 - (-ZOOM_EASE * dt).exp())).clamp(-ZOOM_MAX_RATE * dt, ZOOM_MAX_RATE * dt);
        self.scope.shown = if (to - self.scope.shown).abs() < 1e-3 { to } else { self.scope.shown + step };
        self.scope.aim =
            if active { ground::cast(&self.sim.world, eye, self.body().look_dir(), ground::RANGE) } else { None };
        let ruin = self.scope.aim.and_then(|a| self.sim.world.generator().ruin_near(a.x, a.z, RUIN_SPOT));
        if ruin.is_some_and(|(x, z)| self.minimap.known.remember_ruin(x, z)) {
            self.play(crate::sound::PLACE, crate::block::sound::STONE, self.body().eye(), 0.5);
        }
    }

    /// The HUD label while looking through the telescope; `None` otherwise.
    pub(crate) fn telescope_label(&self) -> Option<String> {
        if !self.scope_active() {
            return None;
        }
        let keys = crate::factory::hud_keys;
        let zoom = ZOOMS[self.scope.step] as u32;
        Some(match self.scope.aim {
            Some(a) => format!(
                "Telescope · {zoom}×\n{} blocks away\n{}",
                a.dist as i32,
                keys(&[("LMB", "Pin spot"), ("Wheel", "Zoom"), ("G", "Lower")])
            ),
            None => {
                format!("Telescope · {zoom}×\nLook lower to pin ground\n{}", keys(&[("Wheel", "Zoom"), ("G", "Lower")]))
            }
        })
    }

    /// The zoom to draw now (1 with no telescope up).
    pub(crate) fn scope_view_zoom(&self) -> f64 {
        f64::from(self.scope.shown.exp())
    }

    /// Lowers the telescope.
    pub(crate) fn lower_scope(&mut self) {
        self.scope.on = false;
    }

    /// The pin the player just made, once.
    pub(crate) fn take_scope_pin(&mut self) -> Option<[i32; 3]> {
        self.scope.request.take()
    }

    /// Replaces the pins drawn as columns: `[x, y, z, 0xRRGGBB]` each, `y` being [`NO_HEIGHT`] for a pin made before
    /// they had heights. Kept to [`MAX_PINS`].
    pub(crate) fn set_scope_marks(&mut self, flat: &[i32]) {
        self.scope.pins = flat
            .chunks_exact(4)
            .take(MAX_PINS)
            .map(|p| PinMark { x: p[0], z: p[2], y: (p[1] != NO_HEIGHT).then_some(p[1]), rgb: p[3] as u32 })
            .collect();
    }

    fn first_person(&self) -> bool {
        !self.strategy.active() && !self.third_person.active()
    }
}

#[cfg(test)]
mod tests;
