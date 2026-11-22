//! Local player's hands for the Paint Sprayer (`L-COLOUR`, presentation only — acts only by queuing
//! `Action::PaintFace`): cycling pigment swatches (`Shift+R` / `Shift+Wheel`), eyedropper (`R`), instant
//! hold-to-spray across paintable surfaces, persistent HUD tool card, live aimed-face target reticle, and
//! ballistic coloured paint-mist emission straight from the rendered brass nozzle (`avatars.muzzle_of`).

use crate::action::Action;
use crate::block::{self, tex};
use crate::colour;
use crate::factory::{self, Axis};
use crate::item::PAINT_SPRAYER;
use crate::math::{hash3, IVec3, Vec3};
use crate::mode::Mode;
use crate::sound;
use crate::Game;

/// Cap on simultaneous in-flight paint-mist droplets.
const MAX_DROPS: usize = 96;
/// Droplets emitted per second while spraying.
const EMIT_RATE: f32 = 160.0;
/// Seconds to skip re-queuing the exact same `(pos, face, colour)` while its tick action applies.
const SAME_FACE_DEBOUNCE: f32 = 0.05;

#[derive(Clone, Copy)]
struct SprayDrop {
    pos: Vec3,
    vel: Vec3,
    age: f32,
    max_age: f32,
    size: f32,
    rgb: u32,
    splashed: bool,
}

/// Presentation-only state for the local player's Paint Sprayer: in-flight mist stream + debounce.
#[derive(Default)]
pub(crate) struct PaintSpray {
    drops: Vec<SprayDrop>,
    emit_acc: f32,
    seq: u32,
    last_sent: Option<(IVec3, u8, u8)>,
    last_sent_ttl: f32,
}

/// Packed 24-bit RGB of swatch `col` (`0` = pale soap-mist for Strip Paint).
fn swatch_rgb(col: u8) -> u32 {
    if col == colour::NONE {
        return 0xd8eef8;
    }
    let [r, g, b] = colour::PALETTE[col as usize];
    ((r * 255.0).round() as u32) << 16 | ((g * 255.0).round() as u32) << 8 | ((b * 255.0).round() as u32)
}

impl Game {
    /// Whether the selected hotbar slot holds the Paint Sprayer.
    pub(crate) fn holds_paint_sprayer(&self) -> bool {
        let s = self.inventory().selected_stack();
        !s.is_empty() && s.item == PAINT_SPRAYER
    }

    /// True while the local player is actively spraying paint (or stripping paint) this frame.
    pub(crate) fn paint_sprayer_active(&self) -> bool {
        if !self.using || !self.holds_paint_sprayer() {
            return false;
        }
        let creative = self.sim.mode == Mode::Creative;
        self.sim.paint.can_paint(self.held_paint_colour, self.inventory(), creative)
    }

    /// Steps the Paint Sprayer swatch (`0 = Strip Paint`, `1..=16 = Red..Navy`) when holding a Paint Sprayer.
    pub(crate) fn cycle_paint_colour(&mut self, delta: i32) -> bool {
        if !self.holds_paint_sprayer() {
            return false;
        }
        let n = (colour::COUNT + 1) as i32;
        self.held_paint_colour = (i32::from(self.held_paint_colour) + delta).rem_euclid(n) as u8;
        self.paint_spray.last_sent = None;
        self.play(sound::PLACE, block::sound::METAL, self.body().eye(), 0.25);
        true
    }

    /// Eyedropper (`R` while holding Paint Sprayer): picks up the paint colour of the aimed surface.
    pub(crate) fn eyedrop_paint_colour(&mut self) -> bool {
        if !self.holds_paint_sprayer() {
            return false;
        }
        let Some(hit) = self.target else { return true };
        if !colour::is_paintable(hit.id) {
            return true;
        }
        let Some(face) = colour::face_of_normal(hit.normal) else { return true };
        self.held_paint_colour = self.sim.world.get_face_paint(hit.block, face);
        self.paint_spray.last_sent = None;
        self.play(sound::PLACE, block::sound::METAL, self.body().eye(), 0.35);
        true
    }

