//! Drop pods' two machines (Milestone 12, step 12.4, tech Drop Pods): the pod launcher and the landing pad, both
//! 3×3 and both holding items like a silo (`Pick::Pods`, `Pick::Land`, `Pick::stores`).
//!
//! - **Launcher** (3×3×3, [`LAUNCHER_KW`]): belts fill its store with cargo (up to 12 stacks) and bring it a drop pod item
//!   (`DROP_POD`, kept in its one input slot). While it holds a pod and cargo, a comms satellite covers it (`satellites.rs`,
//!   `COMMS`) and a landing pad on its channel is under comms cover too, it loads for [`LOAD_SECONDS`] and then spends the
//!   pod: up to `POD_STACKS` stacks of cargo go up (`pods.rs`) and land `FLIGHT_TICKS` later.
//! - **Pad** (3×3×1, no power): the pods bound for it come down on it and their cargo lands in its store, which belts leading
//!   away empty. A pod whose pad is full waits and lands what fits each tick.
//! - **Channel:** a colour the player tunes the launcher or pad to in its panel (`channels.rs`). A pod goes to the nearest
//!   pad on its launcher's channel that comms cover reaches, so one world can run as many routes as there are colours.
//!
//! Invariants: everything here is derived from the processors, the satellites and the tick; the state is the launcher's
//! `progress`, the channel of each machine (`Extra::Channel`) and the pods in the sky (`Factory::pods`).
//! `Processor::store.flow` holds why a launcher waits (derived).
//!
//! To add a destination kind: a `Pick` like `Land`, its list in `kinds.rs` and a branch in [`pad_index`].

use crate::block::{tex, BlockId, LANDING_PAD, POD_LAUNCHER};
use crate::inventory::Stack;
use crate::item::{ItemId, DROP_POD};
use crate::math::{IVec3, Vec3};
use crate::pods::{Pods, POD_STACKS};
use crate::satellites::{Satellites, COMMS};
use crate::sim::SimEvent;

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::super::power::{FULL_SPEED, NOT_WIRED};
use super::super::ticks;
use super::model::{part, Look, Part};
use super::{Energy, Kinds, Pick, ProcessSpec, ProcessTier, Processor, Status};
use crate::factory::Factory;

mod channels;
#[cfg(test)]
mod tests;

pub(super) use channels::CHANNEL_SAVED;
pub use channels::{channel_colour, channel_name};

/// What a launcher draws while it loads a pod, in kW.
pub const LAUNCHER_KW: u32 = 500;
/// Seconds a launcher takes to load and send a pod, at full power.
pub const LOAD_SECONDS: f64 = 15.0;
/// Stacks of cargo a launcher keeps waiting.
const CARGO_SLOTS: usize = 12;
/// Stacks a landing pad keeps.
const PAD_SLOTS: usize = 24;
/// What a launcher says about itself in `store.flow`, and why it is not loading.
const IDLE: u32 = 0;
const UNCOVERED: u32 = 1;
const NO_PAD: u32 = 2;
const SKY_FULL: u32 = 3;
const LOADING: u32 = 4;

const fn ports(role: Role) -> [Port; 4] {
    [
        Port { side: Side::Front, role, cell: Which::All },
        Port { side: Side::Back, role, cell: Which::All },
        Port { side: Side::Left, role, cell: Which::All },
        Port { side: Side::Right, role, cell: Which::All },
    ]
}

pub const LAUNCHER_SPEC: ProcessSpec = ProcessSpec {
    block: POD_LAUNCHER,
    categories: &[],
    pick: Pick::Pods,
    buffers: [1, 0, CARGO_SLOTS],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: LAUNCHER_KW }],
    footprint: Footprint { size: [3, 3, 3], ports: &ports(Role::In) },
    verb: "Launching",
    products: "cargo",
    waiting: "Waiting for a drop pod and cargo",
    map_colour: 0x3cbedc,
    compute: 0,
    parts: &LAUNCHER_PARTS,
};

