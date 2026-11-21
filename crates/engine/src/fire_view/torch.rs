//! Ballistic travelling blowtorch flame packets (`TorchPuff`) emitted straight out of the rendered chrome nozzle
//! (`avatars.rs`), with real world-space velocity, air drag, upward thermal buoyancy, ignition ramp, solid-block
//! splash collision, and torch-bright dynamic illumination (`avatars.lights`).

use super::{push_emitter, push_light, MAX_FIRE_HALOS};
use crate::block::{AIR, LIQUID};
use crate::halos::HALO_FLOATS;
use crate::held_light::FLAME_REACH;
use crate::math::{hash3, Vec3};
use crate::raycast::raycast;
use crate::sim::fire::FLAME_RANGE;
use crate::sim::PlayerId;
use crate::Game;

/// Cap on simultaneous ballistic torch puffs across all players.
const MAX_PUFFS: usize = 192;
/// Emission rate (puffs per second) at full torch pressure; sub-stepped across nozzle motion so fast sweeps never gap.
const EMIT_RATE: f32 = 220.0;
/// Seconds for a fresh torch ignition to ramp from first spark to full roaring jet pressure.
const IGNITION_RAMP: f32 = 0.22;
/// How long a remote player's torch keeps emitting after their last `SimEvent::Flamed`.
const REMOTE_JET_LIFE: f64 = 0.28;

/// One physical flame packet emitted from a flamethrower's chrome nozzle, travelling freely through world space.
#[derive(Clone, Copy)]
pub(super) struct TorchPuff {
    pub pos: Vec3,
    pub vel: Vec3,
    pub age: f32,
    pub max_age: f32,
    pub start_size: f32,
    pub seed: f32,
    pub stuck: bool,
}

/// Per-player torch nozzle emitter state (ignition ramp, sub-frame interpolation, remote trigger timer).
#[derive(Clone, Copy)]
pub(super) struct TorchEmitter {
    pub player: u8,
    pub ignition: f32,
    pub emit_acc: f32,
    pub prev_tip: Option<Vec3>,
    pub prev_dir: Option<Vec3>,
    pub active_until: f64,
    pub fallback_origin: Vec3,
    pub fallback_dir: Vec3,
}

impl Game {
    /// Marks a remote player's flamethrower as actively emitting from `SimEvent::Flamed`.
    pub(crate) fn note_flame_jet(&mut self, player: PlayerId, origin: Vec3, dir: Vec3) {
        let until = self.render_time + REMOTE_JET_LIFE;
        if let Some(t) = self.fire_view.torches.iter_mut().find(|t| t.player == player.0) {
            t.active_until = until;
            t.fallback_origin = origin;
            t.fallback_dir = dir;
        } else {
            self.fire_view.torches.push(TorchEmitter {
                player: player.0,
                ignition: 0.0,
                emit_acc: 0.0,
                prev_tip: None,
                prev_dir: None,
                active_until: until,
                fallback_origin: origin,
                fallback_dir: dir,
            });
        }
    }

    /// Emits all live travelling torch puffs (oldest billowing orange tips first, newest electric-blue nozzle core
    /// last) and appends torch-bright dynamic point lights along the flame plume into `self.avatars.lights`.
    pub(super) fn write_torch_puffs(&mut self, eye: Vec3, time: f64) {
        let mut sum_pos = Vec3::ZERO;
        let mut head_pos = Vec3::ZERO;
        let mut head_n = 0usize;
        let mut tail_pos = Vec3::ZERO;
        let mut tail_n = 0usize;
        let live = self.fire_view.puffs.len();
        let split = (live / 2).max(1);
        for (idx, p) in self.fire_view.puffs.iter().enumerate() {
            let u = (p.age / p.max_age).clamp(0.0, 1.0);
            let rel = p.pos - eye;
            let grow = if p.stuck { 1.25 } else { 1.0 };
            let size = p.start_size + 1.32 * u.powf(0.88) * grow;
            let fade_out = 1.0 - ((u - 0.68).max(0.0) / 0.32).powf(1.3);
            let intensity = fade_out.clamp(0.0, 1.0);
            push_emitter(
                &mut self.fire_view.emitters,
                Vec3::new(rel.x, rel.y - f64::from(size) * 0.5, rel.z),
                size,
                size,
                p.seed,
                intensity,
                2.0 + u,
            );
            sum_pos += rel;
            if idx < split {
                head_pos += rel;
                head_n += 1;
            } else {
                tail_pos += rel;
                tail_n += 1;
            }
        }
        if live == 0 {
            return;
        }
        let mid = sum_pos * (1.0 / live as f64);
        let flicker = 0.93 + 0.07 * ((time * 24.0).sin() as f32);
        let ramp = (live as f32 / 14.0).clamp(0.45, 1.0);
        if self.fire_view.halos.len() < MAX_FIRE_HALOS * HALO_FLOATS {
            self.fire_view.halos.extend_from_slice(&[
                mid.x as f32,
                mid.y as f32,
                mid.z as f32,
                3.4,
                1.0,
                0.62,
                0.20,
                0.9 * ramp * flicker,
            ]);
        }
        // Dynamic torch-bright illumination along the travelling flame plume (head + nozzle half).
        let head = head_pos * (1.0 / head_n.max(1) as f64);
        push_light(&mut self.avatars.lights, head, FLAME_REACH * ramp * flicker);
        if tail_n > 0 {
            let tail = tail_pos * (1.0 / tail_n as f64);
            if (head - tail).length() > 1.8 {
                push_light(&mut self.avatars.lights, tail, (FLAME_REACH * 0.88) * ramp * flicker);
            }
        }
    }

