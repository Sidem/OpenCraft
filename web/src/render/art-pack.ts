// The art pack: hand-painted 128 × 128 layers made by `tools/art` (docs/ART_STYLE.md) that replace layers of the
// engine's own generated textures. `composeTextures` builds the array the renderer uploads: engine layers the pack
// doesn't cover are repeated 2 × 2 to 128 so every layer has one size, pack layers are copied in, and a second array
// holds each texel's self-light (glow). To add layers, extend `tools/art/registry.py` and rebuild the pack.

import glowUrl from '../art/pack_glow.png?url';
import table from '../art/pack.json';

export const ART_SIZE = 128;

// The colour stack ships as lossless WebP slices (`pack_<n>.webp`, 127 layers each: WebP images are at most 16383 rows).
const sliceUrls = Object.entries(import.meta.glob<string>('../art/pack_[0-9]*.webp', { query: '?url', import: 'default', eager: true }))
  .sort(([a], [b]) => a.localeCompare(b))
  .map(([, url]) => url);
/** Rows read back from the GPU at a time: any WebGL2 device accepts a texture this tall. */
const CHUNK_ROWS = 1024;

export interface ArtTextures {
  size: number;
  pixels: Uint8Array;
  /** One byte a texel, 0..255 self-light, same layer order as `pixels`. */
  glow: Uint8Array;
}

/** Decodes an 8-bit, non-interlaced PNG of colour type 6 (RGBA) or 0 (grey) into raw texels. */
async function decodePng(url: string): Promise<{ width: number; height: number; data: Uint8Array }> {
  const bytes = new Uint8Array(await (await fetch(url)).arrayBuffer());
  const view = new DataView(bytes.buffer);
  let width = 0, height = 0, channels = 4;
  const idat: Uint8Array[] = [];
  for (let at = 8; at < bytes.length; ) {
    const length = view.getUint32(at);
    const type = String.fromCharCode(...bytes.subarray(at + 4, at + 8));
    const body = bytes.subarray(at + 8, at + 8 + length);
    if (type === 'IHDR') {
      width = view.getUint32(at + 8);
      height = view.getUint32(at + 12);
      channels = bytes[at + 8 + 9] === 6 ? 4 : 1;
    } else if (type === 'IDAT') idat.push(body);
    at += 12 + length;
  }
  const zipped = new Blob(idat as BlobPart[]);
  const raw = new Uint8Array(await new Response(zipped.stream().pipeThrough(new DecompressionStream('deflate'))).arrayBuffer());
  const stride = width * channels;
  const out = new Uint8Array(stride * height);
  // One loop per filter type (the filter is chosen per row), so the inner loops carry no branches.
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)];
    const src = y * (stride + 1) + 1;
    const dst = y * stride;
    const up = dst - stride;
    const lead = Math.min(channels, stride);
    if (filter === 0 || (y === 0 && filter === 2)) {
      out.set(raw.subarray(src, src + stride), dst);
    } else if (filter === 1) {
      for (let i = 0; i < lead; i++) out[dst + i] = raw[src + i];
      for (let i = channels; i < stride; i++) out[dst + i] = raw[src + i] + out[dst + i - channels];
    } else if (filter === 2) {
      for (let i = 0; i < stride; i++) out[dst + i] = raw[src + i] + out[up + i];
    } else if (filter === 3) {
      for (let i = 0; i < lead; i++) out[dst + i] = raw[src + i] + (y > 0 ? out[up + i] >> 1 : 0);
      for (let i = channels; i < stride; i++) out[dst + i] = raw[src + i] + ((out[dst + i - channels] + (y > 0 ? out[up + i] : 0)) >> 1);
    } else {
      for (let i = 0; i < stride; i++) {
        const left = i >= channels ? out[dst + i - channels] : 0;
        const above = y > 0 ? out[up + i] : 0;
        const corner = y > 0 && i >= channels ? out[up + i - channels] : 0;
        const p = left + above - corner;
        const pa = Math.abs(p - left), pb = Math.abs(p - above), pc = Math.abs(p - corner);
        out[dst + i] = raw[src + i] + (pa <= pb && pa <= pc ? left : pb <= pc ? above : corner);
      }
    }
  }
  return { width, height, data: out };
}

/**
 * Pushes 16 known texels through the same upload-and-`readPixels` path the pack takes. False when the browser tampers
 * with readback (Firefox's fingerprint protection and canvas-blocking extensions answer with zeros or noise), which
 * would turn every pack layer black without any error: the caller then keeps the engine's own textures.
 */
