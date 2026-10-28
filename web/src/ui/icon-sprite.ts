// Flat item icons: a block drawn as a picture with see-through edges (a torch, a sapling; the engine marks them flat,
// `item_models/icon_frame.rs`) is drawn as that picture, trimmed to its visible pixels and scaled to fill the icon box
// like every other icon. `opaqueBounds` is pure (tested in web/test/icon-sprite.test.mjs); `drawSprite` draws.

/** Pixels fainter than this (alpha, 0–255) count as see-through when trimming. */
const ALPHA_MIN = 40;

/** The smallest rectangle `[x, y, w, h]` holding every visible pixel of RGBA `data`, or null when none is visible. */
export function opaqueBounds(data: ArrayLike<number>, width: number, height: number): number[] | null {
  let x0 = width, y0 = height, x1 = -1, y1 = -1;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      if (data[(y * width + x) * 4 + 3] < ALPHA_MIN) continue;
      x0 = Math.min(x0, x);
      x1 = Math.max(x1, x);
      y0 = Math.min(y0, y);
      y1 = Math.max(y1, y);
    }
  }
  return x1 < 0 ? null : [x0, y0, x1 - x0 + 1, y1 - y0 + 1];
}

/** Draws `img`'s visible part centred in a `box`-pixel icon, its longer side `fill` of the box. */
export function drawSprite(ctx: CanvasRenderingContext2D, img: HTMLCanvasElement, box: number, fill: number): void {
  const bounds = opaqueBounds(img.getContext('2d')!.getImageData(0, 0, img.width, img.height).data, img.width, img.height);
  if (!bounds) return;
  const [x, y, w, h] = bounds;
  const s = (box * fill) / Math.max(w, h);
  ctx.drawImage(img, x, y, w, h, (box - w * s) / 2, (box - h * s) / 2, w * s, h * s);
}
