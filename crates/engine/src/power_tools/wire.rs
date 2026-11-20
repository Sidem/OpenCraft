//! The wiring hand, part of `power_tools.rs`: select a pole, then wire machines and other poles to it.
//! Active with a pole in hand or an empty hand.
//!
//! - Right-click a pole to select it (again to deselect); it stays selected while it stands and is near.
//!   A pole just placed is selected already.
//! - With a pole selected, aiming at a machine that takes power and has none suggests wiring it (a green
//!   outline and a wire); a click does. Crouch-click moves a machine that hangs on another pole, cuts a
//!   wire, or links two poles.
//! - What a click does is `Factory::hookup`; the effect is a `Connect` or `Disconnect` action.
//!
//! Everything else on aiming at things (labels, outlines, the preview wire) is derived from [`WireAim`].

use crate::action::Action;
use crate::block;
use crate::factory::{hud_keys, tier_stats, Hookup};
use crate::math::{IVec3, Vec3};
use crate::sound;
use crate::Game;

use super::{ANCHOR_BLUE, BLOCKED_RED, CUT_AMBER, GHOST_GREEN, PENDING_TICKS};

/// A selected pole is forgotten when the player is further than this from it, in blocks.
const SELECT_KEEP: f64 = 48.0;

/// What is aimed at while wiring.
pub(super) struct WireAim {
    /// The machine or pole's anchor cell.
    pub anchor: IVec3,
    /// It is a real pole (a plain click selects it).
    pub pole: bool,
    /// The selected pole, if any.
    pub sel: Option<IVec3>,
    /// What wiring it to the selected pole would do.
    pub hookup: Hookup,
    /// A plain click wires it (a machine with no power at all).
    pub plain: bool,
}

impl Game {
    /// Wiring is done with a pole (or long-distance pole) in hand or nothing in hand.
    pub(super) fn wiring_hand(&self) -> bool {
        self.inventory().selected_stack().is_empty() || self.held_pole_tier().is_some() || self.holds_long_pole()
    }

    /// The selected pole while it stands (or has just been placed) near the player.
    pub(super) fn selected_pole(&self) -> Option<IVec3> {
        let sel = self.tools.selected?;
        let placed = self.tools.last.is_some_and(|l| l.0 == sel && l.2 < PENDING_TICKS);
        let near = (sel.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - self.body().eye()).length() <= SELECT_KEEP;
        ((self.sim.factory.pole_tier(sel).is_some() || placed) && near).then_some(sel)
    }

    /// The pole or machine aimed at, if wiring has something to say about it.
    pub(super) fn wire_aim(&self) -> Option<WireAim> {
        if !self.wiring_hand() {
            return None;
        }
        let cell = self.target?.block;
        let f = &self.sim.factory;
        let anchor = f.anchor_of(cell)?;
        let pole = f.pole_slots(anchor).is_some();
        let sel = self.selected_pole();
        let hookup = sel.map_or(Hookup::Nothing, |s| f.hookup(s, cell));
        if !pole && hookup == Hookup::Nothing {
            return None;
        }
        let plain = pole || (hookup == Hookup::Connect && !f.is_wired(cell));
        Some(WireAim { anchor, pole, sel, hookup, plain })
    }

    /// Handles a press aimed at something wireable. True when it did something (the press is used up).
    pub(super) fn wire_click(&mut self, crouch: bool) -> bool {
        let Some(aim) = self.wire_aim() else { return false };
        let used = self.sim.factory.slots_used(aim.anchor);
        let act = match (crouch, aim.sel, aim.hookup) {
            (true, Some(pole), Hookup::Connect | Hookup::Move(_)) => Action::Connect { pole, to: aim.anchor },
            (true, Some(pole), Hookup::Disconnect) => Action::Disconnect { pole, to: aim.anchor },
            (true, sel, _) if aim.pole && (sel.is_none() || sel == Some(aim.anchor)) && used > 0 => {
                Action::Disconnect { pole: aim.anchor, to: aim.anchor }
            }
            (false, _, _) if aim.pole => {
                self.tools.selected = Some(aim.anchor).filter(|&a| aim.sel != Some(a));
                self.wire_sound(aim.anchor);
                return true;
            }
            (false, Some(pole), Hookup::Connect) if aim.plain => Action::Connect { pole, to: aim.anchor },
            _ => return false,
        };
        self.act(act);
        self.wire_sound(aim.anchor);
        true
    }

    /// R key while aiming at a pole or wired machine with a pole selected: connects, moves or severs the wire
    /// (or severs every wire on the selected pole when aiming at it). True when it handled the key.
    pub(crate) fn wire_toggle(&mut self) -> bool {
        let Some(aim) = self.wire_aim() else { return false };
        let Some(pole) = aim.sel else { return false };
        let act = match aim.hookup {
            Hookup::Connect | Hookup::Move(_) => Action::Connect { pole, to: aim.anchor },
            Hookup::Disconnect => Action::Disconnect { pole, to: aim.anchor },
            _ if aim.pole && aim.anchor == pole && self.sim.factory.slots_used(pole) > 0 => {
                Action::Disconnect { pole, to: pole }
            }
            _ => return false,
        };
        self.act(act);
        self.wire_sound(aim.anchor);
        true
    }

    fn wire_sound(&mut self, at: IVec3) {
        self.play(sound::PLACE, block::sound::METAL, at.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 0.4);
    }

