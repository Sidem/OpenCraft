//! The sensor: logic in one block. It watches the box, silo or belt cell behind it and switches the
//! machine in front of it on or off by a rule, as if the machine's power wire were cut while the sensor
//! says off. A rule is a row of [`RULES`]: always on or off (a plain switch), or a pair of fullness
//! thresholds in percent with hysteresis (the state only flips at the far threshold, so a machine does not
//! flicker). With nothing to watch the sensor stays on. Right-click steps through the rules (`SetSensor`),
//! R turns it.
//!
//! Only machines that take a power wire can be switched (`wiring.rs` `powered_cells`): burners, boxes and
//! belts ignore a sensor. The off state works through `relink`: `Factory::switched_off` lists the anchors
//! whose pole `Power::rebuild` leaves out, and a flip marks the factory dirty. The sensor's `on` is core
//! state (hysteresis needs memory), saved with its rule and facing.
//!
//! To add a rule: a row in [`RULES`] (append; saves store the index). To watch another kind of thing: an arm
//! in `Factory::level_at`.

use rustc_hash::FxHashSet;

use crate::block::{self, tex, BlockId};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::math::{IVec3, Vec3};

use super::links::Slot;
use super::pipes::Part;
use super::process::Pick;
use super::render::push_box;
use super::{Factory, Machine, DIRS};

/// How a sensor decides. Percentages are of how full the watched thing is.
pub struct Rule {
    pub name: &'static str,
    /// A plain switch: the state is this whatever is watched.
    pub fixed: Option<bool>,
    /// Turns on at this level (at or below it when `on_at <= off_at`, at or above it otherwise).
    pub on_at: u8,
    /// Turns off at this level, on the other side.
    pub off_at: u8,
}

pub const RULES: [Rule; 6] = [
    Rule { name: "Always on", fixed: Some(true), on_at: 0, off_at: 0 },
    Rule { name: "Always off", fixed: Some(false), on_at: 0, off_at: 0 },
    Rule { name: "Run while under half full", fixed: None, on_at: 40, off_at: 60 },
    Rule { name: "Run until nearly full", fixed: None, on_at: 75, off_at: 95 },
    Rule { name: "Run while over half full", fixed: None, on_at: 60, off_at: 40 },
    Rule { name: "Run while it holds anything", fixed: None, on_at: 1, off_at: 0 },
];

/// What a switched-off machine's readout says instead of "no power".
pub const SWITCHED_OFF: &str = "Switched off by a sensor";
/// The rule a new sensor starts with.
pub const DEFAULT_RULE: u8 = 3;
/// A belt cell holds this many items bumper to bumper (`ITEM_SPACING` 0.35).
const BELT_CELL_ITEMS: usize = 3;

pub struct Sensor {
    pub pos: IVec3,
    pub dir: u8,
    pub rule: u8,
    pub on: bool,
}

impl Sensor {
    pub fn new(pos: IVec3, dir: u8) -> Sensor {
        Sensor { pos, dir: dir % 4, rule: DEFAULT_RULE, on: true }
    }

    /// The cell it switches.
    pub fn front(&self) -> IVec3 {
        self.pos + DIRS[self.dir as usize]
    }

    /// The cell it reads.
    pub fn behind(&self) -> IVec3 {
        self.pos - DIRS[self.dir as usize]
    }

    /// The state the rule gives for a watched `level` (`None`: nothing to watch), from the current one.
    fn decide(&self, level: Option<u8>) -> bool {
        let rule = &RULES[self.rule as usize];
        if let Some(fixed) = rule.fixed {
            return fixed;
        }
        let Some(level) = level else { return true };
        let (up, down) = (rule.on_at > rule.off_at, rule.on_at <= rule.off_at);
        if (down && level <= rule.on_at) || (up && level >= rule.on_at) {
            true
        } else if (down && level >= rule.off_at) || (up && level <= rule.off_at) {
            false
        } else {
            self.on
        }
    }
}

impl Factory {
    /// The sensor at `pos`, if there is one.
    pub fn sensor(&self, pos: IVec3) -> Option<&Sensor> {
        match self.at.get(&pos) {
            Some(&Slot::Sensor(i)) => Some(&self.sensors[i as usize]),
            _ => None,
        }
    }

    /// Chooses the rule of the sensor at `pos` (a [`RULES`] index; others are ignored).
    pub fn set_sensor(&mut self, pos: IVec3, rule: u8) {
        if let Some(&Slot::Sensor(i)) = self.at.get(&pos) {
            if (rule as usize) < RULES.len() {
                self.sensors[i as usize].rule = rule;
            }
        }
    }

    /// Reads every sensor once; a flip makes the next `relink` leave its machine out of (or back in) the grid.
    pub(super) fn step_sensors(&mut self) {
        for i in 0..self.sensors.len() {
            let s = &self.sensors[i];
            let on = s.decide(self.level_at(s.behind()));
            if on != s.on {
                self.sensors[i].on = on;
                self.dirty = true;
            }
        }
    }

