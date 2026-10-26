//! Placing and wiring power poles and cables, part of the local player's hands (`interaction.rs`).
//! Presentation and input only: everything it builds is an ordinary action (`PlaceBlock`, `Connect`,
//! `Disconnect`), so the core, co-op and saves see plain placements and wires.
//!
//! **Poles.** With a pole in hand a click places it where you aim, if that is within the pole's `link` of the
//! pole to chain from (the selected one, else the last placed, else the nearest to the aim, all near the
//! player); further off, at the widest spacing towards the aim, standing on the ground (`plan_pole`). A pole needs
//! ground under it or a wall beside it (`factory/pole_mount.rs`); against a wall it is bolted to it.
//! Holding Shift snaps to the *full reach* in the direction the player faces instead, and holding the use
//! button with Shift down keeps placing at full reach whenever that spot comes within `HOLD_REACH` of the
//! player, so walking along lays a line. A new pole wires itself to the nearest powered pole in range
//! (`Factory::auto_hook`) and is selected; nothing else is wired for you.
//!
//! **Wiring** (`wire.rs`): right-click a pole to select it; then aiming at an unpowered machine offers
//! to wire it, and a click does. Crouch-click moves, cuts and links poles.
//!
//! **Cables** (`cables.rs`): with cables in hand a click hangs them: the aimed cell and every free cell below it,
//! down to the ground or the stack's size, built a few per tick. Crouching places one.
//!
//! To change the spacing rule: [`plan_pole`]. The preview: a ghost pole, wire and cables
//! (`write_power_preview`), the outline boxes (`power_boxes`) and the label (`power_label`), which
//! `api/hud.rs` forwards. A long-distance pole is placed like a machine; its preview is `long_pole.rs`.

mod cables;
mod long_pole;
mod wire;

use crate::action::Action;
use crate::block::{self, tex, BlockId, AIR, POLE, SOLID};
use crate::factory::{self, pole_mount, tiers, POLE_TIERS};
use crate::interaction::{PLACE_REPEAT_SECONDS, REACH};
use crate::math::{IVec3, Vec3};
use crate::Game;

/// A held button places the next pole only when its spot is this close to the player.
const HOLD_REACH: f64 = 6.0;
/// How far above or below its line a pole's ground is searched for, in blocks.
const CLIMB: i32 = 8;
/// Ticks a just-placed pole counts as standing before the core has it (a co-op client's round trip).
const PENDING_TICKS: u32 = 90;
/// Pointing this close to a pole (squared blocks) picks it as the one to chain from.
const POINT_AT2: i32 = 4;
const UP: IVec3 = IVec3::new(0, 1, 0);
/// Outline colours: the next pole or cable, the selected pole, something in the way, a wire to cut.
const GHOST_GREEN: i32 = 0x4ade80;
const ANCHOR_BLUE: i32 = 0x7dd3fc;
const BLOCKED_RED: i32 = 0xff4a3d;
const CUT_AMBER: i32 = 0xfbbf24;

/// The player's power tools: input state only, never saved.
#[derive(Default)]
pub struct PowerTools {
    /// The use button has been down since the last press was handled.
    down: bool,
    /// The press that began a hold with cables was a single placement (nothing to drop).
    single: bool,
    /// The pole that machines and poles are wired to (`wire.rs`).
    selected: Option<IVec3>,
    /// The last pole this tool sent, its tier and the ticks since (it counts before the core has it).
    last: Option<(IVec3, u8, u32)>,
    /// Cables of a drop still to send (last first), and the hotbar slot and item they come from.
    drop: Vec<IVec3>,
    drop_from: (u8, u16),
    /// Whether the held long-distance pole's spot linked when last looked at (`long_pole.rs`'s edge cue).
    long_cue: Option<bool>,
}

/// The next pole: where, the pole it is placed from, its tier, and how it stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoleGhost {
    pub pos: IVec3,
    pub anchor: IVec3,
    pub tier: u8,
    /// The reach between the two poles, in blocks.
    pub link: i32,
    /// It stands at the full reach (snapped or clamped), not somewhere nearer.
    pub full: bool,
    /// Its cell is free.
    pub free: bool,
    /// Something solid is under it or beside it; the core refuses a pole in open air (`pole_mount.rs`).
    pub held_up: bool,
    /// The wall it would hang on (`pole_mount::wall_for`), 0 when it stands on the ground.
    pub wall: u8,
}

