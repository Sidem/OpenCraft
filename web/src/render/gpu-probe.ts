// Which graphics the browser really gives the game, and whether that is a graphics card at all. `classifyGpu` is the rule
// (pure; web/test/perf.test.mjs); `probeGpu` gathers its two signals once at start. Software is only claimed when a signal
// says so: browsers may hide the GPU's name, and a wrong warning is worse than none. To recognise another software
// renderer: add it to `SOFTWARE`.

export type GpuKind = 'software' | 'hardware' | 'unknown';

export interface GpuFacts {
  name: string;
  kind: GpuKind;
}

const SOFTWARE = /swiftshader|swangle|llvmpipe|softpipe|basic render|software adapter|warp|gdi generic/i;
/** What browsers answer when they will not name the GPU. */
const MASKED = /^(|mozilla|webkit webgl|webkit|generic renderer|unknown|not available)$/i;

/** `caveatPassed`: a context that refuses software rendering could be made; null when that could not be tried. */
export function classifyGpu(name: string, caveatPassed: boolean | null): GpuKind {
  if (SOFTWARE.test(name)) return 'software';
  if (MASKED.test(name.trim())) return caveatPassed === null ? 'unknown' : caveatPassed ? 'hardware' : 'software';
  // A real GPU named, yet the browser reports a caveat: the two disagree, so say nothing.
  return caveatPassed === false ? 'unknown' : 'hardware';
}

/** `gl` is the game's own context, so the name is the GPU it actually draws on. */
export function probeGpu(gl: WebGL2RenderingContext): GpuFacts {
  const info = gl.getExtension('WEBGL_debug_renderer_info');
  const name = String(gl.getParameter(info ? info.UNMASKED_RENDERER_WEBGL : gl.RENDERER) ?? '');
  return { name, kind: classifyGpu(name, caveatPassed()) };
}

/** Asks for a throwaway context that must not be software, and gives it straight back. */
function caveatPassed(): boolean | null {
  try {
    const gl = document.createElement('canvas').getContext('webgl2', {
      failIfMajorPerformanceCaveat: true,
      powerPreference: 'high-performance',
    });
    gl?.getExtension('WEBGL_lose_context')?.loseContext();
    return gl !== null;
  } catch {
    return null;
  }
}
