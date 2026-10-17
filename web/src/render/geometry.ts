// Static geometry the renderer uploads once: the unit cube for the mining-crack overlay and the 12 edges of a box for outlines.
// Presentation only.

/**
 * Unit cube centred on the origin: 36 vertices of (pos.xyz, u, v, face kind, shade).
 * Face order and u/v axes match the engine mesher so textures line up with world blocks.
 */
export function cubeVertices(): Float32Array {
  const faces: [number[], number[], number[], number, number, number[]][] = [
    // normal, u axis, v axis, face kind (0 top, 1 side, 2 bottom), shade, texture right axis
    [[1, 0, 0], [0, 1, 0], [0, 0, 1], 1, 0.72, [0, 0, -1]],
    [[-1, 0, 0], [0, 0, 1], [0, 1, 0], 1, 0.72, [0, 0, 1]],
    [[0, 1, 0], [0, 0, 1], [1, 0, 0], 0, 1.0, [1, 0, 0]],
    [[0, -1, 0], [1, 0, 0], [0, 0, 1], 2, 0.52, [1, 0, 0]],
    [[0, 0, 1], [1, 0, 0], [0, 1, 0], 1, 0.86, [1, 0, 0]],
    [[0, 0, -1], [0, 1, 0], [1, 0, 0], 1, 0.86, [-1, 0, 0]],
  ];
  const out: number[] = [];
  for (const [n, u, v, kind, shade, right] of faces) {
    const corner = (cu: number, cv: number) => {
      const p = [0, 1, 2].map((i) => n[i] * 0.5 + (cu - 0.5) * u[i] + (cv - 0.5) * v[i]);
      const s = p[0] * right[0] + p[1] * right[1] + p[2] * right[2] + 0.5;
      const t = kind === 1 ? 0.5 - p[1] : p[2] + 0.5;
      return [...p, s, t, kind, shade];
    };
    const c = [corner(0, 0), corner(1, 0), corner(1, 1), corner(0, 1)];
    for (const i of [0, 1, 2, 0, 2, 3]) out.push(...c[i]);
  }
  return new Float32Array(out);
}

export function boxEdges(): Float32Array {
  const e: number[] = [];
  for (let a = 0; a < 3; a++) {
    for (const i of [0, 1]) {
      for (const j of [0, 1]) {
        const p0 = [0, 0, 0], p1 = [0, 0, 0];
        p1[a] = 1;
        p0[(a + 1) % 3] = p1[(a + 1) % 3] = i;
        p0[(a + 2) % 3] = p1[(a + 2) % 3] = j;
        e.push(...p0, ...p1);
      }
    }
  }
  return new Float32Array(e);
}