async function readbackIsExact(gl: WebGL2RenderingContext, raw: ImageBitmapOptions): Promise<boolean> {
  const known = new Uint8ClampedArray(64);
  for (let i = 0; i < 16; i++) known.set([(i * 37 + 11) & 255, (i * 91 + 5) & 255, (i * 53 + 200) & 255, 255], i * 4);
  const probe = await createImageBitmap(new ImageData(known, 4, 4), raw);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, gl.RGBA, gl.UNSIGNED_BYTE, probe);
  probe.close();
  const back = new Uint8Array(64);
  gl.readPixels(0, 0, 4, 4, gl.RGBA, gl.UNSIGNED_BYTE, back);
  return back.every((v, i) => v === known[i]);
}

/**
 * Decodes the WebP slices into one RGBA stack of `rows` rows, keeping un-premultiplied alpha: canvas 2D would premultiply
 * and wreck the colour under transparent texels (the shaders read it), so the bitmaps go through a WebGL texture and
 * `readPixels` with premultiplication off. Needs WebGL2, which the game needs anyway.
 */
async function decodeWebpStack(urls: string[], rows: number): Promise<Uint8Array> {
  const out = new Uint8Array(rows * ART_SIZE * 4);
  const gl = new OffscreenCanvas(1, 1).getContext('webgl2');
  if (!gl) throw new Error('no WebGL2 to decode the art pack');
  const raw = { premultiplyAlpha: 'none', colorSpaceConversion: 'none', imageOrientation: 'none' } as const;
  gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
  gl.pixelStorei(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL, gl.NONE);
  gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);
  const texture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, texture);
  const frame = gl.createFramebuffer();
  gl.bindFramebuffer(gl.FRAMEBUFFER, frame);
  gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
  if (!(await readbackIsExact(gl, raw))) throw new Error('the browser alters canvas readback');
  let row = 0;
  const downloads = urls.map((url) => fetch(url).then((response) => response.blob())); // all slices download at once
  const maxRows = gl.getParameter(gl.MAX_TEXTURE_SIZE) as number;
  for (const download of downloads) {
    const blob = await download;
    const slice = await createImageBitmap(blob, raw);
    // A slice the GPU takes whole goes up in one piece. Otherwise chunks are cut from the blob, never from `slice`:
    // Firefox ignores the vertical offset when cropping an ImageBitmap, so every chunk came out as the slice's first rows.
    const whole = slice.height <= maxRows;
    for (let top = 0; top < slice.height; top += whole ? slice.height : CHUNK_ROWS) {
      const height = whole ? slice.height : Math.min(CHUNK_ROWS, slice.height - top);
      const chunk = whole ? slice : await createImageBitmap(blob, 0, top, ART_SIZE, height, raw);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, gl.RGBA, gl.UNSIGNED_BYTE, chunk);
      if (!whole) chunk.close();
      gl.readPixels(0, 0, ART_SIZE, height, gl.RGBA, gl.UNSIGNED_BYTE, out.subarray(row * ART_SIZE * 4, (row + height) * ART_SIZE * 4));
      row += height;
    }
    slice.close();
  }
  gl.getExtension('WEBGL_lose_context')?.loseContext();
  if (row !== rows) throw new Error(`art pack has ${row} rows, expected ${rows}`);
  return out;
}

/** Loads the pack and merges it over the engine's layers; null when the pack can't be read (the engine's own are used). */
export async function composeTextures(engine: Uint8Array, engineSize: number, layers: number): Promise<ArtTextures | null> {
  try {
    const [art, grey] = await Promise.all([
      decodeWebpStack(sliceUrls, table.length * ART_SIZE).then((data) => ({ data })),
      decodePng(glowUrl),
    ]);
    const layerTexels = ART_SIZE * ART_SIZE;
    const pixels = new Uint8Array(layers * layerTexels * 4);
    // Whole texels move as one 32-bit word each.
    const from32 = new Uint32Array(engine.buffer, engine.byteOffset, engine.length >> 2);
    const to32 = new Uint32Array(pixels.buffer);
    const scale = ART_SIZE / engineSize;
    for (let layer = 0; layer < layers; layer++) {
      for (let y = 0; y < ART_SIZE; y++) {
        const src = (layer * engineSize + Math.floor(y / scale)) * engineSize;
        const dst = (layer * ART_SIZE + y) * ART_SIZE;
        for (let x = 0; x < ART_SIZE; x++) to32[dst + x] = from32[src + Math.floor(x / scale)];
      }
    }
    const glow = new Uint8Array(layers * layerTexels);
    table.forEach((entry, i) => {
      pixels.set(art.data.subarray(i * layerTexels * 4, (i + 1) * layerTexels * 4), entry.layer * layerTexels * 4);
      glow.set(grey.data.subarray(i * layerTexels, (i + 1) * layerTexels), entry.layer * layerTexels);
    });
    return { size: ART_SIZE, pixels, glow };
  } catch (error) {
    console.warn('art pack not used, the engine textures are shown instead:', error);
    return null;
  }
}