    /// Handles the use button while holding a Paint Sprayer (returns `true` to claim right-click).
    /// Paints any newly aimed paintable face immediately with zero artificial sweep delay.
    pub(crate) fn update_paint_sprayer(&mut self, dt: f32) -> bool {
        self.paint_cooldown = (self.paint_cooldown - dt).max(0.0);
        self.paint_spray.last_sent_ttl = (self.paint_spray.last_sent_ttl - dt).max(0.0);
        if self.paint_spray.last_sent_ttl <= 0.0 {
            self.paint_spray.last_sent = None;
        }
        if !self.holds_paint_sprayer() {
            return false;
        }
        if !self.using {
            return true;
        }
        let col = self.held_paint_colour;
        let creative = self.sim.mode == Mode::Creative;
        if !self.sim.paint.can_paint(col, self.inventory(), creative) {
            if self.refuse_cooldown <= 0.0 {
                self.sounds.push(sound::DENIED, 0, Vec3::new(0.0, -0.2, 0.0), 0.7);
                self.refuse_cooldown = 0.4;
            }
            return true;
        }
        let Some(hit) = self.target else { return true };
        if !colour::is_paintable(hit.id) {
            return true;
        }
        let Some(face) = colour::face_of_normal(hit.normal) else { return true };
        let face_u8 = face as u8;
        if self.sim.world.get_face_paint(hit.block, face) == col {
            return true;
        }
        if self.paint_spray.last_sent == Some((hit.block, face_u8, col)) {
            return true;
        }
        self.act(Action::PaintFace { pos: hit.block, face: face_u8, colour: col });
        self.paint_spray.last_sent = Some((hit.block, face_u8, col));
        self.paint_spray.last_sent_ttl = SAME_FACE_DEBOUNCE;
        if self.paint_cooldown <= 0.0 {
            self.play(sound::PLACE, block::sound::SAND, hit.block.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 0.3);
            self.paint_cooldown = 0.09;
        }
        true
    }

    /// Persistent on-screen tool HUD card while holding the Paint Sprayer (`api/hud.rs`), matching `flamethrower_label`.
    pub(crate) fn paint_sprayer_label(&self) -> Option<String> {
        if !self.holds_paint_sprayer() {
            return None;
        }
        let keys = factory::hud_keys;
        let col = self.held_paint_colour;
        let name = colour::NAMES[col as usize];
        let status = if col == colour::NONE {
            "Removes paint from one surface (free)".to_string()
        } else if self.sim.mode == Mode::Creative {
            "Creative · free · sprays one surface at a time".to_string()
        } else {
            let rem = self.sim.paint.remaining(col);
            let stock = colour::pigment_of(col).map_or(0, |p| self.inventory().count(p));
            if rem > 0 || stock > 0 {
                format!("{rem}/{} surfaces left · {stock} {name} Pigment in pack", colour::FACES_PER_PIGMENT)
            } else {
                format!("No {name} Pigment in pack · craft pigment or switch colour")
            }
        };
        Some(format!(
            "Paint Sprayer · {name}\n{status}\n{}",
            keys(&[("Hold RMB", "Spray"), ("Shift+Wheel / Shift+R", "Colour"), ("R", "Eyedropper")])
        ))
    }

    /// Held-item status subtitle when holding the Paint Sprayer (empty otherwise).
    pub(crate) fn held_paint_status(&self) -> String {
        if !self.holds_paint_sprayer() {
            return String::new();
        }
        let col = self.held_paint_colour;
        let name = colour::NAMES[col as usize];
        if col == colour::NONE {
            return name.to_string();
        }
        if self.sim.mode == Mode::Creative {
            return format!("{name} · Creative");
        }
        let rem = self.sim.paint.remaining(col);
        let stock = colour::pigment_of(col).map_or(0, |p| self.inventory().count(p));
        if rem > 0 || stock > 0 {
            format!("{name} ({rem}/{} · {stock} pigment)", colour::FACES_PER_PIGMENT)
        } else {
            format!("{name} (no pigment)")
        }
    }

