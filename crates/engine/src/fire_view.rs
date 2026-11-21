//! Fire presentation (FIRE3, presentation only): fills the GPU fire-emitter instance buffer for burning trees
//! (`Blaze`, fitted to each tree's measured crown and trunk), localized ground embers (`Front`), and ballistic
//! travelling blowtorch flame packets (`torch.rs`). Also emits smouldering tree smoke via `factory::smoke`, warm
//! glow halos, and torch-bright dynamic point lights (`avatars.lights`, `held_light.rs`) so flames illuminate dark
//! ground, trees, machines and water with zero chunk remeshing.
//!
//! Cost contract: zero work when no fire or torch puff is active; at most `MAX_SHOWN` nearest blazes, `MAX_FRONT_SHOWN`
//! nearest scorches, and `torch::MAX_PUFFS` travelling puffs; all flame animation runs in GLSL (`web/src/render/fire.ts`).

mod torch;

use std::f64::consts::TAU;

use torch::{TorchEmitter, TorchPuff};

use crate::factory::smoke;
use crate::halos::{HALO_FLOATS, HALO_RANGE};
use crate::held_light::{self, FLAME_REACH};
use crate::math::{hash3, sort_small_by_key, Vec3};
use crate::sim::fire::Stage;
use crate::Game;

/// Floats per fire emitter instance: x, y, z, width, height, seed, intensity, kind.
pub const FIRE_FLOATS: usize = 8;
/// Most burning trees drawn at once (nearest first).
pub const MAX_SHOWN: usize = 48;
/// Most localized ground scorch emitters drawn at once (nearest first).
pub const MAX_FRONT_SHOWN: usize = 128;
/// Fire emitters beyond this distance (blocks) are skipped.
pub const FIRE_RANGE: f64 = 160.0;
/// Max halos appended by fire per frame so lamps stay visible.
pub(super) const MAX_FIRE_HALOS: usize = 96;

#[derive(Default)]
pub(crate) struct FireView {
    pub emitters: Vec<f32>,
    puffs: Vec<TorchPuff>,
    torches: Vec<TorchEmitter>,
    seq: u32,
    /// Cached nearest blaze/ember/torch halo entries: `[x, y, z, r, cr, cg, cb, strength]`.
    halos: Vec<f32>,
}