    /// Advances in-flight torch puffs with air drag, upward buoyancy and block splash collision, and emits fresh
    /// packets right out of each firing player's rendered chrome nozzle (`self.avatars.muzzle_of`).
    pub(super) fn step_torch_puffs(&mut self, dt: f32, eye: Vec3, time: f64, local_burning: bool) {
        let dt64 = f64::from(dt);
        let world = &self.sim.world;
        let mut i = 0;
        while i < self.fire_view.puffs.len() {
            let p = &mut self.fire_view.puffs[i];
            p.age += dt;
            if p.age >= p.max_age {
                self.fire_view.puffs.remove(i);
                continue;
            }
            let drag = if p.stuck { 7.5 } else { 2.35 };
            p.vel = p.vel * (-drag * dt64).exp();
            p.vel.y += (if p.stuck { 3.4 } else { 2.7 }) * dt64;
            let next = p.pos + p.vel * dt64;
            if world.get_block(next.floor()).is_some_and(|b| b != AIR && !LIQUID[b as usize]) {
                if !p.stuck {
                    p.stuck = true;
                    let s = p.seed * 13.7;
                    p.vel = Vec3::new(
                        p.vel.x * 0.25 + f64::from(s.cos()) * 1.4,
                        1.8,
                        p.vel.z * 0.25 + f64::from(s.sin()) * 1.4,
                    );
                } else {
                    p.vel.x *= 0.5;
                    p.vel.z *= 0.5;
                    p.pos.y += 1.4 * dt64;
                }
            } else {
                p.pos = next;
            }
            i += 1;
        }

        let local_id = self.local.0;
        if local_burning && !self.fire_view.torches.iter().any(|t| t.player == local_id) {
            self.fire_view.torches.push(TorchEmitter {
                player: local_id,
                ignition: 0.0,
                emit_acc: 0.0,
                prev_tip: None,
                prev_dir: None,
                active_until: time,
                fallback_origin: eye,
                fallback_dir: self.body().look_dir(),
            });
        }

        let local_vel = self.body().vel;
        let local_look = self.body().look_dir();
        let third_person = self.third_person.feet.is_some();
        let local_aim_target = if local_burning {
            let world = &self.sim.world;
            let aim_origin = if third_person { eye } else { self.body().eye() };
            let aim_dir = if third_person {
                let (sy, cy) = self.strategy.render_yaw.sin_cos();
                let (sp, cp) = self.strategy.render_pitch.sin_cos();
                Vec3::new(sy * cp, sp, -cy * cp)
            } else {
                local_look
            };
            let reach = raycast(aim_origin, aim_dir, FLAME_RANGE, |p| world.get_block(p).filter(|&b| b != AIR))
                .map_or(FLAME_RANGE, |h| (h.block.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - aim_origin).length());
            Some(aim_origin + aim_dir * reach.max(3.5))
        } else {
            None
        };

        let mut ti = 0;
        while ti < self.fire_view.torches.len() {
            let player = self.fire_view.torches[ti].player;
            let active = if player == local_id {
                local_burning
            } else {
                time < self.fire_view.torches[ti].active_until && time >= self.fire_view.torches[ti].active_until - 1.0
            };
            if !active {
                self.fire_view.torches[ti].ignition = (self.fire_view.torches[ti].ignition - dt * 5.0).max(0.0);
                self.fire_view.torches[ti].prev_tip = None;
                self.fire_view.torches[ti].prev_dir = None;
                if self.fire_view.torches[ti].ignition <= 0.0 {
                    self.fire_view.torches.remove(ti);
                    continue;
                }
                ti += 1;
                continue;
            }

            let (tip, barrel_dir, bell) = if let Some(m) = self.avatars.muzzle_of(player) {
                m
            } else if player == local_id {
                (eye + local_look * 0.45, local_look, 0.05)
            } else {
                let t = self.fire_view.torches[ti];
                (t.fallback_origin + t.fallback_dir * 0.5, t.fallback_dir, 0.06)
            };

            let launch_dir = if player == local_id {
                local_aim_target
                    .and_then(|target| {
                        let to = target - tip;
                        let d = to.length();
                        (d > 1.5).then(|| {
                            let blend = if third_person { 0.35 } else { 0.85 };
                            unit(barrel_dir * (1.0 - blend) + (to * (1.0 / d)) * blend)
                        })
                    })
                    .unwrap_or(barrel_dir)
            } else {
                barrel_dir
            };

            let t = &mut self.fire_view.torches[ti];
            t.ignition = (t.ignition + dt / IGNITION_RAMP).min(1.0);
            let prev_tip = t.prev_tip.filter(|&p| (p - tip).length() < 1.2).unwrap_or(tip);
            let prev_dir = t.prev_dir.unwrap_or(launch_dir);
            t.emit_acc += dt * EMIT_RATE * (0.45 + 0.55 * t.ignition);
            let count = (t.emit_acc.floor() as usize).min(20);
            t.emit_acc -= count as f32;
            let ign = t.ignition;
            t.prev_tip = Some(tip);
            t.prev_dir = Some(launch_dir);

            let carry = if player == local_id { local_vel * 0.25 } else { Vec3::ZERO };
            let start_size = (bell * 1.6).clamp(0.036, 0.11);
            for s in 0..count {
                let frac = (s as f64 + 1.0) / count.max(1) as f64;
                let sub_age = dt * (1.0 - frac as f32);
                let origin = prev_tip + (tip - prev_tip) * frac;
                let dir = unit(prev_dir + (launch_dir - prev_dir) * frac);
                self.fire_view.seq = self.fire_view.seq.wrapping_add(1);
                let h = hash3(0xF1A3, self.fire_view.seq as i32, player as i32, (time * 100.0) as i32);
                let r1 = ((h & 0xFF) as f64 / 255.0) * 2.0 - 1.0;
                let r2 = (((h >> 8) & 0xFF) as f64 / 255.0) * 2.0 - 1.0;
                let r3 = (((h >> 16) & 0xFF) as f32 / 255.0) * 2.0 - 1.0;
                let (right, up) = basis_for(dir);
                let spread = 0.014 + 0.014 * f64::from(ign);
                let speed = 9.0 + 10.5 * f64::from(ign) * (0.94 + 0.12 * r1.abs());
                let vel = unit(dir + (right * r1 + up * r2) * spread) * speed + carry;
                let max_age = (0.16 + 0.26 * ign) * (0.90 + 0.16 * r3);
                if self.fire_view.puffs.len() >= MAX_PUFFS {
                    self.fire_view.puffs.remove(0);
                }
                self.fire_view.puffs.push(TorchPuff {
                    pos: origin + vel * f64::from(sub_age),
                    vel,
                    age: sub_age,
                    max_age: max_age.max(0.10),
                    start_size,
                    seed: (h % 997) as f32 * 0.13,
                    stuck: false,
                });
            }
            ti += 1;
        }
    }
}

fn unit(v: Vec3) -> Vec3 {
    let l = v.length();
    if l > 1e-9 {
        v * (1.0 / l)
    } else {
        Vec3::new(0.0, 0.0, -1.0)
    }
}

fn basis_for(dir: Vec3) -> (Vec3, Vec3) {
    let up_ref = if dir.y.abs() < 0.95 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
    let rx = dir.y * up_ref.z - dir.z * up_ref.y;
    let ry = dir.z * up_ref.x - dir.x * up_ref.z;
    let rz = dir.x * up_ref.y - dir.y * up_ref.x;
    let rl = (rx * rx + ry * ry + rz * rz).sqrt().max(1e-6);
    let right = Vec3::new(rx / rl, ry / rl, rz / rl);
    let up = Vec3::new(
        right.y * dir.z - right.z * dir.y,
        right.z * dir.x - right.x * dir.z,
        right.x * dir.y - right.y * dir.x,
    );
    (right, up)
}