    /// CSS hex colour of the active Paint Sprayer swatch (empty when not holding it or in Strip mode).
    pub(crate) fn held_paint_hex_str(&self) -> &'static str {
        if !self.holds_paint_sprayer() {
            return "";
        }
        colour::HEX[self.held_paint_colour as usize]
    }

    /// Per-frame visual feedback for the Paint Sprayer (`frame.rs`):
    /// 1. Live coloured target frame on the aimed paintable surface so you see exactly which face gets painted.
    /// 2. Ballistic aerosol paint-mist stream emitted straight from the rendered brass nozzle, splashing on impact.
    pub(crate) fn write_paint_spray(&mut self, dt: f64, eye: Vec3, _time: f64) {
        let dt_f = (dt as f32).min(0.1);
        let rgb = swatch_rgb(self.held_paint_colour);
        let active = self.paint_sprayer_active();

        // 1. Live aimed-face target frame on any paintable block surface.
        if self.holds_paint_sprayer() {
            if let Some(hit) = self.target.filter(|h| colour::is_paintable(h.id)) {
                if let Some(face) = colour::face_of_normal(hit.normal) {
                    push_face_frame(&mut self.instances, hit.block, face, eye, rgb);
                }
            }
        }

        // 2. Emit fresh aerosol droplets from the held gun's brass nozzle while spraying.
        if active {
            let look = self.body().look_dir();
            let (tip, nozzle_dir) = self
                .avatars
                .muzzle_of(self.local.0)
                .map(|(t, d, _)| (t, d))
                .unwrap_or_else(|| (eye + look * 0.45, look));
            // Aim droplets straight from the nozzle toward the exact crosshair impact point.
            let aim_point =
                self.target.map(|h| self.hit_surface(h)).unwrap_or_else(|| eye + look * crate::interaction::REACH);
            let to_hit = aim_point - tip;
            let dist = to_hit.length().max(0.25);
            let aim_dir = if dist > 0.35 { to_hit * (1.0 / dist) } else { nozzle_dir };
            let up = if aim_dir.y.abs() < 0.92 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
            let right = norm(cross(aim_dir, up));
            let ortho_up = norm(cross(right, aim_dir));

            self.paint_spray.emit_acc = (self.paint_spray.emit_acc + EMIT_RATE * dt_f).min(12.0);
            while self.paint_spray.emit_acc >= 1.0 {
                self.paint_spray.emit_acc -= 1.0;
                self.paint_spray.seq = self.paint_spray.seq.wrapping_add(1);
                let s = self.paint_spray.seq;
                let r1 = (hash3(s, 17, 3, 1) as f64 / u32::MAX as f64) * 2.0 - 1.0;
                let r2 = (hash3(s, 31, 7, 2) as f64 / u32::MAX as f64) * 2.0 - 1.0;
                let r3 = hash3(s, 53, 11, 3) as f64 / u32::MAX as f64;
                // Tight aerosol spray fan (~6° half-angle) expanding into a soft coat at the wall.
                let spread = 0.095;
                let dir = norm(aim_dir + right * (r1 * spread) + ortho_up * (r2 * spread));
                let speed = 13.0 + r3 * 4.5;
                let max_age = ((dist / speed) as f32 + 0.14).clamp(0.14, 0.42);
                if self.paint_spray.drops.len() >= MAX_DROPS {
                    self.paint_spray.drops.remove(0);
                }
                self.paint_spray.drops.push(SprayDrop {
                    pos: tip + aim_dir * 0.03,
                    vel: dir * speed,
                    age: 0.0,
                    max_age,
                    size: 0.032 + (r3 as f32) * 0.022,
                    rgb,
                    splashed: false,
                });
            }
        } else {
            self.paint_spray.emit_acc = 0.0;
        }

        // 3. Advance & draw all live paint-mist droplets (splashing outward along the block face on contact).
        if self.paint_spray.drops.is_empty() {
            return;
        }
        let world = &self.sim.world;
        let dt64 = f64::from(dt_f);
        let mut i = 0;
        while i < self.paint_spray.drops.len() {
            let d = &mut self.paint_spray.drops[i];
            d.age += dt_f;
            if d.age >= d.max_age {
                self.paint_spray.drops.swap_remove(i);
                continue;
            }
            let next = d.pos + d.vel * dt64;
            let cell = next.floor();
            let hit_solid = world.get_block(cell).is_some_and(|b| block::SOLID[b as usize]);
            if hit_solid && !d.splashed {
                // Splash: stop at the surface and fan gently along the wall like wet paint mist.
                let prev_cell = d.pos.floor();
                let n = (prev_cell - cell).as_vec3();
                let n_len = n.length();
                if n_len > 0.5 {
                    let normal = n * (1.0 / n_len);
                    let vn = dot(d.vel, normal);
                    d.vel = (d.vel - normal * vn) * 0.35 + normal * 0.25;
                } else {
                    d.vel = d.vel * 0.15;
                }
                d.splashed = true;
                d.max_age = d.max_age.min(d.age + 0.14);
            } else {
                d.pos = next;
                d.vel = d.vel * (1.0 - 2.6 * dt64).max(0.2);
            }
            let u = (d.age / d.max_age).clamp(0.0, 1.0);
            let scale = if d.splashed { d.size * (1.2 + u * 1.4) * (1.0 - u * 0.65) } else { d.size * (0.7 + u * 1.1) };
            let sz = scale.max(0.014);
            let rel = d.pos - eye;
            factory::push_box(&mut self.instances, rel, 0.0, [sz, sz, sz], 0.0, [tex::MAT_METAL; 3], false);
            factory::make_round(&mut self.instances, Axis::Up, 1.0);
            factory::mark_beacon_tinted(&mut self.instances, d.rgb);
            i += 1;
        }
    }
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}

