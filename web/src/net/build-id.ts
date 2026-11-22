// The co-op build id of a module URL (used by protocol.ts as `BUILD_ID`). Kept apart so node tests can import it.

/** The build id of the module at `url`: its file name only. In a production build the name carries the
 * bundle's content hash, which changes with any script or the wasm. The directory is left out on purpose:
 * the same build is served from different places (GitHub Pages' /OpenCraft/, itch.io's /html/<id>/), and
 * those peers must still match. */
export function buildIdOf(url: string): string {
  const path = new URL(url).pathname;
  return path.slice(path.lastIndexOf('/') + 1);
}