pub const PAD_SPEC: ProcessSpec = ProcessSpec {
    block: LANDING_PAD,
    categories: &[],
    pick: Pick::Land,
    buffers: [0, 0, PAD_SLOTS],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Recipe, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint { size: [3, 3, 1], ports: &ports(Role::Out) },
    verb: "Receiving",
    products: "cargo",
    waiting: "Waiting for pods",
    map_colour: 0x9fe6f6,
    compute: 0,
    parts: &PAD_PARTS,
};

impl Processor {
    /// Whether this is a pod launcher.
    pub(super) fn is_launcher(&self) -> bool {
        self.spec.pick == Pick::Pods
    }

    /// Whether this is a landing pad.
    pub(super) fn is_pad(&self) -> bool {
        self.spec.pick == Pick::Land
    }

    /// Whether `item` is a drop pod and this is a launcher: it keeps pods in its input slot, apart from the cargo.
    pub(super) fn takes_pod(&self, item: ItemId) -> bool {
        self.is_launcher() && item == DROP_POD
    }

    /// Whether a launcher is loading a pod and so draws its power.
    pub(super) fn pod_wants_power(&self) -> bool {
        self.store.flow == LOADING
    }

    /// The status line of a launcher or a pad (`None`: another processor).
    pub(super) fn pods_text(&self) -> Option<String> {
        let channel = channel_name(self.channel());
        if self.is_pad() {
            let used = self.out.slots.iter().filter(|s| !s.is_empty()).count();
            return Some(format!("Channel {channel} · holding {used} of {} stacks", self.out.slots.len()));
        }
        if !self.is_launcher() {
            return None;
        }
        Some(match self.store.flow {
            _ if self.input.count(DROP_POD) == 0 => "Needs a drop pod: an assembler builds them".to_string(),
            _ if self.out.total() == 0 => format!("Channel {channel} · waiting for cargo from belts"),
            _ if self.status == Status::NoPower => NOT_WIRED.to_string(),
            UNCOVERED => "No comms satellite overhead: it can launch only under one".to_string(),
            NO_PAD => format!("No landing pad on channel {channel} under comms cover"),
            SKY_FULL => "The sky is full of pods: waiting for one to land".to_string(),
            _ => {
                let whole = ticks(LOAD_SECONDS) * FULL_SPEED;
                format!("Channel {channel} · loading the pod: {}%", (self.progress * 100 / whole).min(100))
            }
        })
    }

    /// A pad's second line: how many pods are on their way to its channel (starts with a newline; empty for none).
    pub(super) fn pods_line(&self, f: &Factory) -> String {
        match f.pods.inbound(self.channel()) {
            _ if !self.is_pad() => String::new(),
            0 => String::new(),
            1 => "\n1 pod on the way".to_string(),
            n => format!("\n{n} pods on the way"),
        }
    }

    /// One tick of a launcher: why it waits or loads, and when the load is done, the launch.
    fn pod_step(&mut self, pad: Option<IVec3>, uplink: bool, pods: &mut Pods, tick: u64) {
        let loaded = self.input.count(DROP_POD) > 0 && self.out.total() > 0;
        self.store.flow = match pad {
            _ if !loaded => IDLE,
            _ if !uplink => UNCOVERED,
            None => NO_PAD,
            _ if !pods.has_room() => SKY_FULL,
            Some(_) => LOADING,
        };
        let Some(pad) = pad.filter(|_| self.store.flow == LOADING) else {
            (self.progress, self.status) = (0, Status::NoInput);
            return;
        };
        if self.speed == 0 {
            self.status = Status::NoPower;
            return;
        }
        self.status = Status::Working;
        self.progress += self.speed;
        if self.progress < ticks(LOAD_SECONDS) * FULL_SPEED {
            return;
        }
        let mut cargo = Vec::new();
        for i in 0..self.out.slots.len() {
            let s = self.out.slots[i];
            if cargo.len() < POD_STACKS && !s.is_empty() {
                cargo.push(s);
                self.out.take(i, s.count);
            }
        }
        self.input.remove(DROP_POD, 1);
        self.used.push(Stack { item: DROP_POD, count: 1 });
        pods.send(self.pos, pad, self.channel(), cargo, tick);
        self.progress = 0;
    }
}