    /// The selected pole's outline (blue), and the aimed thing's: green where a click wires it, amber where
    /// a crouch-click would cut it, red when it can't be wired.
    pub(super) fn wire_boxes(&self) -> Vec<i32> {
        let mut out = Vec::new();
        if !self.wiring_hand() {
            return out;
        }
        let mut boxed = |lo: IVec3, hi: IVec3, colour: i32| out.extend([lo.x, lo.y, lo.z, hi.x, hi.y, hi.z, colour]);
        if let Some(s) = self.selected_pole() {
            boxed(s, s, ANCHOR_BLUE);
        }
        if let (Some(aim), Some(hit)) = (self.wire_aim(), self.target) {
            let colour = match aim.hookup {
                Hookup::Connect | Hookup::Move(_) => GHOST_GREEN,
                Hookup::Disconnect => CUT_AMBER,
                Hookup::TooFar | Hookup::PoleFull | Hookup::OtherFull => BLOCKED_RED,
                Hookup::Nothing => return out,
            };
            let cells = self.sim.factory.machine_cells(hit.block);
            let lo = cells.iter().fold(cells[0], |m, c| IVec3::new(m.x.min(c.x), m.y.min(c.y), m.z.min(c.z)));
            let hi = cells.iter().fold(cells[0], |m, c| IVec3::new(m.x.max(c.x), m.y.max(c.y), m.z.max(c.z)));
            boxed(lo, hi, colour);
        }
        out
    }

    /// The wire a click would make or cut, as its two ends in the world and texture layer: grid colour when
    /// connecting or moving, red when severing (`Hookup::Disconnect`).
    pub(super) fn wire_preview(&self) -> Option<(Vec3, Vec3, u16)> {
        let aim = self.wire_aim()?;
        let layer = match aim.hookup {
            Hookup::Connect | Hookup::Move(_) => block::tex::GRID_FIRST,
            Hookup::Disconnect => block::tex::LAMP_RED,
            _ => return None,
        };
        let sel = aim.sel?;
        let f = &self.sim.factory;
        let sel_mid = f.pole_top(sel)?;
        let pole_mid = f.pole_top(aim.anchor).filter(|_| aim.pole);
        let lead = pole_mid.unwrap_or(aim.anchor.as_vec3() + Vec3::new(0.5, 0.7, 0.5));
        let end = if pole_mid.is_some() { f.pole_hang(aim.anchor, sel_mid)? } else { lead };
        Some((f.pole_hang(sel, lead)?, end, layer))
    }

    /// The HUD lines for the aimed pole or machine: status and control chips.
    pub(super) fn wire_label(&self) -> Option<String> {
        let aim = self.wire_aim()?;
        let f = &self.sim.factory;
        let slots = |p: IVec3| format!("{} / {} connections", f.slots_used(p), f.pole_slots(p).unwrap_or(0));
        let reach = |p: IVec3| tier_stats(f.pole_tier(p).unwrap_or(0)).reach;
        let text = match (aim.pole, aim.hookup) {
            (true, _) if aim.sel == Some(aim.anchor) => {
                let keys = if f.slots_used(aim.anchor) > 0 {
                    hud_keys(&[("RMB", "Deselect"), ("Ctrl+RMB/R", "Disconnect all")])
                } else {
                    hud_keys(&[("RMB", "Deselect")])
                };
                format!("Power pole (selected)\n{}\n{keys}", slots(aim.anchor))
            }
            (true, Hookup::Connect) => {
                format!("Power pole\n{}\n{}", slots(aim.anchor), hud_keys(&[("RMB", "Select"), ("Ctrl+RMB/R", "Link")]))
            }
            (true, Hookup::Disconnect) => format!(
                "Power pole\n{} · linked\n{}",
                slots(aim.anchor),
                hud_keys(&[("RMB", "Select"), ("Ctrl+RMB/R", "Sever wire")])
            ),
            (true, Hookup::TooFar) => {
                format!("Power pole\n{} · too far to link\n{}", slots(aim.anchor), hud_keys(&[("RMB", "Select")]))
            }
            (true, Hookup::PoleFull | Hookup::OtherFull) => {
                format!("Power pole\n{} · no free slot to link\n{}", slots(aim.anchor), hud_keys(&[("RMB", "Select")]))
            }
            (true, _) => {
                let keys = if f.slots_used(aim.anchor) > 0 {
                    hud_keys(&[("RMB", "Select to wire"), ("Ctrl+RMB", "Disconnect all")])
                } else {
                    hud_keys(&[("RMB", "Select to wire")])
                };
                format!("Power pole\n{}\n{keys}", slots(aim.anchor))
            }
            (false, Hookup::Connect) => {
                let s = aim.sel?;
                let key = if aim.plain { "RMB" } else { "Ctrl+RMB/R" };
                format!("Wire to selected pole\nPole: {}\n{}", slots(s), hud_keys(&[(key, "Connect")]))
            }
            (false, Hookup::Move(_)) => {
                format!("On another pole\n{}", hud_keys(&[("Ctrl+RMB/R", "Move wire here")]))
            }
            (false, Hookup::Disconnect) => {
                format!("Wired to selected pole\n{}", hud_keys(&[("Ctrl+RMB/R", "Disconnect")]))
            }
            (false, Hookup::TooFar) => format!("Out of reach\nSelected pole reaches {} blocks", reach(aim.sel?)),
            (false, Hookup::PoleFull | Hookup::OtherFull) => {
                "Pole full\nNo free connections on selected pole".to_string()
            }
            (false, Hookup::Nothing) => return None,
        };
        Some(text)
    }
}
