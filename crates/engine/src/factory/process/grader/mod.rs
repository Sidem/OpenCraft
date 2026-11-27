//! The Telescoping Bucket-Wheel Site Grader (`SITE_GRADER`): a 3×3×2 heavy earthmoving machine that flattens a
//! customizable rectangular site in front of it (`width × depth`, shifted laterally by `offset`, flush with the ground
//! it sits on at `grade_y = pos.y - 1`).
//!
//! - **Customizable site area (`width`, `depth`, `offset`):** choose from presets or dial in any exact width
//!   (`MIN_WIDTH..=MAX_WIDTH`), depth (`MIN_DEPTH..=MAX_DEPTH`) and left/right shift (`-MAX_OFFSET..=MAX_OFFSET`) in
//!   the machine panel. Perimeter beacons, guide rails and boom reach adapt live.
//! - **100% block conservation:** every excavated natural earth block (dirt, grass→dirt, sand, stone, granite,
//!   sandstone, basalt) is tracked with its exact `BlockId` in a compact cut-and-fill hopper (`FILL_CAP = 64`) for
//!   filling hollows; anything past the hopper cap exits immediately out the rear belt, and once no hollows remain on
//!   the site the hopper flushes completely onto the rear output belt (and returns all held blocks if broken).
//! - **Spare ore deposits (`spare_ores`):** toggleable in the machine's panel. When enabled, ore deposits (and clay /
//!   kaolin beds) above and at grade are skipped and preserved intact for miners; when disabled, they are mined
//!   straight to the rear output belt.
//! - **Choose floor material (`floor`):** pick `AIR` (`Natural`) to level ground without paving, or any solid building
//!   or terrain block (`CONCRETE`, `STONE_BRICKS`, `CLAY_BRICKS`, `PLANKS`, `DIRT`, `STONE`, `SAND`, `SLAG`, …) to
//!   surface the finished pad flush at `grade_y` from the left intake port (or from matching earth in its own hopper).

use crate::block::{
    self, BlockId, AIR, CLAY_BRICKS, CONCRETE, DIRT, PLANKS, SAND, SITE_GRADER, SLAG, STONE, STONE_BRICKS,
};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::math::IVec3;
use crate::sim::SimEvent;
use crate::world::World;

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::super::power::NOT_WIRED;
use super::super::DIRS;
use super::{Energy, Extra, Pick, ProcessSpec, ProcessTier, Processor, Status};

mod view;
mod work;

#[cfg(test)]
mod tests;

pub(super) use view::draw_grader;
pub(super) use work::is_paving_block;

use view::PARTS;

/// kW drawn while the bucket wheel is actively cutting, filling or paving.
pub const GRADER_KW: u32 = 45;
/// Default site dimensions (19 wide × 18 deep, centred on the 3×3 turntable).
pub const DEFAULT_WIDTH: u8 = 19;
pub const DEFAULT_DEPTH: u8 = 18;
/// Customizable site area bounds.
pub const MIN_WIDTH: u8 = 5;
pub const MAX_WIDTH: u8 = 41;
pub const MIN_DEPTH: u8 = 5;
pub const MAX_DEPTH: u8 = 36;
pub const MAX_OFFSET: i8 = 20;
/// How far above `grade_y` the boom reaches to cut ridges/trees, and how deep below `grade_y` it fills hollows.
pub const MAX_CUT_HEIGHT: i32 = 18;
pub const MAX_FILL_DEPTH: i32 = 10;
/// Working buffer capacity of the internal cut-and-fill hopper (1 stack; excess flows straight to the rear belt, and
/// any remainder flushes out once all hollows are filled).
pub const FILL_CAP: u16 = 64;
/// Curated floor materials offered in the Site Grader panel (`AIR` = keep natural ground).
pub const FLOOR_CHOICES: &[BlockId] = &[AIR, CONCRETE, STONE_BRICKS, CLAY_BRICKS, PLANKS, DIRT, STONE, SAND, SLAG];
/// Seconds per block cut/fill/pave at full power, and seconds to fell a whole tree.
pub(super) const WORK_SECONDS: f64 = 0.20;
pub(super) const FELL_SECONDS: f64 = 0.55;
/// Ticks to wait after a clean pass before checking the site again (5 s at 20 TPS).
pub(super) const REST_TICKS: u16 = 100;

/// Self-describing state header written by `Grader::write_state` so prototype demo saves upgrade cleanly.
const STATE_MAGIC: u16 = 0xFFFF;
const STATE_VER: u8 = 3;