impl Factory {
    /// The pods in their last stretch before landing, as the centre of each capsule's base, relative to nothing (world
    /// coordinates): above its pad by how far it still has to fall. `time` is in ticks, with the frame's fraction.
    /// Presentation (`drone_view.rs`).
    pub fn falling_pods(&self, time: f64) -> Vec<Vec3> {
        let mut seen = Vec::new();
        for (to, height) in self.pods.falling(time) {
            if let Some(p) = self.processors.iter().find(|p| p.is_pad() && p.pos == to) {
                let top = p.pos.as_vec3() + Vec3::new(0.5, 0.0, 0.5) + p.spec.footprint.centre(p.dir);
                seen.push(top + Vec3::new(0.0, height + 0.5, 0.0));
            }
        }
        seen
    }

    /// The pods just launched, rising out of their launchers' tubes, as the centre of each capsule's base (world
    /// coordinates). Presentation (`drone_view.rs`), like `falling_pods`.
    pub fn rising_pods(&self, time: f64) -> Vec<Vec3> {
        let mut seen = Vec::new();
        for (from, height) in self.pods.rising(time) {
            if let Some(p) = self.processors.iter().find(|p| p.is_launcher() && p.pos == from) {
                let mouth = p.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) + p.spec.footprint.centre(p.dir);
                seen.push(mouth + Vec3::new(0.0, TUBE_TOP + height, 0.0));
            }
        }
        seen
    }

    /// Every pod launcher and landing pad: its block, anchor and channel. Presentation (the maps' channel marks).
    pub fn pod_machines(&self) -> impl Iterator<Item = (BlockId, IVec3, u8)> + '_ {
        self.processors.iter().filter(|p| p.is_launcher() || p.is_pad()).map(|p| (p.spec.block, p.pos, p.channel()))
    }

    /// The middle of the processor anchored at `pos`, level with the bottom of its anchor cell (that cell's middle when
    /// none is there any more). Presentation: where a launch or a landing bursts (`orbit_fx.rs`).
    pub fn base_middle(&self, pos: IVec3) -> Vec3 {
        let base = pos.as_vec3() + Vec3::new(0.5, 0.0, 0.5);
        match self.processors.iter().find(|p| p.pos == pos) {
            Some(p) => {
                let c = p.spec.footprint.centre(p.dir);
                base + Vec3::new(c.x, 0.0, c.z)
            }
            None => base,
        }
    }
}

/// Above a launcher's centre, the base of the capsule that shows in its tube (`LAUNCHER_PARTS`'s nose part).
const TUBE_TOP: f64 = 0.8;

/// One tick of the whole pod system, after the processors have stepped: pods that are due land, then each launcher loads or
/// launches. Reports each launch and touchdown (`SimEvent::PodSent`, `PodLanded`). Called by `Factory::update`.
pub(in crate::factory) fn run(
    processors: &mut [Processor],
    kinds: &Kinds,
    pods: &mut Pods,
    sats: &Satellites,
    tick: u64,
    events: &mut Vec<SimEvent>,
) {
    if pods.list.is_empty() && kinds.launcher.is_empty() {
        return;
    }
    land((processors, &kinds.pad), pods, sats, tick, events);
    for &i in &kinds.launcher {
        let i = i as usize;
        let (pos, channel) = (processors[i].pos, processors[i].channel());
        let pad = pad_index((processors, &kinds.pad), pos, channel, sats, tick).map(|k| processors[k].pos);
        let uplink = sats.covering(COMMS, pos.x, pos.z, tick) > 0;
        let before = pods.list.len();
        processors[i].pod_step(pad, uplink, pods, tick);
        if pods.list.len() > before {
            events.push(SimEvent::PodSent { pos });
        }
    }
}