fn dist2(a: IVec3, b: IVec3) -> i32 {
    let d = a - b;
    d.x * d.x + d.y * d.y + d.z * d.z
}

/// A cell an item can stand on: air over a solid block.
fn standing(block: &impl Fn(IVec3) -> Option<BlockId>, c: IVec3) -> bool {
    block(c) == Some(AIR) && block(c - UP).is_some_and(|b| SOLID[b as usize])
}

/// Where the next pole from `anchor` goes. Free (`snap` off): `aim` if it is within `link`, else the standing
/// cell nearest the full reach towards `aim`. Snapped: the standing cell nearest the full reach along `look`
/// (the horizontal view; towards `aim` when it points nowhere). Returns the cell and whether it is at the
/// full reach; `None` when no ground is found.
pub fn plan_pole(
    block: impl Fn(IVec3) -> Option<BlockId>,
    anchor: IVec3,
    aim: IVec3,
    look: Vec3,
    link: i32,
    snap: bool,
) -> Option<(IVec3, bool)> {
    if !snap && dist2(aim, anchor) <= link * link {
        return Some((aim, false));
    }
    let unit = |v: Vec3| (v.length() > 1e-6).then(|| v * (1.0 / v.length()));
    let level = Vec3::new(look.x, 0.0, look.z);
    let dir = unit(level).filter(|_| snap && level.length() > 0.3).or_else(|| unit((aim - anchor).as_vec3()))?;
    let centre = anchor.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
    for r in (1..=link).rev() {
        let c0 = (centre + dir * r as f64).floor();
        for step in 0..=2 * CLIMB {
            let dy = if step % 2 == 1 { (step + 1) / 2 } else { -(step / 2) };
            let c = c0 + UP * dy;
            if standing(&block, c) && dist2(c, anchor) <= link * link {
                return Some((c, true));
            }
        }
    }
    None
}

impl Game {
    /// The tier of the pole in the selected slot (a tier item or the base pole), if one is held.
    fn held_pole_tier(&self) -> Option<u8> {
        let stack = self.inventory().selected_stack();
        if stack.is_empty() {
            return None;
        }
        match tiers::placed_by(stack.item) {
            Some((POLE, tier)) => Some(tier),
            _ => (stack.item.places() == Some(POLE)).then_some(0),
        }
    }

    /// The cell aimed at: in front of the face pointed at, or a point `REACH` along the view.
    pub(crate) fn aim_cell(&self) -> IVec3 {
        match self.target {
            Some(hit) => hit.block + hit.normal,
            None => (self.body().eye() + self.body().look_dir() * REACH).floor(),
        }
    }

    /// The pole the next one is placed from, near enough to the player to matter: the one pointed at, else the
    /// selected one, else the last one placed, else the one nearest the aim.
    fn pole_anchor(&self, aim: IVec3, held_link: i32) -> Option<(IVec3, u8)> {
        let eye = self.body().eye();
        let last = self.tools.last.map(|p| (p.0, p.1));
        let standing = |&(pos, _): &(IVec3, u8)| {
            self.sim.factory.poles().any(|p| p.0 == pos)
                || self.tools.last.is_some_and(|l| l.0 == pos && l.2 < PENDING_TICKS)
        };
        let near = |&(pos, tier): &(IVec3, u8)| {
            let link = POLE_TIERS[tier as usize].link.max(held_link) as f64;
            (pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye).length() <= link + REACH + 1.0
        };
        let poles = || self.sim.factory.poles().chain(last).filter(near);
        let pointed = poles().filter(|&(pos, _)| dist2(pos, aim) <= POINT_AT2).min_by_key(|&(pos, _)| dist2(pos, aim));
        let chosen = self.selected_pole().and_then(|s| poles().find(|p| p.0 == s));
        pointed
            .or(chosen)
            .or(last.filter(standing).filter(near))
            .or_else(|| poles().min_by_key(|&(pos, _)| dist2(pos, aim)))
    }

