// The marks painted onto an item icon (GEAR-ICONS: the outline says what, the chip says the tier), in the icon's own
// pixels so every panel that shows the icon shows them too (`hud.ts` itemIcon caches the result):
// - a tier chip, bottom left (the slot's key label is top left, its count bottom right): a square in the tier's kit
//   colour (`item_tier_colour`, so the colour-blind palette applies) with the Mk number, so colour is never the only cue;
// - a dashed corner, top right, on items that work from the left hand (`item_left_hand`; the left-hand slot is dashed).
// To mark something new: a painter here, called from `itemIcon`.

/** The chip's side as a share of the icon: 15 px in a 40 px slot icon, 12 px at the 32 px the icon check renders. */
const CHIP = 0.38;
const DARK = '#10131a';

/** Paints the tier chip for tier `tier` (0 is Mk1) in colour `colour` (0xRRGGBB) on a `px`-pixel icon. */
export function paintTierChip(ctx: CanvasRenderingContext2D, px: number, tier: number, colour: number): void {
  const side = Math.round(px * CHIP), y = px - side;
  ctx.fillStyle = DARK;
  ctx.fillRect(0, y, side, side);
  ctx.fillStyle = `#${colour.toString(16).padStart(6, '0')}`;
  ctx.fillRect(2, y + 2, side - 4, side - 4);
  ctx.font = `bold ${Math.round(side * 0.72)}px system-ui, sans-serif`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.lineWidth = Math.max(2, side / 8);
  ctx.strokeStyle = DARK;
  const [cx, cy] = [side / 2, y + side / 2 + 1];
  ctx.strokeText(String(tier + 1), cx, cy);
  ctx.fillStyle = '#ffffff';
  ctx.fillText(String(tier + 1), cx, cy);
}

/** Paints the left hand's dashed corner on a `px`-pixel icon. */
export function paintLeftMark(ctx: CanvasRenderingContext2D, px: number): void {
  const arm = Math.round(px * 0.34), w = Math.max(2, Math.round(px / 20)), inset = w;
  ctx.save();
  ctx.beginPath();
  ctx.moveTo(px - inset - arm, inset);
  ctx.lineTo(px - inset, inset);
  ctx.lineTo(px - inset, inset + arm);
  ctx.strokeStyle = DARK;
  ctx.lineWidth = w + 2;
  ctx.setLineDash([]);
  ctx.stroke();
  ctx.strokeStyle = '#cfd6e2';
  ctx.lineWidth = w;
  ctx.setLineDash([w * 1.5, w]);
  ctx.stroke();
  ctx.restore();
}
