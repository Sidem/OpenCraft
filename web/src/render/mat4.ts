// Minimal column-major matrix helpers. Rendering is camera-relative, so the view matrix is a pure
// rotation and all world offsets are computed in float64 on the CPU before reaching the GPU.

export function perspective(out: Float32Array, fovY: number, aspect: number, near: number, far: number): void {
  const f = 1 / Math.tan(fovY / 2);
  const nf = 1 / (near - far);
  out.fill(0);
  out[0] = f / aspect;
  out[5] = f;
  out[10] = (far + near) * nf;
  out[11] = -1;
  out[14] = 2 * far * near * nf;
}

/** Rotation-only view matrix. yaw = 0 looks down -Z, positive yaw turns right (matches the engine). */
export function viewRotation(out: Float32Array, yaw: number, pitch: number): void {
  const cp = Math.cos(pitch), sp = Math.sin(pitch);
  const cy = Math.cos(yaw), sy = Math.sin(yaw);
  // forward f = (sy·cp, sp, -cy·cp), right s = (cy, 0, sy), up u = s × f
  const fx = sy * cp, fy = sp, fz = -cy * cp;
  const ux = -sy * sp, uy = cp, uz = cy * sp;
  out.fill(0);
  out[0] = cy; out[4] = 0; out[8] = sy;
  out[1] = ux; out[5] = uy; out[9] = uz;
  out[2] = -fx; out[6] = -fy; out[10] = -fz;
  out[15] = 1;
}

export function multiply(out: Float32Array, a: Float32Array, b: Float32Array): void {
  for (let c = 0; c < 4; c++) {
    const b0 = b[c * 4], b1 = b[c * 4 + 1], b2 = b[c * 4 + 2], b3 = b[c * 4 + 3];
    out[c * 4] = a[0] * b0 + a[4] * b1 + a[8] * b2 + a[12] * b3;
    out[c * 4 + 1] = a[1] * b0 + a[5] * b1 + a[9] * b2 + a[13] * b3;
    out[c * 4 + 2] = a[2] * b0 + a[6] * b1 + a[10] * b2 + a[14] * b3;
    out[c * 4 + 3] = a[3] * b0 + a[7] * b1 + a[11] * b2 + a[15] * b3;
  }
}

/** Extracts the 6 frustum planes (Gribb–Hartmann) as (a, b, c, d) quadruples. */
export function frustumPlanes(out: Float32Array, m: Float32Array): void {
  const rows = [0, 1, 2].flatMap((i) => [
    [m[3] + m[i], m[7] + m[4 + i], m[11] + m[8 + i], m[15] + m[12 + i]],
    [m[3] - m[i], m[7] - m[4 + i], m[11] - m[8 + i], m[15] - m[12 + i]],
  ]);
  rows.forEach((p, i) => {
    const len = Math.hypot(p[0], p[1], p[2]) || 1;
    for (let k = 0; k < 4; k++) out[i * 4 + k] = p[k] / len;
  });
}

export function boxInFrustum(
  planes: Float32Array,
  minX: number, minY: number, minZ: number,
  maxX: number, maxY: number, maxZ: number,
): boolean {
  for (let i = 0; i < 24; i += 4) {
    const a = planes[i], b = planes[i + 1], c = planes[i + 2], d = planes[i + 3];
    const x = a > 0 ? maxX : minX;
    const y = b > 0 ? maxY : minY;
    const z = c > 0 ? maxZ : minZ;
    if (a * x + b * y + c * z + d < 0) return false;
  }
  return true;
}