    /// Where the held pole would go now: `None` when none is held, a machine or pole is aimed at (wiring), or
    /// no pole is near enough to place it from (the first pole goes where you point).
    pub(crate) fn pole_ghost(&self) -> Option<PoleGhost> {
        let tier = self.held_pole_tier()?;
        if self.wire_aim().is_some() {
            return None;
        }
        let held_link = POLE_TIERS[tier as usize].link;
        let aim = self.aim_cell();
        let (anchor, anchor_tier) = self.pole_anchor(aim, held_link)?;
        let link = POLE_TIERS[anchor_tier as usize].link.max(held_link);
        let look = self.body().look_dir();
        let world = &self.sim.world;
        let (pos, full) = plan_pole(|p| world.get_block(p), anchor, aim, look, link, self.body().input.sprint)?;
        let free = world.get_block(pos).is_some_and(block::replaceable);
        let solid = |p: IVec3| world.get_block(p).is_some_and(|b| SOLID[b as usize]);
        let held_up = pole_mount::is_held_up(solid, pos);
        let wall = pole_mount::wall_for(solid, pos, self.pole_against(pos));
        Some(PoleGhost { pos, anchor, tier, link, full, free, held_up, wall })
    }

    /// The block a pole at `pos` counts as clicked against: the wall aimed at when the pole goes right in front of
    /// it, else the ground under it. The core picks that wall to hang the pole on when it has one.
    fn pole_against(&self, pos: IVec3) -> IVec3 {
        match self.target {
            Some(hit) if hit.normal != IVec3::ZERO && hit.normal.y == 0 && hit.block + hit.normal == pos => hit.block,
            _ => pos - UP,
        }
    }

    /// Runs the power tools for one tick. True while they own the use button (wiring, poles, or a cable
    /// drop); false leaves plain placement and machine panels to `interaction.rs`.
    pub(crate) fn update_power_tools(&mut self, dt: f32) -> bool {
        if let Some(last) = self.tools.last.as_mut() {
            last.2 = last.2.saturating_add(1);
        }
        self.update_long_pole_cue();
        let sending = self.send_cable_drop();
        if !self.using {
            self.tools.down = false;
            return sending;
        }
        let crouch = self.body().input.crouch;
        if sending {
            return true;
        }
        let edge = !self.tools.down;
        if self.wire_aim().is_some() {
            // One wiring click per press; a held button must not go on to cut what it just wired.
            let done = edge && self.wire_click(crouch);
            self.tools.down |= done;
            if done || !edge {
                return true;
            }
        }
        let panel = |id| factory::machine(id).is_some_and(|m| m.panel || m.slots > 0);
        if self.target.is_some_and(|h| !crouch && panel(h.id)) {
            return false;
        }
        self.tools.down = true;
        if self.holds_cable() {
            if edge {
                let cells = self.cable_drop();
                self.tools.single = crouch || cells.len() < 2;
                if !self.tools.single {
                    let inv = self.inventory();
                    self.tools.drop_from = (inv.selected as u8, inv.selected_stack().item.0);
                    self.tools.drop = cells.into_iter().rev().collect();
                    self.using = false;
                }
            }
            return !self.tools.single;
        }
        let Some(tier) = self.held_pole_tier() else { return false };
        self.use_cooldown -= dt;
        if self.use_cooldown > 0.0 {
            return true;
        }
        let Some(ghost) = self.pole_ghost() else {
            // No pole near: the first one goes where you point.
            let hit = self.target.filter(|h| h.normal != IVec3::ZERO);
            let pos = hit.map(|h| h.block + h.normal);
            let world = &self.sim.world;
            let solid = |p: IVec3| world.get_block(p).is_some_and(|b| SOLID[b as usize]);
            let fits =
                |p: IVec3| world.get_block(p).is_some_and(block::replaceable) && pole_mount::is_held_up(solid, p);
            if let Some(pos) = pos.filter(|&p| edge && fits(p)) {
                self.place_pole(pos, tier);
            }
            return true;
        };
        let centre = ghost.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
        let close = (centre - self.body().eye()).length() <= HOLD_REACH;
        let line = self.body().input.sprint && ghost.full && close;
        if ghost.free && ghost.held_up && (edge || line) {
            self.place_pole(ghost.pos, ghost.tier);
        }
        true
    }

    /// Sends a pole from the selected slot to `pos` and selects it, so it can be wired at once.
    fn place_pole(&mut self, pos: IVec3, tier: u8) {
        let slot = self.inventory().selected as u8;
        self.act(Action::PlaceBlock { pos, slot, facing: 0, against: self.pole_against(pos) });
        self.tools.last = Some((pos, tier, 0));
        self.tools.selected = Some(pos);
        self.use_cooldown = PLACE_REPEAT_SECONDS;
    }

