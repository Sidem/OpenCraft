//! Factory machines: conveyor belts, miners and storage boxes.
//!
//! Machines occupy one voxel each (the chunk holds their block id, so collision, targeting and
//! breaking work unchanged) while their state lives here, keyed by position. They keep running
//! when their chunk is streamed out.
//!
//! Belts are simulated per cell: each belt holds a few items with a progress value `p` in 0..1
//! along its length, kept at least [`ITEM_SPACING`] apart. Belts are updated downstream first, so a
//! moving line never stalls for a frame at cell borders. A belt hands its front item to whatever is
//! in front of it: another belt (entering at its start, or in its middle when joining from the side)
//! or a storage box. Miners and boxes push into belts whose back faces them.

use std::f32::consts::FRAC_PI_2;

use rustc_hash::FxHashMap;

use crate::block::{self, tex, BlockId, FACE_BOTTOM, FACE_SIDE, FACE_TEX, FACE_TOP, STONE};
use crate::deposits::{DepositKey, Deposits};
use crate::inventory::{add_to_slots, Stack, MAX_STACK};
use crate::math::{IVec3, Vec3};
use crate::sound::{self, Sounds};
use crate::world::World;

/// Belt speed in blocks per second.
pub const BELT_SPEED: f32 = 1.0;
/// Minimum distance between item centres on a belt, in blocks.
pub const ITEM_SPACING: f32 = 0.35;
const ITEM_SIZE: f32 = 0.25;
/// Where items wait on a belt with nothing in front of it (still fully on the belt).
const END_STOP: f32 = 1.0 - ITEM_SIZE * 0.5;
const BELT_HEIGHT: f32 = 0.18;
/// Ore units a Mk1 miner draws per second (before the deposit's taper and draw cap).
pub const MINER_RATE: f64 = 1.0;
/// Share of drawn units a Mk1 miner turns into ore items.
pub const MINER_RECOVERY: f64 = 0.6;
/// Ore a miner holds before it stops drilling.
pub const MINER_BUFFER: u32 = 64;
pub const STORAGE_SLOTS: usize = 24;
const MINER_SOUND_INTERVAL: f32 = 0.9;
const MINER_SOUND_RANGE: f64 = 24.0;
/// Floats per box instance: centre xyz (camera-relative), yaw, size xyz, uv scroll,
/// texture layers top/side/bottom, uv mode (0 = whole texture per face, 1 = world-scaled).
pub const INSTANCE_FLOATS: usize = 12;

/// Horizontal directions in player-yaw quarter turns: 0 = -Z (north), 1 = +X, 2 = +Z, 3 = -X.
pub const DIRS: [IVec3; 4] = [IVec3::new(0, 0, -1), IVec3::new(1, 0, 0), IVec3::new(0, 0, 1), IVec3::new(-1, 0, 0)];
const DIR_NAMES: [&str; 4] = ["north", "east", "south", "west"];
/// Block face normals in mesher order: +X, -X, +Y, -Y, +Z, -Z.
pub const FACES: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

/// The horizontal direction a player facing `yaw` looks along.
pub fn dir_from_yaw(yaw: f64) -> u8 {
    ((yaw / std::f64::consts::FRAC_PI_2).round() as i32).rem_euclid(4) as u8
}

pub fn face_of(v: IVec3) -> Option<u8> {
    FACES.iter().position(|&f| f == v).map(|i| i as u8)
}

#[inline]
fn opposite(dir: u8) -> u8 {
    (dir + 2) % 4
}

/// Pushes one box instance (see [`INSTANCE_FLOATS`]).
#[allow(clippy::too_many_arguments)]
pub fn push_box(out: &mut Vec<f32>, center: Vec3, yaw: f32, size: [f32; 3], scroll: f32, tex: [u16; 3], world_uv: bool) {
    out.extend_from_slice(&[
        center.x as f32,
        center.y as f32,
        center.z as f32,
        yaw,
        size[0],
        size[1],
        size[2],
        scroll,
        tex[0] as f32,
        tex[1] as f32,
        tex[2] as f32,
        if world_uv { 1.0 } else { 0.0 },
    ]);
}

fn item_tex(item: BlockId) -> [u16; 3] {
    let f = FACE_TEX[item as usize];
    [f[FACE_TOP], f[FACE_SIDE], f[FACE_BOTTOM]]
}

#[derive(Clone, Copy, Debug)]
struct BeltItem {
    item: BlockId,
    p: f32,
}

/// Where a belt, miner or box delivers to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Link {
    None,
    /// `mid`: joining from the side, so the item enters halfway along the target belt.
    Belt { belt: u32, mid: bool },
    Storage(u32),
}

