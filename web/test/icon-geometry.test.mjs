// Tests for the geometry of round and leaned parts (web/src/ui/icon-geometry.ts) and the cylinder the box pipeline draws
// round bodies with (web/src/render/round-mesh.ts), run with `node --test web/test`: an icon's outline stays within a
// pixel of the exact one, the end that faces the viewer is the right one, a leaned box leans as the shader leans it,
// sides shade as a box's do, and the mesh is whole and wound outwards.

import assert from 'node:assert/strict';
import test from 'node:test';
import { boxAxes, farEndShows, hull, iso, outline, rimPoint, roundBody, shadeOf } from '../src/ui/icon-geometry.ts';
import { ROUND_SIDES, roundMesh } from '../src/render/round-mesh.ts';

/** A part record (`item_model`): centre, size, yaw, three layers, form, taper. */
const record = (size, form, taper = 1, yaw = 0) => [0, 0, 0, ...size, yaw, 0, 0, 0, form, taper];

/** The distance from `q` to the polygon `poly`'s edges. */
function toEdges(poly, q) {
  let best = Infinity;
  poly.forEach((a, i) => {
    const b = poly[(i + 1) % poly.length];
    const [dx, dy] = [b[0] - a[0], b[1] - a[1]];
    const t = Math.max(0, Math.min(1, ((q[0] - a[0]) * dx + (q[1] - a[1]) * dy) / (dx * dx + dy * dy)));
    best = Math.min(best, Math.hypot(q[0] - a[0] - t * dx, q[1] - a[1] - t * dy));
  });
  return best;
}

test('a round body outline is within a pixel of the exact one on a 64 px icon it fills', () => {
  for (const [size, form, taper] of [[[1, 0.6, 1], 1, 1], [[0.9, 0.3, 0.3], 2, 0.5], [[0.4, 0.4, 1], 3, 1]]) {
    const b = roundBody(record(size, form, taper));
    const drawn = outline(b);
    // The exact outline: the hull of both rims sampled densely.
    const pts = [];
    for (const end of [-1, 1]) for (let i = 0; i < 4096; i++) pts.push(iso(rimPoint(b, end, (2 * Math.PI * i) / 4096)));
    const exact = hull(pts);
    const xs = exact.map((q) => q[0]), ys = exact.map((q) => q[1]);
    const px = 64 / Math.max(Math.max(...xs) - Math.min(...xs), Math.max(...ys) - Math.min(...ys));
    // Between its corners each edge strays furthest at its middle.
    let worst = 0;
    drawn.forEach((a, i) => {
      const c = drawn[(i + 1) % drawn.length];
      worst = Math.max(worst, toEdges(exact, a), toEdges(exact, [(a[0] + c[0]) / 2, (a[1] + c[1]) / 2]));
    });
    assert.ok(worst * px <= 1, `form ${form}: ${(worst * px).toFixed(2)} px off`);
  }
});

test('records decode to the right axis and only round forms are round', () => {
  assert.equal(roundBody(record([1, 1, 1], 0)), null, 'a box');
  assert.equal(roundBody(record([1, 1, 1], 4, 0.3)), null, 'a leaned box');
  assert.deepEqual(roundBody(record([0.2, 0.5, 0.2], 1)).axis, [0, 1, 0]);
  assert.deepEqual(roundBody(record([0.5, 0.2, 0.2], 2)).axis, [1, 0, 0]);
  assert.equal(roundBody(record([0.5, 0.2, 0.3], 2)).length, 0.5, 'lying along x, its length is its x');
  assert.ok(farEndShows(roundBody(record([0.2, 0.5, 0.2], 1))), 'the top of an upright body faces a viewer above');
});

test('a leaned box leans its up as the shader does and keeps a right-handed frame', () => {
  const [yaw, pitch] = [0.7, 0.4];
  const [x, y, z] = boxAxes(yaw, pitch);
  const want = [-Math.sin(yaw) * Math.sin(pitch), Math.cos(pitch), Math.cos(yaw) * Math.sin(pitch)];
  y.forEach((v, i) => assert.ok(Math.abs(v - want[i]) < 1e-9));
  const cross = [x[1] * y[2] - x[2] * y[1], x[2] * y[0] - x[0] * y[2], x[0] * y[1] - x[1] * y[0]];
  cross.forEach((v, i) => assert.ok(Math.abs(v - z[i]) < 1e-9, 'x cross y is z'));
});

test('faces shade as the icon shades a box: the top lit, the sides by which way they face', () => {
  assert.equal(shadeOf([0, 1, 0]), 0);
  assert.ok(Math.abs(shadeOf([1, 0, 0]) - 0.4) < 1e-9);
  assert.ok(Math.abs(shadeOf([0, 0, 1]) - 0.22) < 1e-9);
});

test('the cylinder mesh has every triangle, indexes only its vertices and winds outwards', () => {
  const { corners, indices } = roundMesh();
  const n = corners.length / 4;
  assert.equal(indices.length, 3 * 4 * ROUND_SIDES, 'two triangles a side, one a cap each');
  assert.ok(indices.every((i) => i < n));
  const at = (i) => [corners[4 * i], corners[4 * i + 1], corners[4 * i + 2]];
  for (let t = 0; t < indices.length; t += 3) {
    const [a, b, c] = [at(indices[t]), at(indices[t + 1]), at(indices[t + 2])];
    const u = b.map((v, i) => v - a[i]), v = c.map((w, i) => w - a[i]);
    const normal = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
    const mid = [0, 1, 2].map((i) => (a[i] + b[i] + c[i]) / 3);
    // Outwards: away from the axis on the side, up or down on the caps.
    const out = Math.abs(mid[1]) > 0.49 && Math.abs(normal[1]) > 1e-9 ? [0, mid[1], 0] : [mid[0], 0, mid[2]];
    assert.ok(normal[0] * out[0] + normal[1] * out[1] + normal[2] * out[2] > 0, `triangle ${t / 3} faces in`);
  }
});