    /// Outline boxes (7 numbers each: lowest and highest cells, colour) for the wiring, pole or cables about
    /// to be placed; empty when none applies.
    pub(crate) fn power_boxes(&self) -> Vec<i32> {
        let mut out = self.wire_boxes();
        let mut boxed = |lo: IVec3, hi: IVec3, colour: i32| out.extend([lo.x, lo.y, lo.z, hi.x, hi.y, hi.z, colour]);
        if let Some(g) = self.pole_ghost() {
            let ok = g.free && g.held_up;
            let hook = self.sim.factory.auto_hook(g.pos, g.tier).filter(|&h| ok && self.selected_pole() != Some(h));
            if let Some(hook) = hook {
                boxed(hook, hook, ANCHOR_BLUE);
            }
            boxed(g.pos, g.pos, if ok { GHOST_GREEN } else { BLOCKED_RED });
        } else if self.holds_cable() && !self.body().input.crouch {
            let cells = self.cable_drop();
            if let (Some(&top), Some(&bottom)) = (cells.first(), cells.last()) {
                boxed(bottom, top, GHOST_GREEN);
            }
        }
        out
    }

    /// The HUD lines while poles, wiring or cables are in hand and aimed: a title, then the details
    /// ("" otherwise).
    pub(crate) fn power_label(&self) -> String {
        if let Some(text) = self.wire_label() {
            return text;
        }
        if let Some(g) = self.pole_ghost() {
            let name = if g.tier == 0 { "Power pole".to_string() } else { format!("Power pole Mk{}", g.tier + 1) };
            if !g.free {
                return format!("{name}\nSomething is in the way");
            }
            if !g.held_up {
                return format!("{name}\nNeeds the ground under it or a wall beside it");
            }
            let far = f64::from(dist2(g.pos, g.anchor)).sqrt().round() as i32;
            let hook = match self.sim.factory.auto_hook(g.pos, g.tier) {
                Some(p) => format!(
                    "wires itself to the powered pole {} blocks away",
                    f64::from(dist2(g.pos, p)).sqrt().round()
                ),
                None => "no powered pole in range: wire it by hand".to_string(),
            };
            let how = if self.body().input.sprint { "release Shift: free placement" } else { "hold Shift: full reach" };
            return format!(
                "{name}\n{hook} · {far} of {} blocks from the last pole · right-click to place · {how}",
                g.link
            );
        }
        if self.holds_cable() && !self.body().input.crouch {
            let cells = self.cable_drop();
            let Some(&top) = cells.first() else { return String::new() };
            let n = cells.len();
            let joined =
                if self.sim.factory.cable_joins(top) { "joined to a pole" } else { "not joined to a pole yet" };
            if n == 1 {
                return format!("Power cable\n{joined} · right-click to place");
            }
            return format!("Power cable\n{n} hang from here · {joined} · right-click to lower, crouch to place one");
        }
        String::new()
    }

    /// Ghost pole and wire, or ghost cables, drawn with the machines.
    pub(crate) fn write_power_preview(&mut self, eye: Vec3) {
        let half = Vec3::new(0.5, 0.5, 0.5);
        let top = |p: IVec3| p.as_vec3() + Vec3::new(0.5, factory::POLE_WIRE_Y, 0.5) - eye;
        if let Some(g) = self.pole_ghost().filter(|g| g.free && g.held_up) {
            factory::preview_pole(&mut self.instances, g.tier, g.wall, g.pos.as_vec3() + half - eye);
            if let Some(hook) = self.sim.factory.auto_hook(g.pos, g.tier) {
                factory::preview_wire(&mut self.instances, top(hook), top(g.pos), tex::GRID_FIRST);
            }
        } else if self.holds_cable() && !self.body().input.crouch {
            let cells = self.cable_drop();
            for (i, &c) in cells.iter().enumerate() {
                let mut arms = [false; 6];
                arms[2] = i > 0;
                arms[3] = i + 1 < cells.len();
                factory::preview_cable(&mut self.instances, c.as_vec3() + half - eye, arms);
            }
        } else if let Some((from, to)) = self.wire_preview() {
            factory::preview_wire(&mut self.instances, from - eye, to - eye, tex::GRID_FIRST);
        }
        self.write_long_pole_preview(eye);
    }
}

#[cfg(test)]
mod tests;