const GRADER_PORTS: &[Port] = &[
    Port { side: Side::Left, role: Role::In, cell: Which::Nth(1) },
    Port { side: Side::Front, role: Role::Out, cell: Which::Nth(1) },
];

pub const GRADER_SPEC: ProcessSpec = ProcessSpec {
    block: SITE_GRADER,
    categories: &[],
    pick: Pick::Grade,
    // Keep 8 output slots for compatibility with earlier demo saves; belts drain 1 item/tick.
    buffers: [4, 0, 8],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: GRADER_KW }],
    footprint: Footprint { size: [3, 3, 2], ports: GRADER_PORTS },
    verb: "Grading",
    products: "spoil",
    waiting: "Site graded and level",
    map_colour: 0xd9a426,
    compute: 0,
    parts: &PARTS,
};

/// Active operation of the bucket wheel (`Grader::active`). Saved as `u8`.
pub(super) const ACT_IDLE: u8 = 0;
pub(super) const ACT_CUT: u8 = 1;
pub(super) const ACT_FILL: u8 = 2;
pub(super) const ACT_PAVE: u8 = 3;
pub(super) const ACT_FELL: u8 = 4;

/// Full settings and live earthworks telemetry returned to the panel API.
pub struct GraderInfo {
    pub spare_ores: bool,
    pub floor: BlockId,
    pub hopper: u16,
    pub cut_count: u32,
    pub fill_count: u32,
    pub paved_count: u32,
    pub trees_felled: u32,
    pub width: u8,
    pub depth: u8,
    pub offset: i8,
    pub dir: u8,
    pub anchor: IVec3,
    pub active: u8,
    pub cur: IVec3,
}

pub struct Grader {
    /// Current serpentine column index `0..(width * depth)`.
    pub col: u16,
    /// World cell currently targeted by the bucket wheel.
    pub cur: IVec3,
    /// Previous world cell targeted (for smooth boom slew & telescoping interpolation).
    pub prev: IVec3,
    /// Current action (`ACT_IDLE`, `ACT_CUT`, `ACT_FILL`, `ACT_PAVE`, `ACT_FELL`).
    pub active: u8,
    /// Customizable site width across (`MIN_WIDTH..=MAX_WIDTH`).
    pub width: u8,
    /// Customizable site depth ahead (`MIN_DEPTH..=MAX_DEPTH`).
    pub depth: u8,
    /// Customizable lateral shift left/right (`-MAX_OFFSET..=MAX_OFFSET`).
    pub offset: i8,
    /// Whether ore deposits (and clay/kaolin beds) are spared instead of excavated.
    pub spare_ores: bool,
    /// Chosen finished floor material at `grade_y` (`AIR` = keep natural ground, no surface replacement).
    pub floor: BlockId,
    /// Exact typed natural earth blocks temporarily held for cut-and-fill (sum <= `FILL_CAP`).
    pub hopper: Vec<(BlockId, u16)>,
    /// Live counters for the readout & panel.
    pub cut_count: u32,
    pub fill_count: u32,
    pub paved_count: u32,
    pub trees_felled: u32,
    /// Ticks remaining before re-scanning after a completed pass.
    pub rest: u16,
    /// Whether any work was performed during the current sweep pass.
    pub pass_worked: bool,
    /// Derived: whether the latest pass left cells waiting for the chosen floor material.
    pub needs_floor: bool,
}

impl Default for Grader {
    fn default() -> Grader {
        Grader {
            col: 0,
            cur: IVec3::ZERO,
            prev: IVec3::ZERO,
            active: ACT_IDLE,
            width: DEFAULT_WIDTH,
            depth: DEFAULT_DEPTH,
            offset: 0,
            spare_ores: true,
            floor: AIR,
            hopper: Vec::new(),
            cut_count: 0,
            fill_count: 0,
            paved_count: 0,
            trees_felled: 0,
            rest: 0,
            pass_worked: false,
            needs_floor: false,
        }
    }
}

impl Grader {
    pub fn columns(&self) -> u16 {
        u16::from(self.width).saturating_mul(u16::from(self.depth)).max(1)
    }

    /// Lateral bounds `(left_col, right_col)` inclusive, relative to the footprint's centre column (`pos + right`).
    pub fn across_bounds(&self) -> (i32, i32) {
        let w = i32::from(self.width);
        let left = -(w / 2) + i32::from(self.offset);
        (left, left + w - 1)
    }

    /// Total blocks currently held in the internal cut-and-fill hopper.
    pub fn hopper_total(&self) -> u16 {
        self.hopper.iter().map(|&(_, n)| n).sum()
    }

