// The unit cylinder the box pipeline draws round bodies with (`boxes.ts`, engine `factory::push_round`): diameter
// and height 1, centred on 0, its axis y. Each vertex is (x, y, z, face): face 2 is the top cap, 3 the bottom one,
// and 6 plus the share of the way round is the side (`boxVert` in `shaders.ts` reads it). Side and caps keep their own
// vertices so the rim stays sharp, the side's seam vertex repeats so its texture wraps, and `ROUND_SIDES` is a
// multiple of 4 so a vertex stands on every quarter turn, where the shader mirrors the texture. Triangles wind
// counter-clockwise seen from outside, as the cube's do. More `ROUND_SIDES` is rounder and costs vertices per body.

/** Sides of the cylinder: its outline strays at most 1 - cos(pi / ROUND_SIDES) of its radius from a circle. */
export const ROUND_SIDES = 16;

/** The cylinder's vertices (4 floats each) and the indices of its triangles. */
export function roundMesh(sides = ROUND_SIDES): { corners: Float32Array; indices: Uint16Array } {
  const corners: number[] = [];
  const indices: number[] = [];
  // Going round from +x towards -z, so the side's texture runs left to right seen from outside.
  const ring = (i: number): [number, number] => {
    const a = (2 * Math.PI * i) / sides;
    return [0.5 * Math.cos(a), -0.5 * Math.sin(a)];
  };
  for (let i = 0; i <= sides; i++) {
    const [x, z] = ring(i);
    corners.push(x, -0.5, z, 6 + i / sides, x, 0.5, z, 6 + i / sides);
  }
  for (let i = 0; i < sides; i++) {
    const b = 2 * i, t = b + 1, b2 = b + 2, t2 = b + 3;
    indices.push(b, b2, t2, b, t2, t);
  }
  for (const [y, face] of [[0.5, 2], [-0.5, 3]]) {
    const centre = corners.length / 4;
    corners.push(0, y, 0, face);
    for (let i = 0; i < sides; i++) {
      const [x, z] = ring(i);
      corners.push(x, y, z, face);
    }
    for (let i = 0; i < sides; i++) {
      const a = centre + 1 + i, b = centre + 1 + ((i + 1) % sides);
      if (face === 2) indices.push(centre, a, b);
      else indices.push(centre, b, a);
    }
  }
  return { corners: new Float32Array(corners), indices: new Uint16Array(indices) };
}