pub struct Belt {
    pub pos: IVec3,
    pub dir: u8,
    /// Front (highest `p`) first.
    items: Vec<BeltItem>,
    out: Link,
    /// Set when the only thing feeding this belt comes in from one side: items then enter from that
    /// side and turn the corner at the centre.
    curve_from: Option<u8>,
}

impl Belt {
    /// Item offset from the cell centre (horizontal) at progress `p`.
    fn offset(&self, p: f32) -> (f32, f32) {
        let (d, t) = match self.curve_from {
            Some(side) if p < 0.5 => (DIRS[side as usize], 0.5 - p),
            _ => (DIRS[self.dir as usize], p - 0.5),
        };
        (d.x as f32 * t, d.z as f32 * t)
    }

    fn mid_free(&self) -> bool {
        self.items.iter().all(|it| (it.p - 0.5).abs() >= ITEM_SPACING)
    }

    /// Accepts an item at the start (with `overflow` progress already travelled) or in the middle.
    fn accept(&mut self, item: BlockId, mid: bool, overflow: f32) -> bool {
        if mid {
            if !self.mid_free() {
                return false;
            }
            let i = self.items.iter().position(|it| it.p < 0.5).unwrap_or(self.items.len());
            self.items.insert(i, BeltItem { item, p: 0.5 });
            return true;
        }
        let p = match self.items.last() {
            Some(rear) if rear.p < ITEM_SPACING => return false,
            Some(rear) => overflow.min(rear.p - ITEM_SPACING),
            None => overflow.min(END_STOP),
        };
        self.items.push(BeltItem { item, p: p.max(0.0) });
        true
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MinerStatus {
    Running,
    OutputFull,
    NoDeposit,
    Exhausted,
}

pub struct Miner {
    pub pos: IVec3,
    /// Face index (see [`FACES`]) pointing from the miner into the ore it was placed against.
    pub drill: u8,
    pub deposit: Option<DepositKey>,
    pub ore: BlockId,
    pub held: u32,
    /// Recovered ore not yet a whole item.
    carry: f64,
    outs: Vec<Link>,
    next_out: usize,
    pub status: MinerStatus,
    /// Smoothed draw in units per second, for the readout.
    pub draw_rate: f64,
    sound_timer: f32,
}

pub struct Storage {
    pub pos: IVec3,
    pub slots: [Stack; STORAGE_SLOTS],
    outs: Vec<u32>,
    next_out: usize,
}

impl Storage {
    fn can_accept(&self, item: BlockId) -> bool {
        self.slots.iter().any(|s| s.is_empty() || (s.item == item && s.count < MAX_STACK))
    }
}

#[derive(Clone, Copy, Debug)]
enum Slot {
    Belt(u32),
    Miner(u32),
    Storage(u32),
}

#[derive(Default)]
pub struct Factory {
    belts: Vec<Belt>,
    miners: Vec<Miner>,
    storages: Vec<Storage>,
    at: FxHashMap<IVec3, Slot>,
    /// Belt indices, downstream first.
    order: Vec<u32>,
    dirty: bool,
    pub deposits: Deposits,
    tick: u64,
}

/// Hands one item to a link. `overflow` is how far past the end of the source belt it already is.
fn deliver(belts: &mut [Belt], storages: &mut [Storage], link: Link, item: BlockId, overflow: f32) -> bool {
    match link {
        Link::None => false,
        Link::Belt { belt, mid } => belts[belt as usize].accept(item, mid, overflow),
        Link::Storage(s) => add_to_slots(&mut storages[s as usize].slots, item, 1) == 0,
    }
}

/// Merges loose items into stacks (for dropping a machine's contents).
fn stacks_of(items: impl Iterator<Item = BlockId>) -> Vec<Stack> {
    let mut out: Vec<Stack> = Vec::new();
    for item in items {
        match out.iter_mut().find(|s| s.item == item && s.count < MAX_STACK) {
            Some(s) => s.count += 1,
            None => out.push(Stack { item, count: 1 }),
        }
    }
    out
}

pub fn fmt_int(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

pub fn fmt_duration(seconds: f64) -> String {
    let s = seconds.max(0.0).round() as u64;
    match s {
        0..=59 => format!("{s} s"),
        60..=3599 => format!("{} min", s / 60),
        3600..=86_399 => format!("{} h {} min", s / 3600, s % 3600 / 60),
        _ => format!("{} d {} h", s / 86_400, s % 86_400 / 3600),
    }
}

impl Factory {
    pub fn belt_count(&self) -> usize {
        self.belts.len()
    }

    pub fn miner_count(&self) -> usize {
        self.miners.len()
    }

    pub fn storage_count(&self) -> usize {
        self.storages.len()
    }

    pub fn add_belt(&mut self, pos: IVec3, dir: u8) {
        self.remove(pos);
        self.at.insert(pos, Slot::Belt(self.belts.len() as u32));
        self.belts.push(Belt { pos, dir: dir % 4, items: Vec::new(), out: Link::None, curve_from: None });
        self.dirty = true;
    }

    pub fn add_miner(&mut self, pos: IVec3, drill: u8, deposit: Option<DepositKey>) {
        self.remove(pos);
        self.at.insert(pos, Slot::Miner(self.miners.len() as u32));
        self.miners.push(Miner {
            pos,
            drill: drill.min(5),
            deposit,
            ore: deposit.map_or(STONE, |k| k.ore),
            held: 0,
            carry: 0.0,
            outs: Vec::new(),
            next_out: 0,
            status: if deposit.is_some() { MinerStatus::Running } else { MinerStatus::NoDeposit },
            draw_rate: 0.0,
            sound_timer: 0.0,
        });
        self.dirty = true;
    }

    pub fn add_storage(&mut self, pos: IVec3) {
        self.remove(pos);
        self.at.insert(pos, Slot::Storage(self.storages.len() as u32));
        self.storages.push(Storage { pos, slots: [Stack::default(); STORAGE_SLOTS], outs: Vec::new(), next_out: 0 });
        self.dirty = true;
    }

    /// Removes the machine at `pos`, returning whatever it was holding or carrying.
    pub fn remove(&mut self, pos: IVec3) -> Vec<Stack> {
        let Some(slot) = self.at.remove(&pos) else { return Vec::new() };
        self.dirty = true;
        match slot {
            Slot::Belt(i) => {
                let b = self.belts.swap_remove(i as usize);
                if let Some(moved) = self.belts.get(i as usize) {
                    self.at.insert(moved.pos, Slot::Belt(i));
                }
                stacks_of(b.items.iter().map(|it| it.item))
            }
            Slot::Miner(i) => {
                let m = self.miners.swap_remove(i as usize);
                if let Some(moved) = self.miners.get(i as usize) {
                    self.at.insert(moved.pos, Slot::Miner(i));
                }
                if m.held > 0 {
                    vec![Stack { item: m.ore, count: m.held }]
                } else {
                    Vec::new()
                }
            }
            Slot::Storage(i) => {
                let s = self.storages.swap_remove(i as usize);
                if let Some(moved) = self.storages.get(i as usize) {
                    self.at.insert(moved.pos, Slot::Storage(i));
                }
                s.slots.into_iter().filter(|s| !s.is_empty()).collect()
            }
        }
    }

    /// Right-click on a box or miner: moves its contents into `take(item, count)`, which returns
    /// how many it accepted. Returns false if there is no such machine at `pos`.
    pub fn take_contents(&mut self, pos: IVec3, mut take: impl FnMut(BlockId, u32) -> u32) -> bool {
        match self.at.get(&pos) {
            Some(Slot::Storage(i)) => {
                for s in self.storages[*i as usize].slots.iter_mut().filter(|s| !s.is_empty()) {
                    s.count -= take(s.item, s.count).min(s.count);
                    if s.count == 0 {
                        *s = Stack::default();
                    }
                }
                true
            }
            Some(Slot::Miner(i)) => {
                let m = &mut self.miners[*i as usize];
                if m.held > 0 {
                    m.held -= take(m.ore, m.held).min(m.held);
                }
                true
            }
            _ => false,
        }
    }

    /// Recomputes belt links, curves, machine outputs and the belt update order.
    fn relink(&mut self) {
        self.dirty = false;
        let at = &self.at;
        let belts = &self.belts;
        let belt_at = |q: IVec3| match at.get(&q) {
            Some(Slot::Belt(j)) => Some(*j),
            _ => None,
        };

        let curves: Vec<Option<u8>> = belts
            .iter()
            .map(|b| {
                let back = b.pos - DIRS[b.dir as usize];
                let (mut back_fed, mut sides, mut side) = (false, 0, None);
                for s in 0..4u8 {
                    if s == b.dir {
                        continue;
                    }
                    let q = b.pos + DIRS[s as usize];
                    let feeds = match at.get(&q) {
                        Some(Slot::Belt(j)) => {
                            let o = &belts[*j as usize];
                            o.pos + DIRS[o.dir as usize] == b.pos
                        }
                        // Machines only feed belts whose back faces them.
                        Some(_) => q == back,
                        None => false,
                    };
                    if feeds && q == back {
                        back_fed = true;
                    } else if feeds {
                        sides += 1;
                        side = Some(s);
                    }
                }
                if !back_fed && sides == 1 {
                    side
                } else {
                    None
                }
            })
            .collect();

        let outs: Vec<Link> = belts
            .iter()
            .map(|b| {
                let t = b.pos + DIRS[b.dir as usize];
                match at.get(&t) {
                    Some(Slot::Belt(j)) => {
                        let o = &belts[*j as usize];
                        if o.dir == opposite(b.dir) {
                            Link::None
                        } else {
                            let from_back = o.pos - DIRS[o.dir as usize] == b.pos;
                            let start = from_back || curves[*j as usize] == Some(opposite(b.dir));
                            Link::Belt { belt: *j, mid: !start }
                        }
                    }
                    Some(Slot::Storage(k)) => Link::Storage(*k),
                    _ => Link::None,
                }
            })
            .collect();

        // Machines feed belts that lead away from them.
        let feeds = |pos: IVec3| -> Vec<u32> {
            (0..4u8)
                .filter_map(|s| belt_at(pos + DIRS[s as usize]).filter(|&j| belts[j as usize].dir == s))
                .collect()
        };
        let miner_outs: Vec<Vec<Link>> = self
            .miners
            .iter()
            .map(|m| {
                let mut v: Vec<Link> = feeds(m.pos).into_iter().map(|belt| Link::Belt { belt, mid: false }).collect();
                for (f, &n) in FACES.iter().enumerate() {
                    if f as u8 == m.drill {
                        continue;
                    }
                    if let Some(Slot::Storage(k)) = at.get(&(m.pos + n)) {
                        v.push(Link::Storage(*k));
                    }
                }
                v
            })
            .collect();
        let storage_outs: Vec<Vec<u32>> = self.storages.iter().map(|s| feeds(s.pos)).collect();

        for ((b, c), o) in self.belts.iter_mut().zip(curves).zip(outs) {
            b.curve_from = c;
            b.out = o;
        }
        for (m, o) in self.miners.iter_mut().zip(miner_outs) {
            m.outs = o;
            m.next_out %= m.outs.len().max(1);
        }
        for (s, o) in self.storages.iter_mut().zip(storage_outs) {
            s.outs = o;
            s.next_out %= s.outs.len().max(1);
        }

        // Each belt has at most one belt downstream, so walking the chain from every unvisited belt
        // and appending it reversed puts every belt after the one it feeds.
        let n = self.belts.len();
        let mut visited = vec![false; n];
        let mut path = Vec::new();
        self.order.clear();
        for start in 0..n {
            let mut cur = start;
            path.clear();
            while !visited[cur] {
                visited[cur] = true;
                path.push(cur as u32);
                match self.belts[cur].out {
                    Link::Belt { belt, .. } => cur = belt as usize,
                    _ => break,
                }
            }
            self.order.extend(path.iter().rev());
        }
    }

    pub fn update(&mut self, dt: f64, world: &mut World, eye: Vec3, sounds: &mut Sounds) {
        if self.dirty {
            self.relink();
        }
        self.tick += 1;
        let tick = self.tick;
        let Factory { belts, miners, storages, deposits, order, .. } = self;

        for m in miners.iter_mut() {
            let mut drawn = 0.0;
            match m.deposit {
                None => m.status = MinerStatus::NoDeposit,
                Some(key) => {
                    if m.held < MINER_BUFFER {
                        let face = m.pos + FACES[m.drill as usize];
                        drawn = deposits.draw(world, &key, MINER_RATE, face, tick, dt);
                        m.carry += drawn * MINER_RECOVERY;
                        let whole = m.carry.floor();
                        m.held += whole as u32;
                        m.carry -= whole;
                    }
                    let exhausted = deposits.get(&key).is_none_or(|s| s.exhausted());
                    m.status = if m.held >= MINER_BUFFER {
                        MinerStatus::OutputFull
                    } else if exhausted && drawn == 0.0 {
                        MinerStatus::Exhausted
                    } else {
                        MinerStatus::Running
                    };
                }
            }
            // A new miner's average starts from its first reading, so the readout is right at once.
            let rate = drawn / dt.max(1e-6);
            if m.draw_rate == 0.0 {
                m.draw_rate = rate;
            } else {
                m.draw_rate += (rate - m.draw_rate) * (dt / 2.0).min(1.0);
            }

            if m.held > 0 && !m.outs.is_empty() {
                let n = m.outs.len();
                for i in 0..n {
                    let slot = (m.next_out + i) % n;
                    if deliver(belts, storages, m.outs[slot], m.ore, 0.0) {
                        m.held -= 1;
                        m.next_out = (slot + 1) % n;
                        break;
                    }
                }
            }

            if drawn > 0.0 {
                m.sound_timer -= dt as f32;
                if m.sound_timer <= 0.0 {
                    m.sound_timer = MINER_SOUND_INTERVAL;
                    let at = (m.pos + FACES[m.drill as usize]).as_vec3() + Vec3::new(0.5, 0.5, 0.5);
                    if (at - eye).length() < MINER_SOUND_RANGE {
                        sounds.push(sound::DIG, block::sound::STONE, at - eye, 0.35);
                    }
                }
            }
        }

        for s in storages.iter_mut() {
            if s.outs.is_empty() {
                continue;
            }
            let Some(src) = s.slots.iter().rposition(|st| !st.is_empty()) else { continue };
            let item = s.slots[src].item;
            let n = s.outs.len();
            for i in 0..n {
                let slot = (s.next_out + i) % n;
                if belts[s.outs[slot] as usize].accept(item, false, 0.0) {
                    let st = &mut s.slots[src];
                    st.count -= 1;
                    if st.count == 0 {
                        *st = Stack::default();
                    }
                    s.next_out = (slot + 1) % n;
                    break;
                }
            }
        }

        let step = BELT_SPEED * dt as f32;
        for &bi in order.iter() {
            let bi = bi as usize;
            let out = belts[bi].out;
            let Some(front) = belts[bi].items.first().copied() else { continue };
            let mut limit = match out {
                Link::None => END_STOP,
                Link::Belt { belt, mid: false } => {
                    belts[belt as usize].items.last().map_or(f32::INFINITY, |r| 1.0 + r.p - ITEM_SPACING)
                }
                Link::Belt { belt, mid: true } => {
                    if belts[belt as usize].mid_free() {
                        f32::INFINITY
                    } else {
                        1.0
                    }
                }
                Link::Storage(k) => {
                    if storages[k as usize].can_accept(front.item) {
                        f32::INFINITY
                    } else {
                        1.0
                    }
                }
            };
            for it in belts[bi].items.iter_mut() {
                it.p = (it.p + step).min(limit).max(it.p);
                limit = it.p - ITEM_SPACING;
            }
            while let Some(front) = belts[bi].items.first().copied() {
                if front.p < 1.0 {
                    break;
                }
                if deliver(belts, storages, out, front.item, front.p - 1.0) {
                    belts[bi].items.remove(0);
                } else {
                    belts[bi].items[0].p = 1.0;
                    break;
                }
            }
        }
    }

    /// Writes box instances for every machine within `range` of the camera.
    pub fn write_instances(&self, out: &mut Vec<f32>, eye: Vec3, time: f64, range: f64) {
        let r2 = range * range;
        let near = |c: Vec3| {
            let d = c - eye;
            d.x * d.x + d.y * d.y + d.z * d.z <= r2
        };
        let scroll = (time * BELT_SPEED as f64).fract() as f32;
        for b in &self.belts {
            let base = b.pos.as_vec3() + Vec3::new(0.5, 0.0, 0.5);
            if !near(base) {
                continue;
            }
            let rel = base - eye;
            let yaw = b.dir as f32 * FRAC_PI_2;
            let (s, c) = yaw.sin_cos();
            let at = |x: f32, y: f32, z: f32| rel + Vec3::new((c * x - s * z) as f64, y as f64, (s * x + c * z) as f64);
            push_box(out, at(0.0, BELT_HEIGHT * 0.5, 0.0), yaw, [0.84, BELT_HEIGHT, 1.0], scroll, [tex::BELT_TOP, tex::FRAME, tex::FRAME], true);
            for side in [-0.46, 0.46] {
                push_box(out, at(side, 0.13, 0.0), yaw, [0.08, 0.26, 1.0], 0.0, [tex::FRAME; 3], true);
            }
            for it in &b.items {
                let (x, z) = b.offset(it.p);
                let pos = rel + Vec3::new(x as f64, (BELT_HEIGHT + ITEM_SIZE * 0.5) as f64, z as f64);
                push_box(out, pos, yaw, [ITEM_SIZE; 3], 0.0, item_tex(it.item), false);
            }
        }

        for m in &self.miners {
            let center = m.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
            if !near(center) {
                continue;
            }
            let rel = center - eye;
            let f = FACES[m.drill as usize].as_vec3();
            let axis = m.drill as usize / 2;
            let size = |along: f32, across: f32| {
                let mut s = [across; 3];
                s[axis] = along;
                s
            };
            let running = m.status == MinerStatus::Running && m.draw_rate > 0.01;
            let pump = if running { 0.05 * (0.5 + 0.5 * (time * 10.0).sin()) } else { 0.0 };
            push_box(out, rel + f * -0.13, 0.0, size(0.7, 0.86), 0.0, [tex::MINER_TOP, tex::MINER_SIDE, tex::FRAME], true);
            push_box(out, rel + f * 0.27, 0.0, size(0.1, 0.6), 0.0, [tex::FRAME; 3], true);
            push_box(out, rel + f * (0.44 + pump), 0.0, size(0.36, 0.22), 0.0, [tex::DRILL; 3], true);
            let lamp = match m.status {
                MinerStatus::Running => tex::LAMP_GREEN,
                MinerStatus::OutputFull => tex::LAMP_YELLOW,
                MinerStatus::NoDeposit | MinerStatus::Exhausted => tex::LAMP_RED,
            };
            push_box(out, rel + f * -0.5, 0.0, size(0.04, 0.2), 0.0, [lamp; 3], false);
        }
    }

    /// Detail lines for the target readout.
    pub fn describe(&self, pos: IVec3) -> Option<String> {
        match *self.at.get(&pos)? {
            Slot::Belt(i) => {
                let b = &self.belts[i as usize];
                let load = match b.items.len() {
                    0 => "Empty".to_string(),
                    1 => "Carrying 1 item".to_string(),
                    n => format!("Carrying {n} items"),
                };
                let end = if self.dirty {
                    ""
                } else {
                    match b.out {
                        Link::None => " · nothing in front, items wait at the end",
                        Link::Belt { mid: true, .. } => " · joins the next belt from the side",
                        Link::Belt { .. } => "",
                        Link::Storage(_) => " · delivers into a box",
                    }
                };
                Some(format!("{load} · heading {}{end}", DIR_NAMES[b.dir as usize]))
            }
            Slot::Storage(i) => {
                let s = &self.storages[i as usize];
                let used = s.slots.iter().filter(|st| !st.is_empty()).count();
                let mut totals: Vec<(BlockId, u32)> = Vec::new();
                for st in s.slots.iter().filter(|st| !st.is_empty()) {
                    match totals.iter_mut().find(|(item, _)| *item == st.item) {
                        Some((_, n)) => *n += st.count,
                        None => totals.push((st.item, st.count)),
                    }
                }
                // Largest first. At most 24 entries: an insertion sort, rather than pulling a
                // stable-sort instantiation (~9 KB) into the wasm for a readout.
                for i in 1..totals.len() {
                    let mut j = i;
                    while j > 0 && totals[j - 1].1 < totals[j].1 {
                        totals.swap(j - 1, j);
                        j -= 1;
                    }
                }
                let mut lines = vec![format!("{used} of {STORAGE_SLOTS} slots used")];
                if !totals.is_empty() {
                    let list: Vec<String> =
                        totals.iter().take(3).map(|(item, n)| format!("{} {}", fmt_int(*n as u64), block::def(*item).name)).collect();
                    lines.push(list.join(", ") + if totals.len() > 3 { ", ..." } else { "" });
                    lines.push("Right-click to take everything".to_string());
                }
                Some(lines.join("\n"))
            }
            Slot::Miner(i) => {
                let m = &self.miners[i as usize];
                let Some(key) = m.deposit else {
                    return Some("Not on an ore deposit. Place miners against an ore block.".to_string());
                };
                let st = self.deposits.get(&key)?;
                let mut lines = vec![format!(
                    "{} · {} of {} blocks left",
                    st.deposit.name(),
                    fmt_int(st.remaining_blocks as u64),
                    fmt_int(st.initial_blocks as u64)
                )];
                lines.push(match m.status {
                    // Integers only: formatting floats pulls ~25 KB of float printing into the wasm.
                    MinerStatus::Running if m.draw_rate > 0.01 => format!(
                        "Running · {} ore/min ({}% recovery)",
                        (m.draw_rate * MINER_RECOVERY * 60.0).round() as u32,
                        (MINER_RECOVERY * 100.0).round() as u32
                    ),
                    MinerStatus::Running => "Waiting: other miners are using this deposit's full draw".to_string(),
                    MinerStatus::OutputFull => "Output full: put a belt leading away, or a box, next to it".to_string(),
                    MinerStatus::Exhausted => "Deposit worked out".to_string(),
                    MinerStatus::NoDeposit => String::new(),
                });
                if m.held > 0 {
                    lines.push(format!("Holding {} {} · right-click to take", m.held, block::def(m.ore).name));
                }
                let total_draw: f64 = self.miners.iter().filter(|o| o.deposit == Some(key)).map(|o| o.draw_rate).sum();
                if total_draw > 0.01 && !st.exhausted() {
                    lines.push(format!(
                        "{} units left · about {} at the current draw",
                        fmt_int(st.remaining_units() as u64),
                        fmt_duration(st.remaining_units() / total_draw)
                    ));
                }
                Some(lines.join("\n"))
            }
        }
    }

    #[cfg(test)]
    pub fn storage_count_at(&self, pos: IVec3, item: BlockId) -> u32 {
        match self.at.get(&pos) {
            Some(Slot::Storage(i)) => self.storages[*i as usize].slots.iter().filter(|s| s.item == item).map(|s| s.count).sum(),
            _ => 0,
        }
    }

    #[cfg(test)]
    fn belt_at(&self, pos: IVec3) -> &Belt {
        match self.at.get(&pos) {
            Some(Slot::Belt(i)) => &self.belts[*i as usize],
            _ => panic!("no belt at {pos:?}"),
        }
    }

    #[cfg(test)]
    pub fn miner_at(&self, pos: IVec3) -> &Miner {
        match self.at.get(&pos) {
            Some(Slot::Miner(i)) => &self.miners[*i as usize],
            _ => panic!("no miner at {pos:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::IRON_ORE;

    const EAST: u8 = 1;
    const SOUTH: u8 = 2;

    fn run(f: &mut Factory, seconds: f64, mut check: impl FnMut(&Factory)) {
        let mut world = World::new(1, 2);
        let mut sounds = Sounds::default();
        let steps = (seconds * 60.0) as usize;
        for _ in 0..steps {
            f.update(1.0 / 60.0, &mut world, Vec3::ZERO, &mut sounds);
            check(f);
            sounds.clear();
        }
    }

    fn spacing_ok(f: &Factory) {
        for b in &f.belts {
            for w in b.items.windows(2) {
                assert!(w[0].p - w[1].p >= ITEM_SPACING - 1e-4, "items too close on belt at {:?}: {:?}", b.pos, b.items);
            }
            for it in &b.items {
                assert!((0.0..=1.0).contains(&it.p), "item off the belt: {it:?}");
            }
        }
    }

    fn stocked_box(f: &mut Factory, pos: IVec3, item: BlockId, n: u32) {
        f.add_storage(pos);
        let Some(Slot::Storage(i)) = f.at.get(&pos) else { unreachable!() };
        add_to_slots(&mut f.storages[*i as usize].slots, item, n);
    }

    #[test]
    fn box_to_box_through_a_belt_line() {
        let mut f = Factory::default();
        stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE, 10);
        for x in 1..=5 {
            f.add_belt(IVec3::new(x, 0, 0), EAST);
        }
        f.add_storage(IVec3::new(6, 0, 0));
        run(&mut f, 12.0, spacing_ok);
        assert_eq!(f.storage_count_at(IVec3::new(6, 0, 0), IRON_ORE), 10);
        assert_eq!(f.storage_count_at(IVec3::new(0, 0, 0), IRON_ORE), 0);
    }

    #[test]
    fn blocked_line_backs_up_without_overlap() {
        let mut f = Factory::default();
        stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE, 40);
        for x in 1..=3 {
            f.add_belt(IVec3::new(x, 0, 0), EAST);
        }
        run(&mut f, 20.0, spacing_ok);
        let on_belts: usize = f.belts.iter().map(|b| b.items.len()).sum();
        assert!((7..=9).contains(&on_belts), "{on_belts} items on three full belts");
        assert_eq!(f.storage_count_at(IVec3::new(0, 0, 0), IRON_ORE) as usize, 40 - on_belts);
        let last = f.belt_at(IVec3::new(3, 0, 0));
        assert!(last.items[0].p <= END_STOP + 1e-6);
    }

    #[test]
    fn corner_turns_and_side_joins() {
        let mut f = Factory::default();
        // East, then a corner turning south.
        f.add_belt(IVec3::new(0, 0, 0), EAST);
        f.add_belt(IVec3::new(1, 0, 0), SOUTH);
        // A straight southbound line with a belt joining it from the west side.
        f.add_belt(IVec3::new(5, 0, -1), SOUTH);
        f.add_belt(IVec3::new(5, 0, 0), SOUTH);
        f.add_belt(IVec3::new(4, 0, 0), EAST);
        // Two belts facing each other never connect.
        f.add_belt(IVec3::new(9, 0, 0), EAST);
        f.add_belt(IVec3::new(10, 0, 0), 3);
        f.relink();

        assert_eq!(f.belt_at(IVec3::new(1, 0, 0)).curve_from, Some(3));
        assert!(matches!(f.belt_at(IVec3::new(0, 0, 0)).out, Link::Belt { mid: false, .. }));
        assert_eq!(f.belt_at(IVec3::new(5, 0, 0)).curve_from, None);
        assert!(matches!(f.belt_at(IVec3::new(4, 0, 0)).out, Link::Belt { mid: true, .. }));
        assert!(matches!(f.belt_at(IVec3::new(5, 0, -1)).out, Link::Belt { mid: false, .. }));
        assert_eq!(f.belt_at(IVec3::new(9, 0, 0)).out, Link::None);
        assert_eq!(f.belt_at(IVec3::new(10, 0, 0)).out, Link::None);

        // Items entering the corner start at its west edge and leave through its south edge.
        let corner = f.belt_at(IVec3::new(1, 0, 0));
        assert_eq!(corner.offset(0.0), (-0.5, 0.0));
        assert_eq!(corner.offset(1.0), (0.0, 0.5));
    }

    #[test]
    fn merging_lines_deliver_everything() {
        let mut f = Factory::default();
        stocked_box(&mut f, IVec3::new(0, 0, -3), IRON_ORE, 12);
        for z in -2..=2 {
            f.add_belt(IVec3::new(0, 0, z), SOUTH);
        }
        stocked_box(&mut f, IVec3::new(-3, 0, 0), crate::block::COAL_ORE, 12);
        for x in -2..=-1 {
            f.add_belt(IVec3::new(x, 0, 0), EAST);
        }
        f.add_storage(IVec3::new(0, 0, 3));
        run(&mut f, 40.0, spacing_ok);
        assert_eq!(f.storage_count_at(IVec3::new(0, 0, 3), IRON_ORE), 12);
        assert_eq!(f.storage_count_at(IVec3::new(0, 0, 3), crate::block::COAL_ORE), 12);
    }

    #[test]
    fn removing_a_belt_returns_its_items_and_relinks() {
        let mut f = Factory::default();
        stocked_box(&mut f, IVec3::new(0, 0, 0), IRON_ORE, 20);
        for x in 1..=4 {
            f.add_belt(IVec3::new(x, 0, 0), EAST);
        }
        run(&mut f, 3.0, |_| {});
        let dropped = f.remove(IVec3::new(1, 0, 0));
        assert!(dropped.iter().map(|s| s.count).sum::<u32>() > 0);
        // The box has no belt leading away any more; the rest of the line still drains forward.
        run(&mut f, 1.0, spacing_ok);
        assert_eq!(f.belt_count(), 3);
        assert!(f.belt_at(IVec3::new(2, 0, 0)).out != Link::None);
    }

    #[test]
    fn number_formatting() {
        assert_eq!(fmt_int(0), "0");
        assert_eq!(fmt_int(999), "999");
        assert_eq!(fmt_int(1000), "1,000");
        assert_eq!(fmt_int(6_000_000), "6,000,000");
        assert_eq!(fmt_duration(42.0), "42 s");
        assert_eq!(fmt_duration(125.0), "2 min");
        assert_eq!(fmt_duration(3.0 * 3600.0 + 20.0 * 60.0), "3 h 20 min");
        assert_eq!(fmt_duration(90_000.0), "1 d 1 h");
    }

    #[test]
    fn box_readout_lists_the_largest_stock_first() {
        use crate::block::{COAL_ORE, COPPER_ORE, STONE};
        let mut f = Factory::default();
        let pos = IVec3::new(0, 0, 0);
        stocked_box(&mut f, pos, COAL_ORE, 5);
        let Some(Slot::Storage(i)) = f.at.get(&pos) else { unreachable!() };
        let slots = &mut f.storages[*i as usize].slots;
        add_to_slots(slots, IRON_ORE, 130);
        add_to_slots(slots, COPPER_ORE, 40);
        add_to_slots(slots, STONE, 1);
        let text = f.describe(pos).unwrap();
        // 130 iron fills three stacks of 64.
        assert_eq!(text, "6 of 24 slots used\n130 Iron Ore, 40 Copper Ore, 5 Coal Ore, ...\nRight-click to take everything");
    }
}