    /// The anchor cells of the machines a sensor currently switches off.
    pub(super) fn switched_off(&self) -> FxHashSet<IVec3> {
        let off = self.sensors.iter().filter(|s| !s.on);
        off.filter_map(|s| self.controlled(s.front())).collect()
    }

    /// The anchor of the machine at `cell` if it takes a power wire (so a sensor can switch it).
    fn controlled(&self, cell: IVec3) -> Option<IVec3> {
        let slot = *self.at.get(&cell)?;
        self.powered_cells(slot).map(|c| c[0])
    }

    /// How full the thing at `cell` is, 0..=100: a box, a silo or a belt cell.
    fn level_at(&self, cell: IVec3) -> Option<u8> {
        match *self.at.get(&cell)? {
            Slot::Storage(i) => Some(self.storages[i as usize].buf.fullness()),
            Slot::Process(i) => {
                let p = &self.processors[i as usize];
                (p.spec.pick == Pick::Store).then(|| p.out.fullness())
            }
            Slot::Belt(i) => Some((self.belts[i as usize].items.len() * 100 / BELT_CELL_ITEMS).min(100) as u8),
            _ => None,
        }
    }

    /// The block standing in `slot`, for readouts.
    fn block_in(&self, slot: Slot) -> BlockId {
        match slot {
            Slot::Belt(_) => block::BELT,
            Slot::Miner(_) => block::MINER,
            Slot::Storage(_) => block::STORAGE,
            Slot::Process(i) => self.processors[i as usize].spec.block,
            Slot::Router(_) => block::SPLITTER,
            Slot::Generator(_) => block::GENERATOR,
            Slot::Pole(_) => block::POLE,
            Slot::Lab(_) => block::LAB,
            Slot::Pipe(i) => match self.pipework[i as usize].part {
                Part::Pump => block::PUMP,
                _ => block::PIPE,
            },
            Slot::Quarry(_) => block::QUARRY,
            Slot::Sensor(_) => block::SENSOR,
        }
    }

    fn name_at(&self, cell: IVec3) -> Option<&'static str> {
        self.at.get(&cell).map(|&s| block::def(self.block_in(s)).name)
    }
}

impl Machine for Sensor {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state: position, facing, rule, whether it is on.
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.dir);
        w.u8(self.rule);
        w.bool(self.on);
    }

    fn read_state(r: &mut ByteReader) -> Option<Sensor> {
        let (pos, dir, rule, on) = (r.ivec3()?, r.u8()?, r.u8()?, r.bool()?);
        (dir < 4 && (rule as usize) < RULES.len()).then_some(Sensor { pos, dir, rule, on })
    }

    fn contents(&self) -> Vec<Stack> {
        Vec::new()
    }

    fn describe(&self, f: &Factory) -> String {
        let watching = match (f.name_at(self.behind()), f.level_at(self.behind())) {
            (Some(name), Some(level)) => format!("Reading: {name}, {level}% full"),
            _ => "Reading: nothing behind it (a storage box, silo or belt)".to_string(),
        };
        let switching = match (f.controlled(self.front()), f.name_at(self.front())) {
            (Some(_), Some(name)) => format!("Switches: {name} {}", if self.on { "(on)" } else { "(OFF)" }),
            _ => "Switches: nothing in front that takes a power wire".to_string(),
        };
        format!("{}\n{watching}\n{switching}\nRight-click: next rule · R: turn", RULES[self.rule as usize].name)
    }

    /// A low housing with a nose each way (the reading side behind, the switched side ahead) and a lamp.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, _: f64) {
        let yaw = self.dir as f32 * std::f32::consts::FRAC_PI_2;
        let (s, c) = yaw.sin_cos();
        let at = |x: f32, y: f32, z: f32| rel + Vec3::new((c * x - s * z) as f64, y as f64, (s * x + c * z) as f64);
        let lamp = if self.on { tex::LAMP_GREEN } else { tex::LAMP_RED };
        push_box(out, at(0.0, -0.3, 0.0), yaw, [0.7, 0.4, 0.7], 0.0, [tex::CIRCUIT, tex::FRAME, tex::FRAME], false);
        push_box(out, at(0.0, -0.3, -0.42), yaw, [0.3, 0.24, 0.16], 0.0, [tex::PORT_OUT; 3], false);
        push_box(out, at(0.0, -0.3, 0.42), yaw, [0.3, 0.24, 0.16], 0.0, [tex::PORT_IN; 3], false);
        push_box(out, at(0.0, 0.0, 0.0), yaw, [0.22, 0.2, 0.22], 0.0, [lamp; 3], false);
    }
}

#[cfg(test)]
mod tests;