impl Game {
    /// Steps ballistic torch puffs and fills fire emitters, smouldering tree smoke (into `self.instances`), warm glow
    /// halos, and torch-bright dynamic point lights (`self.avatars.lights`). Does zero work when no fire is active.
    pub(crate) fn write_fire(&mut self, dt: f64, eye: Vec3, time: f64) {
        self.fire_view.emitters.clear();
        self.fire_view.halos.clear();
        let local_burning = self.flamethrower_burning();
        if self.sim.fire.is_empty()
            && !local_burning
            && self.fire_view.puffs.is_empty()
            && self.fire_view.torches.is_empty()
        {
            return;
        }
        let dt = dt.clamp(0.0, 0.05) as f32;
        self.step_torch_puffs(dt, eye, time, local_burning);

        let max_d2 = FIRE_RANGE * FIRE_RANGE;
        let halo_d2 = HALO_RANGE * HALO_RANGE;
        let mut tree_lights: Vec<(Vec3, f32)> = Vec::new();
        let mut front_lights: Vec<(Vec3, f32)> = Vec::new();

        // 1. Burning trees: pick nearest `MAX_SHOWN`, fitted to each tree's real trunk and crown geometry.
        let blazes = self.sim.fire.blazes();
        let mut near_blazes: Vec<(u32, usize)> = blazes
            .iter()
            .enumerate()
            .filter_map(|(i, b)| {
                let d2 = (f64::from(b.centre[0]) - eye.x).powi(2)
                    + (f64::from(b.foot.y) - eye.y).powi(2)
                    + (f64::from(b.centre[1]) - eye.z).powi(2);
                (d2 <= max_d2).then_some(((d2 * 4.0) as u32, i))
            })
            .collect();
        if near_blazes.len() > MAX_SHOWN {
            sort_small_by_key(&mut near_blazes, |x| x.0);
            near_blazes.truncate(MAX_SHOWN);
        }
        for &(d2q, idx) in &near_blazes {
            let b = &blazes[idx];
            let h = f32::from(b.height).max(4.0);
            let cy = f32::from(b.crown_y).clamp(1.0, (h - 1.5).max(1.0));
            let rad = f32::from(b.radius).clamp(1.0, 8.0);
            let crown_h = (h - cy).max(2.0);
            let seed = (hash3(73, b.foot.x, b.foot.y, b.foot.z) % 1000) as f32 * 0.1;
            let base = Vec3::new(f64::from(b.centre[0]), f64::from(b.foot.y), f64::from(b.centre[1])) - eye;
            let flicker = 0.88 + 0.12 * ((time * 9.0 + f64::from(seed)).sin() as f32);
            match b.stage {
                Stage::Crown => {
                    let trunk_w = (rad * 0.42).clamp(1.0, 1.85);
                    let trunk_h = (cy + 1.4).clamp(2.2, 5.5);
                    push_emitter(
                        &mut self.fire_view.emitters,
                        base + Vec3::new(0.0, 0.3, 0.0),
                        trunk_w,
                        trunk_h,
                        seed,
                        0.85,
                        0.0,
                    );
                    let ring = if d2q < (64 * 64 * 4) {
                        4
                    } else if d2q < (112 * 112 * 4) {
                        2
                    } else {
                        1
                    };
                    let base_ang = f64::from(seed) * 0.63;
                    for k in 0..ring {
                        let ang = base_ang + (k as f64) * (TAU / ring as f64);
                        let radial = if ring == 1 { 0.0 } else { f64::from(rad) * 0.48 };
                        let tier = (k as f32 / ring.max(1) as f32) * crown_h * 0.34;
                        let off = Vec3::new(ang.cos() * radial, f64::from(cy + tier), ang.sin() * radial);
                        let w = (rad * 0.86).clamp(1.6, 3.8);
                        let eh = (crown_h * 0.72).clamp(2.4, 6.2);
                        push_emitter(
                            &mut self.fire_view.emitters,
                            base + off,
                            w,
                            eh,
                            seed + (k as f32 + 1.0) * 11.3,
                            0.96,
                            0.0,
                        );
                    }
                    if ring >= 2 {
                        let top_w = (rad * 0.72).clamp(1.4, 3.2);
                        let top_h = (crown_h * 0.62).clamp(2.0, 5.0);
                        push_emitter(
                            &mut self.fire_view.emitters,
                            base + Vec3::new(0.0, f64::from(cy + crown_h * 0.46), 0.0),
                            top_w,
                            top_h,
                            seed + 53.7,
                            0.92,
                            0.0,
                        );
                    }
                    if d2q < (96 * 96 * 4) {
                        smoke::puffs(
                            &mut self.instances,
                            base + Vec3::new(0.0, f64::from(h) * 0.9, 0.0),
                            time,
                            smoke::seed(b.foot),
                            1.15,
                        );
                    }
                    if self.fire_view.halos.len() < MAX_FIRE_HALOS * HALO_FLOATS && f64::from(d2q) * 0.25 <= halo_d2 {
                        let light_rel = Vec3::new(base.x, base.y + f64::from(cy + crown_h * 0.45), base.z);
                        self.fire_view.halos.extend_from_slice(&[
                            light_rel.x as f32,
                            light_rel.y as f32,
                            light_rel.z as f32,
                            (rad * 1.9).clamp(4.0, 11.0),
                            1.0,
                            0.48,
                            0.12,
                            0.95 * flicker,
                        ]);
                        if tree_lights.len() < held_light::MAX_LIGHTS {
                            tree_lights.push((light_rel, (FLAME_REACH + 1.5) * flicker));
                        }
                    }
                }
                Stage::Trunk => {
                    let w = (rad * 0.35).clamp(1.15, 1.75);
                    let low_h = (h * 0.50).clamp(2.2, 6.0);
                    push_emitter(
                        &mut self.fire_view.emitters,
                        base + Vec3::new(0.0, 0.2, 0.0),
                        w,
                        low_h,
                        seed,
                        0.80,
                        0.0,
                    );
                    if h > 5.0 && d2q < (96 * 96 * 4) {
                        push_emitter(
                            &mut self.fire_view.emitters,
                            base + Vec3::new(0.0, f64::from(h * 0.36), 0.0),
                            w * 0.88,
                            (h * 0.48).clamp(2.0, 5.5),
                            seed + 19.4,
                            0.72,
                            0.0,
                        );
                    }
                    if d2q < (96 * 96 * 4) {
                        smoke::puffs(
                            &mut self.instances,
                            base + Vec3::new(0.0, f64::from(h) * 0.72, 0.0),
                            time,
                            smoke::seed(b.foot),
                            1.4,
                        );
                    }
                    if self.fire_view.halos.len() < MAX_FIRE_HALOS * HALO_FLOATS && f64::from(d2q) * 0.25 <= halo_d2 {
                        let light_rel = Vec3::new(base.x, base.y + f64::from(h) * 0.35, base.z);
                        self.fire_view.halos.extend_from_slice(&[
                            light_rel.x as f32,
                            light_rel.y as f32,
                            light_rel.z as f32,
                            3.6,
                            1.0,
                            0.36,
                            0.08,
                            0.75 * flicker,
                        ]);
                        if tree_lights.len() < held_light::MAX_LIGHTS {
                            tree_lights.push((light_rel, FLAME_REACH * flicker));
                        }
                    }
                }
            }
        }

        // 2. Localized weed-torch ground embers: pick nearest `MAX_FRONT_SHOWN`.
        let fronts = self.sim.fire.fronts();
        let mut near_fronts: Vec<(u32, usize)> = fronts
            .iter()
            .enumerate()
            .filter_map(|(i, f)| {
                let d2 = (f64::from(f.pos.x) + 0.5 - eye.x).powi(2)
                    + (f64::from(f.pos.y) + 1.0 - eye.y).powi(2)
                    + (f64::from(f.pos.z) + 0.5 - eye.z).powi(2);
                (d2 <= max_d2).then_some(((d2 * 4.0) as u32, i))
            })
            .collect();
        if near_fronts.len() > MAX_FRONT_SHOWN {
            sort_small_by_key(&mut near_fronts, |x| x.0);
            near_fronts.truncate(MAX_FRONT_SHOWN);
        }
        for (k, &(_, idx)) in near_fronts.iter().enumerate() {
            let f = &fronts[idx];
            let seed = (hash3(199, f.pos.x, f.pos.y, f.pos.z) % 1000) as f32 * 0.1;
            let rel = Vec3::new(f64::from(f.pos.x) + 0.5, f64::from(f.pos.y) + 1.0, f64::from(f.pos.z) + 0.5) - eye;
            push_emitter(&mut self.fire_view.emitters, rel, 1.05, 0.95, seed, 0.85, 1.0);
            if k % 3 == 0 && self.fire_view.halos.len() < MAX_FIRE_HALOS * HALO_FLOATS {
                let light_rel = Vec3::new(rel.x, rel.y + 0.35, rel.z);
                self.fire_view.halos.extend_from_slice(&[
                    light_rel.x as f32,
                    light_rel.y as f32,
                    light_rel.z as f32,
                    2.0,
                    1.0,
                    0.45,
                    0.1,
                    0.50,
                ]);
                if front_lights.len() < held_light::MAX_LIGHTS {
                    front_lights.push((light_rel, FLAME_REACH * 0.72));
                }
            }
        }

        // 3. Ballistic travelling blowtorch puffs + prioritized torch-bright dynamic lights.
        self.write_torch_puffs(eye, time);
        for (pos, reach) in tree_lights {
            push_light(&mut self.avatars.lights, pos, reach);
        }
        for (pos, reach) in front_lights {
            push_light(&mut self.avatars.lights, pos, reach);
        }
    }

    /// Appends fire glow halos to `self.halos` right after `write_halos`.
    pub(crate) fn append_fire_halos(&mut self) {
        if !self.fire_view.halos.is_empty() {
            self.halos.extend_from_slice(&self.fire_view.halos);
        }
    }
}

pub(super) fn push_light(lights: &mut Vec<f32>, rel: Vec3, reach: f32) {
    if reach > 0.0 && lights.len() < held_light::MAX_LIGHTS * held_light::LIGHT_FLOATS {
        lights.extend_from_slice(&[rel.x as f32, rel.y as f32, rel.z as f32, reach]);
    }
}

pub(super) fn push_emitter(
    out: &mut Vec<f32>,
    base: Vec3,
    width: f32,
    height: f32,
    seed: f32,
    intensity: f32,
    kind: f32,
) {
    out.extend_from_slice(&[base.x as f32, base.y as f32, base.z as f32, width, height, seed, intensity, kind]);
}
