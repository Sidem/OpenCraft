//! Coal generator: turns fuel (`recipes::fuel_energy`) into stored energy and gives its grid up to its
//! tier's power (`GENERATOR_TIERS`), only as much as the grid draws. Belts, miners and the panel bring fuel into its
//! one buffer; `Power::balance` lights the next item and draws the energy. Right-click opens its panel.
//!
//! Invariants: energy is counted in kW·ticks (1 kJ = `TICK_RATE`); `output` is last tick's supply
//! (for the view, not saved).

use crate::block::{tex, GENERATOR};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::math::{IVec3, Vec3};
use crate::recipes::fuel_energy;
use crate::TICK_RATE;

use super::buffer::Buffer;
use super::panel::{Panel, ROLE_FUEL};
use super::render::push_box;
use super::{Factory, Kind, Machine};

/// What a generator tier gives, Mk1 first.
pub struct GeneratorTier {
    /// kW at most.
    pub power: u32,
    /// How much energy a fuel item gives, in percent of its `fuel_energy`.
    pub yield_percent: u32,
}

/// The Mk1 generator's power, which saves from before version 16 counted in.
pub const GENERATOR_POWER: u32 = 60;

pub const GENERATOR_TIERS: [GeneratorTier; 2] =
    [GeneratorTier { power: GENERATOR_POWER, yield_percent: 100 }, GeneratorTier { power: 100, yield_percent: 125 }];

pub struct Generator {
    pub pos: IVec3,
    pub tier: u8,
    pub fuel: Buffer,
    /// Energy left from the fuel it lit, in kW·ticks.
    pub energy: u32,
    /// kW it supplied last tick (derived).
    pub output: u32,
}

impl Generator {
    pub fn new(pos: IVec3) -> Generator {
        Generator { pos, tier: 0, fuel: Buffer::new(Kind::Generator.def().slots), energy: 0, output: 0 }
    }

    pub fn stats(&self) -> &'static GeneratorTier {
        &GENERATOR_TIERS[self.tier as usize]
    }

    /// How many of `item` it would take now: fuel only, up to the buffer's room.
    pub fn room_for(&self, item: ItemId) -> u32 {
        if fuel_energy(item).is_some() {
            self.fuel.space_for(item)
        } else {
            0
        }
    }

    pub fn accept(&mut self, item: ItemId) -> bool {
        self.room_for(item) > 0 && self.fuel.add(item, 1) == 0
    }

    /// Whether it gave power last tick.
    pub fn running(&self) -> bool {
        self.output > 0
    }

    /// Whole kJ it holds.
    pub fn stored_kj(&self) -> u32 {
        self.energy / TICK_RATE
    }

    /// What it is doing (needs the factory for its grid).
    pub fn status_text(&self, f: &Factory) -> String {
        let pole = f.generators.iter().position(|g| g.pos == self.pos).and_then(|i| f.power.gen_pole.get(i).copied());
        match pole.flatten() {
            _ if f.dirty => String::new(),
            None => f.power.grid_line(None),
            Some(_) if self.running() => format!("Supplying {} of {} kW", self.output, self.stats().power),
            Some(_) if self.fuel.total() == 0 && self.energy == 0 => "Out of fuel: bring coal ore or logs".to_string(),
            Some(_) => "Idle: nothing on its grid needs power".to_string(),
        }
    }

    /// The panel: its fuel, and how long what it holds lasts at this load (the status line needs the
    /// factory, so the caller adds it).
    pub fn panel(&self, status: String) -> Panel {
        Panel {
            block: GENERATOR,
            recipe: None,
            choosable: false,
            progress: 0,
            fire: if self.running() { (self.energy / self.output).div_ceil(TICK_RATE) } else { 0 },
            slots: vec![(ROLE_FUEL, self.fuel.slots[0])],
            status,
            filter: None,
        }
    }
}

