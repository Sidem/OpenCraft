// The art pack: hand-painted 128 × 128 layers made by `tools/art` (docs/ART_STYLE.md) that replace layers of the
// engine's own generated textures. `composeTextures` builds the array the renderer uploads: engine layers the pack
// doesn't cover are repeated 2 × 2 to 128 so every layer has one size, pack layers are copied in, and a second array
// holds each texel's self-light (glow). To add layers, extend `tools/art/registry.py` and rebuild the pack.

import packUrl from '../art/pack.png?url';
import glowUrl from '../art/pack_glow.png?url';
import table from '../art/pack.json';

export const ART_SIZE = 128;

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

/** Loads the pack and merges it over the engine's layers; null when the pack can't be read (the engine's own are used). */
export async function composeTextures(engine: Uint8Array, engineSize: number, layers: number): Promise<ArtTextures | null> {
  try {
    const [art, grey] = await Promise.all([decodePng(packUrl), decodePng(glowUrl)]);
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
  } catch {
    return null;
  }
}