    /// Adds one exact natural earth block to the typed cut-and-fill hopper.
    pub fn hopper_push(&mut self, block: BlockId) {
        if let Some((_, n)) = self.hopper.iter_mut().find(|(b, _)| *b == block) {
            *n = n.saturating_add(1);
        } else {
            self.hopper.push((block, 1));
        }
    }

    /// Removes one block from the typed cut-and-fill hopper (preferring `prefer` if present, else the top entry).
    pub fn hopper_pop(&mut self, prefer: Option<BlockId>) -> Option<BlockId> {
        let idx = prefer
            .and_then(|want| self.hopper.iter().position(|&(b, n)| b == want && n > 0))
            .or_else(|| self.hopper.iter().rposition(|&(_, n)| n > 0))?;
        let (b, n) = &mut self.hopper[idx];
        let block = *b;
        *n -= 1;
        if *n == 0 {
            self.hopper.remove(idx);
        }
        Some(block)
    }

    /// The block texture shown on top of the 3D hopper gauge.
    pub fn hopper_top_block(&self) -> BlockId {
        self.hopper.last().map_or(DIRT, |&(b, _)| b)
    }

    /// Updates user-configurable options and restarts the sweep immediately if anything changed.
    pub fn configure(&mut self, spare_ores: bool, floor: BlockId, width: u8, depth: u8, offset: i8) {
        let valid_floor = floor == AIR || is_paving_block(ItemId::block(floor));
        if !valid_floor {
            return;
        }
        let width = width.clamp(MIN_WIDTH, MAX_WIDTH);
        let depth = depth.clamp(MIN_DEPTH, MAX_DEPTH);
        let offset = offset.clamp(-MAX_OFFSET, MAX_OFFSET);
        if self.spare_ores != spare_ores
            || self.floor != floor
            || self.width != width
            || self.depth != depth
            || self.offset != offset
        {
            self.spare_ores = spare_ores;
            self.floor = floor;
            self.width = width;
            self.depth = depth;
            self.offset = offset;
            self.col = 0;
            self.rest = 0;
            self.active = ACT_IDLE;
            self.pass_worked = false;
            self.needs_floor = false;
        }
    }

    /// World `(x, z)` column for serpentine index `col`, anchored on a machine at `pos` facing `dir`.
    pub fn column_pos(&self, pos: IVec3, dir: u8, col: u16) -> IVec3 {
        let right = DIRS[(dir as usize + 1) % 4];
        let fwd = DIRS[dir as usize % 4];
        let w = i32::from(self.width).max(1);
        let c = i32::from(col % self.columns());
        let row = c / w;
        let idx = c % w;
        let (left, right_edge) = self.across_bounds();
        let across = if row % 2 == 0 { left + idx } else { right_edge - idx };
        // Footprint centre column is `pos + right * 1`; site starts right past the 3-deep base (`ahead = 3`).
        let centre_col = pos + right;
        centre_col + fwd * (3 + row) + right * across
    }

    pub(super) fn write_state(&self, w: &mut ByteWriter) {
        w.u16(STATE_MAGIC);
        w.u8(STATE_VER);
        w.u16(self.col);
        for v in [self.cur, self.prev] {
            w.i32(v.x);
            w.i32(v.y);
            w.i32(v.z);
        }
        w.u8(self.active);
        w.u8(self.width);
        w.u8(self.depth);
        w.u8(self.offset as u8);
        w.bool(self.spare_ores);
        w.block(self.floor);
        w.u8(self.hopper.len().min(u8::MAX as usize) as u8);
        for &(b, n) in self.hopper.iter().take(u8::MAX as usize) {
            w.block(b);
            w.u16(n);
        }
        w.u32(self.cut_count);
        w.u32(self.fill_count);
        w.u32(self.paved_count);
        w.u32(self.trees_felled);
        w.u16(self.rest);
        w.bool(self.pass_worked);
    }