impl Machine for Generator {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state: fuel and stored energy.
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        self.fuel.write_state(w);
        w.u32(self.energy);
        w.u8(self.tier);
    }

    fn read_state(r: &mut ByteReader) -> Option<Generator> {
        let mut g = Generator::new(r.ivec3()?);
        g.fuel = Buffer::read_state(r, g.fuel.slots.len())?;
        // Before version 16 this was ticks of fire at full output.
        let n = r.u32()?;
        g.energy = if r.version >= 16 { n } else { n.saturating_mul(GENERATOR_POWER) };
        if r.version >= 21 {
            g.tier = r.u8()?;
        }
        ((g.tier as usize) < GENERATOR_TIERS.len()).then_some(g)
    }

    fn contents(&self) -> Vec<Stack> {
        self.fuel.contents()
    }

    fn describe(&self, f: &Factory) -> String {
        let mut lines = vec![self.status_text(f)];
        let s = self.fuel.slots[0];
        let mut parts = Vec::new();
        if !s.is_empty() {
            parts.push(format!("Fuel: {} {}", s.count, item::name(s.item)));
        }
        if self.energy > 0 {
            parts.push(format!("{} kJ stored", self.stored_kj()));
        }
        if !parts.is_empty() {
            lines.push(parts.join(" · "));
        }
        lines.push("Right-click to open".to_string());
        lines.join("\n")
    }

    /// A steel housing with a spinning-looking fan cap and a status lamp.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        let body = [tex::GENERATOR_TOP, tex::GENERATOR_SIDE, tex::FRAME];
        push_box(out, rel + Vec3::new(0.0, -0.41, 0.0), 0.0, [0.94, 0.17, 0.94], 0.0, [tex::FRAME; 3], false);
        push_box(out, rel + Vec3::new(0.0, -0.16, 0.0), 0.0, [0.76, 0.39, 0.74], 0.0, body, false);
        // Broad flywheel and two crossing blades on top, plus a separate fuel hopper and exhaust.
        push_box(out, rel + Vec3::new(0.0, 0.08, 0.0), 0.0, [0.63, 0.1, 0.63], 0.0, [tex::GENERATOR_TOP; 3], false);
        let spin = if self.running() { (time * 8.0) as f32 } else { 0.0 };
        push_box(out, rel + Vec3::new(0.0, 0.16, 0.0), spin, [0.57, 0.06, 0.1], 0.0, [tex::IRON_PLATE; 3], false);
        push_box(out, rel + Vec3::new(0.0, 0.16, 0.0), spin, [0.1, 0.06, 0.57], 0.0, [tex::IRON_PLATE; 3], false);
        push_box(out, rel + Vec3::new(0.0, 0.2, 0.0), 0.0, [0.14, 0.08, 0.14], 0.0, [tex::COPPER_INGOT; 3], false);
        push_box(
            out,
            rel + Vec3::new(-0.32, 0.12, 0.3),
            0.0,
            [0.26, 0.39, 0.29],
            0.0,
            [tex::BOX_TOP, tex::GENERATOR_SIDE, tex::FRAME],
            false,
        );
        push_box(out, rel + Vec3::new(0.32, 0.31, -0.3), 0.0, [0.2, 0.63, 0.2], 0.0, body, false);
        push_box(out, rel + Vec3::new(0.32, 0.66, -0.3), 0.0, [0.25, 0.07, 0.25], 0.0, [tex::GENERATOR_SIDE; 3], false);
        let lamp = match (self.running(), self.fuel.total() > 0 || self.energy > 0) {
            (true, _) => tex::LAMP_GREEN,
            (false, true) => tex::FRAME,
            (false, false) => tex::LAMP_RED,
        };
        push_box(out, rel + Vec3::new(0.3, 0.1, 0.36), 0.0, [0.12, 0.08, 0.12], 0.0, [lamp; 3], false);
        if self.tier > 0 {
            let band = [tex::stripe(self.tier); 3];
            push_box(out, rel + Vec3::new(0.0, -0.3, 0.0), 0.0, [0.79, 0.08, 0.77], 0.0, band, false);
        }
    }
}