/// Puts the cargo of every pod that is due into its pad: as much as fits, the rest staying in the pod for the next tick.
/// A pod touches down at its landing tick (where the falling capsule ends), whether or not its pad takes it all yet.
fn land(
    (processors, pads): (&mut [Processor], &[u32]),
    pods: &mut Pods,
    sats: &Satellites,
    tick: u64,
    events: &mut Vec<SimEvent>,
) {
    let mut i = 0;
    while i < pods.list.len() {
        let pod = &mut pods.list[i];
        if pod.lands == tick {
            events.push(SimEvent::PodLanded { pos: pod.to });
        }
        let pad = (pod.lands <= tick).then(|| pad_index((processors, pads), pod.to, pod.channel, sats, tick)).flatten();
        let Some(k) = pad else {
            i += 1;
            continue;
        };
        let out = &mut processors[k].out;
        for s in &mut pod.cargo {
            let n = s.count.min(out.space_for(s.item));
            if n > 0 {
                out.add(s.item, n);
                s.count -= n;
            }
        }
        pod.cargo.retain(|s| !s.is_empty());
        if pod.cargo.is_empty() {
            pods.list.remove(i);
        } else {
            i += 1;
        }
    }
}

/// The landing pad (of `pads`, `Kinds::pad`) on `channel` nearest to `near` that a comms satellite covers right now (an
/// index into `processors`). The first of equally near pads wins.
fn pad_index(
    (processors, pads): (&[Processor], &[u32]),
    near: IVec3,
    channel: u8,
    sats: &Satellites,
    tick: u64,
) -> Option<usize> {
    let dist = |p: &Processor| {
        let (dx, dy, dz) = ((p.pos.x - near.x) as i64, (p.pos.y - near.y) as i64, (p.pos.z - near.z) as i64);
        dx * dx + dy * dy + dz * dz
    };
    pads.iter()
        .map(|&k| k as usize)
        .filter(|&k| {
            let p = &processors[k];
            p.channel() == channel && sats.covering(COMMS, p.pos.x, p.pos.z, tick) > 0
        })
        .min_by_key(|&k| dist(&processors[k]))
}

const SLAB: Look = Look::Tex([tex::POD_PAD_TOP, tex::POD_SIDE, tex::FRAME]);
const BODY: Look = Look::Tex([tex::POD_LAUNCH_TOP, tex::POD_SIDE, tex::FRAME]);
const MARK: Look = Look::Channel;

/// A banded plinth, a squat body, a dark launch tube on top with a capsule's nose just showing, a mark on the front
/// face in its channel's colour (so it can be read from afar) and a status lamp.
const LAUNCHER_PARTS: [Part; 6] = [
    part([0.0, -1.35, 0.0], [2.96, 0.3, 2.96], Look::Band(tex::FRAME)),
    part([0.0, -0.45, 0.0], [2.4, 1.5, 2.4], BODY),
    part([0.0, 0.7, 0.0], [1.3, 0.8, 1.3], Look::Tex([tex::POD_LAUNCH_TOP, tex::STEEL, tex::FRAME])),
    part([0.0, 1.3, 0.0], [0.7, 0.4, 0.7], Look::Tex([tex::DROP_POD; 3])),
    part([0.0, -0.9, 1.22], [0.5, 0.25, 0.1], MARK),
    part([1.1, -1.1, 1.1], [0.14, 0.1, 0.14], Look::Lamp),
];

/// A banded plinth, a flat ringed pad with a steel kerb, a mark in its channel's colour on the front edge and a status lamp.
const PAD_PARTS: [Part; 4] = [
    part([0.0, -0.42, 0.0], [2.96, 0.16, 2.96], Look::Band(tex::FRAME)),
    part([0.0, -0.2, 0.0], [2.8, 0.28, 2.8], SLAB),
    part([0.0, -0.02, 1.35], [0.5, 0.1, 0.2], MARK),
    part([1.2, -0.02, 1.2], [0.12, 0.08, 0.12], Look::Lamp),
];