    pub(super) fn read_state(r: &mut ByteReader) -> Option<Grader> {
        if r.grader_v2_layout {
            return Self::read_v2_state(r);
        }
        let head = r.u16()?;
        if head == STATE_MAGIC {
            let ver = r.u8()?;
            if ver == 2 {
                return Self::read_v2_state(r);
            }
            if ver != STATE_VER {
                return None;
            }
            let col = r.u16()?;
            let cur = IVec3::new(r.i32()?, r.i32()?, r.i32()?);
            let prev = IVec3::new(r.i32()?, r.i32()?, r.i32()?);
            let active = r.u8()?;
            let width = r.u8()?;
            let depth = r.u8()?;
            let offset = r.u8()? as i8;
            let spare_ores = r.bool()?;
            let floor = r.block()?;
            let hlen = usize::from(r.u8()?);
            let mut hopper = Vec::with_capacity(hlen);
            let mut total = 0u32;
            for _ in 0..hlen {
                let b = r.block()?;
                let n = r.u16()?;
                if n > 0 {
                    total = total.saturating_add(u32::from(n));
                    hopper.push((b, n));
                }
            }
            let cut_count = r.u32()?;
            let fill_count = r.u32()?;
            let paved_count = r.u32()?;
            let trees_felled = r.u32()?;
            let rest = r.u16()?;
            let pass_worked = r.bool()?;
            let cols = u16::from(width).saturating_mul(u16::from(depth));
            let ok = (MIN_WIDTH..=MAX_WIDTH).contains(&width)
                && (MIN_DEPTH..=MAX_DEPTH).contains(&depth)
                && (-MAX_OFFSET..=MAX_OFFSET).contains(&offset)
                && col < cols
                && active <= ACT_FELL
                && total <= u32::from(FILL_CAP);
            return ok.then_some(Grader {
                col,
                cur,
                prev,
                active,
                width,
                depth,
                offset,
                spare_ores,
                floor,
                hopper,
                cut_count,
                fill_count,
                paved_count,
                trees_felled,
                rest,
                pass_worked,
                needs_floor: false,
            });
        }

        // Backward-compatible upgrade for v1 prototype demo saves (`col` directly, single `fill` + `fill_block`).
        let col = head;
        let cur = IVec3::new(r.i32()?, r.i32()?, r.i32()?);
        let prev = IVec3::new(r.i32()?, r.i32()?, r.i32()?);
        let active = r.u8()?;
        let fill = r.u16()?;
        let fill_block = r.block()?;
        let cut_count = r.u32()?;
        let fill_count = r.u32()?;
        let paved_count = r.u32()?;
        let trees_felled = r.u32()?;
        let rest = r.u16()?;
        let pass_worked = r.bool()?;
        let cols = u16::from(DEFAULT_WIDTH) * u16::from(DEFAULT_DEPTH);
        let ok = col < cols && active <= ACT_FELL && fill <= 512;
        if !ok {
            return None;
        }
        let mut hopper = Vec::new();
        let kept = fill.min(FILL_CAP);
        if kept > 0 {
            hopper.push((if fill_block == AIR { DIRT } else { fill_block }, kept));
        }
        Some(Grader {
            col,
            cur,
            prev,
            active,
            width: DEFAULT_WIDTH,
            depth: DEFAULT_DEPTH,
            offset: 0,
            spare_ores: true,
            floor: CONCRETE,
            hopper,
            cut_count,
            fill_count,
            paved_count,
            trees_felled,
            rest,
            pass_worked,
            needs_floor: false,
        })
    }

    /// Reads the v2 prototype layout (`spare_ores`, `floor`, typed `hopper`, fixed `DEFAULT_WIDTH × DEFAULT_DEPTH`).
    fn read_v2_state(r: &mut ByteReader) -> Option<Grader> {
        let col = r.u16()?;
        let cur = IVec3::new(r.i32()?, r.i32()?, r.i32()?);
        let prev = IVec3::new(r.i32()?, r.i32()?, r.i32()?);
        let active = r.u8()?;
        let spare_ores = r.bool()?;
        let floor = r.block()?;
        let hlen = usize::from(r.u8()?);
        let mut hopper = Vec::with_capacity(hlen);
        let mut total = 0u32;
        for _ in 0..hlen {
            let b = r.block()?;
            let n = r.u16()?;
            if n > 0 {
                total = total.saturating_add(u32::from(n));
                hopper.push((b, n));
            }
        }
        let cut_count = r.u32()?;
        let fill_count = r.u32()?;
        let paved_count = r.u32()?;
        let trees_felled = r.u32()?;
        let rest = r.u16()?;
        let pass_worked = r.bool()?;
        let cols = u16::from(DEFAULT_WIDTH) * u16::from(DEFAULT_DEPTH);
        let ok = col < cols && active <= ACT_FELL && total <= u32::from(FILL_CAP);
        ok.then_some(Grader {
            col,
            cur,
            prev,
            active,
            width: DEFAULT_WIDTH,
            depth: DEFAULT_DEPTH,
            offset: 0,
            spare_ores,
            floor,
            hopper,
            cut_count,
            fill_count,
            paved_count,
            trees_felled,
            rest,
            pass_worked,
            needs_floor: false,
        })
    }
}