fn norm(v: Vec3) -> Vec3 {
    let l = v.length();
    if l > 1e-9 {
        v * (1.0 / l)
    } else {
        Vec3::new(0.0, 0.0, -1.0)
    }
}

/// Draws a thin coloured 4-edge reticle just in front of `face` (`0..6`) of `cell`, showing which surface will be painted.
fn push_face_frame(out: &mut Vec<f32>, cell: IVec3, face: usize, eye: Vec3, rgb: u32) {
    let base = cell.as_vec3() - eye + Vec3::new(0.5, 0.5, 0.5);
    let off = 0.508;
    let bar = 0.022f32;
    let span = 0.96f32;
    let half = 0.47;
    // Four edges per face axis (axis 0 = X, 1 = Y, 2 = Z).
    let (n, u, v) = match face {
        0 => (Vec3::new(off, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
        1 => (Vec3::new(-off, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
        2 => (Vec3::new(0.0, off, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
        3 => (Vec3::new(0.0, -off, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
        4 => (Vec3::new(0.0, 0.0, off), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        _ => (Vec3::new(0.0, 0.0, -off), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
    };
    let size_along = |axis: Vec3| -> [f32; 3] {
        [
            if axis.x.abs() > 0.5 { span } else { bar },
            if axis.y.abs() > 0.5 { span } else { bar },
            if axis.z.abs() > 0.5 { span } else { bar },
        ]
    };
    let center = base + n;
    for (offset, dir) in [(v * half, u), (v * -half, u), (u * half, v), (u * -half, v)] {
        factory::push_box(out, center + offset, 0.0, size_along(dir), 0.0, [tex::MAT_METAL; 3], false);
        factory::mark_beacon_tinted(out, rgb);
    }
}