impl Processor {
    /// One tick of a Site Grader at `share` of full power (nothing for other processors).
    pub(crate) fn grade(
        &mut self,
        world: &mut World,
        share: u32,
        changed: &mut Vec<(IVec3, BlockId)>,
        events: &mut Vec<SimEvent>,
    ) {
        let work_share = share * self.stats().speed / 1000 * self.boost / 1000;
        let Processor { pos, dir, extra, input, out, progress, made, used, status, .. } = self;
        let Extra::Grader(grader) = extra else { return };
        let mut ctx = work::Site {
            pos: *pos,
            dir: *dir,
            world,
            input,
            out,
            progress,
            made,
            used,
            changed,
            events,
            share: work_share,
        };
        *status = grader.tick(&mut ctx);
    }

    /// Whether the grader draws power this tick.
    pub(super) fn grader_wants_power(&self) -> bool {
        let Extra::Grader(g) = &self.extra else { return false };
        (g.rest == 0 || g.hopper_total() > 0) && self.status != Status::OutputFull
    }

    /// Whether this Site Grader accepts `item` at its left paving intake port.
    pub(super) fn grader_takes(&self, item: ItemId) -> bool {
        let Extra::Grader(g) = &self.extra else { return false };
        g.floor != AIR && item.places() == Some(g.floor)
    }

    /// Every earth block still sitting in the cut-and-fill hopper (returned on dismantle — zero loss!).
    pub(super) fn grader_contents(&self) -> Vec<Stack> {
        let Extra::Grader(g) = &self.extra else { return Vec::new() };
        g.hopper
            .iter()
            .filter_map(|&(b, n)| {
                let item = ItemId::block(b);
                (n > 0 && item.is_valid()).then_some(Stack { item, count: u32::from(n) })
            })
            .collect()
    }

    /// Live settings & counters for the Site Grader panel (`None` for other processors).
    pub fn grader_info(&self) -> Option<GraderInfo> {
        let Extra::Grader(g) = &self.extra else { return None };
        Some(GraderInfo {
            spare_ores: g.spare_ores,
            floor: g.floor,
            hopper: g.hopper_total(),
            cut_count: g.cut_count,
            fill_count: g.fill_count,
            paved_count: g.paved_count,
            trees_felled: g.trees_felled,
            width: g.width,
            depth: g.depth,
            offset: g.offset,
            dir: self.dir,
            anchor: self.pos,
            active: g.active,
            cur: g.cur,
        })
    }

    /// Applies panel settings (`spare_ores`, `floor`, `width`, `depth`, `offset`) to a Site Grader.
    pub fn set_grader(&mut self, spare_ores: bool, floor: BlockId, width: u8, depth: u8, offset: i8) {
        if let Extra::Grader(g) = &mut self.extra {
            g.configure(spare_ores, floor, width, depth, offset);
            self.progress = 0;
        }
    }

    /// Readout status line for a Site Grader (`None` for other processors).
    pub(super) fn grader_text(&self) -> Option<String> {
        let Extra::Grader(g) = &self.extra else { return None };
        let floor_label = if g.floor == AIR { "Natural" } else { block::def(g.floor).name };
        let doing = match (self.status, g.active) {
            (Status::NoPower, _) => NOT_WIRED.to_string(),
            (Status::OutputFull, _) => "Output full: attach a belt to the rear port".to_string(),
            (_, ACT_CUT) => "Cutting hillside".to_string(),
            (_, ACT_FILL) => "Filling hollow from hopper".to_string(),
            (_, ACT_PAVE) => format!("Paving {floor_label}"),
            (_, ACT_FELL) => "Clearing timber".to_string(),
            _ if g.hopper_total() > 0 && g.rest > 0 => "Flushing earthworks hopper to output".to_string(),
            _ if g.needs_floor && g.floor != AIR => format!("Waiting for {floor_label} (left port)"),
            _ if g.rest > 0 => format!("Site level ({}×{})", g.width, g.depth),
            _ => "Scanning site".to_string(),
        };
        let ore_mode = if g.spare_ores { "sparing ores" } else { "mining ores" };
        let shift = if g.offset == 0 { String::new() } else { format!(" ({:+})", g.offset) };
        let mut stats = format!(
            "{}×{}{shift} · floor: {floor_label} · {ore_mode} · hopper {}/{FILL_CAP}",
            g.width,
            g.depth,
            g.hopper_total()
        );
        if g.cut_count > 0 || g.fill_count > 0 || g.paved_count > 0 {
            stats += &format!(" · cut {}, filled {}, paved {}", g.cut_count, g.fill_count, g.paved_count);
        }
        if g.trees_felled > 0 {
            stats += &format!(" · {} trees", g.trees_felled);
        }
        Some(format!("{doing} · {stats}"))
    }
}
