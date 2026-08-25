/**
 * GUI-065a: the Eulerian transport layer — a small stable-fluids solver
 * (Stam-style semi-Lagrangian advection + Jacobi pressure projection on
 * a STAGGERED MAC grid) that animates how one substance meets another
 * inside a vessel: dye plumes, stirring vortices, miscible diffusion,
 * density-buoyant layer separation.
 *
 * MAC, not collocated, deliberately: a first cut on a collocated grid
 * plateaued at ~33% residual divergence no matter the iteration count —
 * the central-difference div/grad pair decouples odd and even cells
 * (checkerboard null space). On the staggered grid the discrete
 * operators are adjoint and projection genuinely kills divergence,
 * which is what keeps swirls from visibly leaking volume.
 *
 * The honesty contract (ROADMAP-GUI.md, realism bar): this simulation
 * is the transport BETWEEN engine states, never the chemistry. Species
 * concentrations are seeded from and relax toward the engine's scene
 * (GUI-058 layers); the settled frame IS the static render. Nothing
 * here feeds back into the engine.
 *
 * Pure typed-array math, no dependencies, no DOM.
 *
 * Layout: cells are (x, y), y = 0 at the TOP (SVG sense; gravity is
 * +y). `u` lives on vertical faces, sized (w+1)×h, index
 * `y*(w+1)+x`. `v` lives on horizontal faces, sized w×(h+1), index
 * `y*w+x`. Scalars (species fields, pressure, solid mask) live at cell
 * centres, sized w×h, index `y*w+x`.
 */

export interface FluidGrid {
  w: number;
  h: number;
  /** Cell size in world units (px of the vessel viewBox). */
  dx: number;
  /** x-velocity on vertical faces, (w+1)×h. */
  u: Float32Array;
  /** y-velocity on horizontal faces, w×(h+1). */
  v: Float32Array;
  /** 1 = wall/outside-glass, 0 = fluid; cell-centred. */
  solid: Uint8Array;
  /** One concentration field per species, cell-centred. */
  fields: Float32Array[];
  pressure: Float32Array;
  divergence: Float32Array;
  /** Scratch buffers sized for the largest arrays, reused to stay
   * allocation-free per frame. */
  scratchU: Float32Array;
  scratchV: Float32Array;
  scratchC: Float32Array;
  scratchP: Float32Array;
}

export function makeGrid(w: number, h: number, dx: number, species: number): FluidGrid {
  return {
    w,
    h,
    dx,
    u: new Float32Array((w + 1) * h),
    v: new Float32Array(w * (h + 1)),
    solid: new Uint8Array(w * h),
    fields: Array.from({ length: species }, () => new Float32Array(w * h)),
    pressure: new Float32Array(w * h),
    divergence: new Float32Array(w * h),
    scratchU: new Float32Array((w + 1) * h),
    scratchV: new Float32Array(w * (h + 1)),
    scratchC: new Float32Array(w * h),
    scratchP: new Float32Array(w * h),
  };
}

const cix = (g: { w: number }, x: number, y: number) => y * g.w + x;
const uix = (g: { w: number }, x: number, y: number) => y * (g.w + 1) + x;
const vix = (g: { w: number }, x: number, y: number) => y * g.w + x;

const solidAt = (g: FluidGrid, x: number, y: number): boolean =>
  x < 0 || y < 0 || x >= g.w || y >= g.h || g.solid[cix(g, x, y)] === 1;

/** Bilinear sample of a cell-centred field at cell coordinates
 * (x, y in cell units, centre of cell (i,j) at (i+0.5, j+0.5) — pass
 * centre-based coordinates minus 0.5). Clamped, never NaN. */
export function sampleCell(g: FluidGrid, f: Float32Array, x: number, y: number): number {
  const cx = Math.min(Math.max(x, 0), g.w - 1.001);
  const cy = Math.min(Math.max(y, 0), g.h - 1.001);
  const x0 = Math.floor(cx);
  const y0 = Math.floor(cy);
  const tx = cx - x0;
  const ty = cy - y0;
  const a = f[cix(g, x0, y0)]!;
  const b = f[cix(g, x0 + 1, y0)]!;
  const c = f[cix(g, x0, y0 + 1)]!;
  const d = f[cix(g, x0 + 1, y0 + 1)]!;
  return a * (1 - tx) * (1 - ty) + b * tx * (1 - ty) + c * (1 - tx) * ty + d * tx * ty;
}

/** Interpolated velocity (in cells/second /dx units) at an arbitrary
 * point given in CELL-CENTRE coordinates. */
export function velocityAt(g: FluidGrid, x: number, y: number): [number, number] {
  // u faces sit at integer x, centre y: coordinate (i, j+0.5).
  const ux = Math.min(Math.max(x + 0.5, 0), g.w - 0.001);
  const uy = Math.min(Math.max(y, 0), g.h - 1.001);
  const ux0 = Math.floor(ux);
  const uy0 = Math.floor(uy);
  const utx = ux - ux0;
  const uty = uy - uy0;
  const u =
    g.u[uix(g, ux0, uy0)]! * (1 - utx) * (1 - uty) +
    g.u[uix(g, ux0 + 1, uy0)]! * utx * (1 - uty) +
    g.u[uix(g, ux0, uy0 + 1)]! * (1 - utx) * uty +
    g.u[uix(g, ux0 + 1, uy0 + 1)]! * utx * uty;
  // v faces sit at centre x, integer y: coordinate (i+0.5, j).
  const vx = Math.min(Math.max(x, 0), g.w - 1.001);
  const vy = Math.min(Math.max(y + 0.5, 0), g.h - 0.001);
  const vx0 = Math.floor(vx);
  const vy0 = Math.floor(vy);
  const vtx = vx - vx0;
  const vty = vy - vy0;
  const v =
    g.v[vix(g, vx0, vy0)]! * (1 - vtx) * (1 - vty) +
    g.v[vix(g, vx0 + 1, vy0)]! * vtx * (1 - vty) +
    g.v[vix(g, vx0, vy0 + 1)]! * (1 - vtx) * vty +
    g.v[vix(g, vx0 + 1, vy0 + 1)]! * vtx * vty;
  return [u, v];
}

/** Zero every face velocity that touches a wall cell — the no-flow
 * boundary, applied after each mutation of the velocity field. */
export function enforceBoundaries(g: FluidGrid): void {
  for (let y = 0; y < g.h; y++) {
    for (let x = 0; x <= g.w; x++) {
      if (solidAt(g, x - 1, y) || solidAt(g, x, y)) g.u[uix(g, x, y)] = 0;
    }
  }
  for (let y = 0; y <= g.h; y++) {
    for (let x = 0; x < g.w; x++) {
      if (solidAt(g, x, y - 1) || solidAt(g, x, y)) g.v[vix(g, x, y)] = 0;
    }
  }
}

/** Semi-Lagrangian advection of the face velocities through themselves.
 * Unconditionally stable — the property that suits a school Chromebook. */
export function advectVelocity(g: FluidGrid, dt: number): void {
  const s = dt / g.dx;
  for (let y = 0; y < g.h; y++) {
    for (let x = 0; x <= g.w; x++) {
      // The u-face at (x, y+0.5) in centre coordinates is (x-0.5, y).
      const px = x - 0.5;
      const py = y;
      const [vu, vv] = velocityAt(g, px, py);
      const [bu] = velocityAt(g, px - s * vu, py - s * vv);
      g.scratchU[uix(g, x, y)] = bu;
    }
  }
  for (let y = 0; y <= g.h; y++) {
    for (let x = 0; x < g.w; x++) {
      const px = x;
      const py = y - 0.5;
      const [vu, vv] = velocityAt(g, px, py);
      const [, bv] = velocityAt(g, px - s * vu, py - s * vv);
      g.scratchV[vix(g, x, y)] = bv;
    }
  }
  g.u.set(g.scratchU);
  g.v.set(g.scratchV);
  enforceBoundaries(g);
}

/** Semi-Lagrangian advection of one cell-centred field. */
export function advectField(g: FluidGrid, f: Float32Array, out: Float32Array, dt: number): void {
  const s = dt / g.dx;
  for (let y = 0; y < g.h; y++) {
    for (let x = 0; x < g.w; x++) {
      const i = cix(g, x, y);
      if (g.solid[i]) {
        out[i] = 0;
        continue;
      }
      const [vu, vv] = velocityAt(g, x, y);
      out[i] = sampleCell(g, f, x - s * vu, y - s * vv);
    }
  }
}

/**
 * Buoyancy from per-species density: applied on v-faces from the two
 * adjacent cells' mixture. `densities[s]` is relative to ambient
 * (1 = neutral); heavier sinks (+y is down in SVG). This is what makes
 * hexane float and permanganate solution sink, from registry numbers.
 */
export function applyBuoyancy(
  g: FluidGrid,
  densities: number[],
  strength: number,
  dt: number,
): void {
  for (let y = 1; y < g.h; y++) {
    for (let x = 0; x < g.w; x++) {
      if (solidAt(g, x, y - 1) || solidAt(g, x, y)) continue;
      let force = 0;
      for (let s = 0; s < g.fields.length; s++) {
        const c =
          0.5 * (g.fields[s]![cix(g, x, y - 1)]! + g.fields[s]![cix(g, x, y)]!);
        force += c * (densities[s]! - 1);
      }
      g.v[vix(g, x, y)] = g.v[vix(g, x, y)]! + strength * force * dt;
    }
  }
}

/** Per-cell divergence from the face velocities. */
export function computeDivergence(g: FluidGrid): void {
  for (let y = 0; y < g.h; y++) {
    for (let x = 0; x < g.w; x++) {
      const i = cix(g, x, y);
      if (g.solid[i]) {
        g.divergence[i] = 0;
        continue;
      }
      g.divergence[i] =
        (g.u[uix(g, x + 1, y)]! - g.u[uix(g, x, y)]! + g.v[vix(g, x, y + 1)]! - g.v[vix(g, x, y)]!) /
        g.dx;
    }
  }
}

/**
 * Pressure projection on the MAC grid: solve ∇²p = div by red–black
 * Gauss–Seidel (in place, converges ~2× per sweep vs Jacobi), then
 * subtract ∇p from the faces. With the staggered operators being
 * adjoint, this genuinely drives divergence toward zero (pinned by
 * test) instead of stalling on a checkerboard mode — the collocated
 * first cut plateaued at ~33% forever.
 */
export function project(g: FluidGrid, iters: number): void {
  computeDivergence(g);
  g.pressure.fill(0);
  const p = g.pressure;
  for (let k = 0; k < iters; k++) {
    for (let colour = 0; colour < 2; colour++) {
      for (let y = 0; y < g.h; y++) {
        for (let x = (y + colour) % 2; x < g.w; x += 2) {
          const i = cix(g, x, y);
          if (g.solid[i]) {
            p[i] = 0;
            continue;
          }
          let sum = 0;
          let count = 0;
          for (const [nx, ny] of [[x - 1, y], [x + 1, y], [x, y - 1], [x, y + 1]] as const) {
            if (solidAt(g, nx, ny)) continue; // Neumann wall
            sum += p[cix(g, nx, ny)]!;
            count++;
          }
          p[i] = count > 0 ? (sum - g.divergence[i]! * g.dx * g.dx) / count : 0;
        }
      }
    }
  }
  for (let y = 0; y < g.h; y++) {
    for (let x = 1; x < g.w; x++) {
      if (solidAt(g, x - 1, y) || solidAt(g, x, y)) continue;
      g.u[uix(g, x, y)] =
        g.u[uix(g, x, y)]! - (p[cix(g, x, y)]! - p[cix(g, x - 1, y)]!) / g.dx;
    }
  }
  for (let y = 1; y < g.h; y++) {
    for (let x = 0; x < g.w; x++) {
      if (solidAt(g, x, y - 1) || solidAt(g, x, y)) continue;
      g.v[vix(g, x, y)] =
        g.v[vix(g, x, y)]! - (p[cix(g, x, y)]! - p[cix(g, x, y - 1)]!) / g.dx;
    }
  }
  enforceBoundaries(g);
}

/** Total of a species over the fluid domain — the conservation number
 * the tests pin. */
export function fieldTotal(g: FluidGrid, s: number): number {
  let sum = 0;
  const f = g.fields[s]!;
  for (let i = 0; i < f.length; i++) {
    if (!g.solid[i]) sum += f[i]!;
  }
  return sum;
}

/**
 * One simulation step: buoyancy → project → advect velocity → advect
 * every species field, re-normalised so numerical diffusion never
 * creates or destroys substance — the conservation half of the honesty
 * contract.
 */
export function step(g: FluidGrid, densities: number[], dt: number, iters = 30): void {
  applyBuoyancy(g, densities, 9.8, dt);
  enforceBoundaries(g);
  project(g, iters);
  advectVelocity(g, dt);
  for (let s = 0; s < g.fields.length; s++) {
    const before = fieldTotal(g, s);
    advectField(g, g.fields[s]!, g.scratchC, dt);
    g.fields[s]!.set(g.scratchC);
    const after = fieldTotal(g, s);
    if (after > 1e-12 && before > 1e-12) {
      const k = before / after;
      const f = g.fields[s]!;
      for (let i = 0; i < f.length; i++) {
        if (!g.solid[i]) f[i] = f[i]! * k;
      }
    }
  }
}

/**
 * Relax the fields toward a target profile (the engine's settled
 * layers): linear blend by `rate·dt`. The honesty gate made mechanism —
 * however the transport swirled, the picture converges to the solver's
 * answer and only ever to it.
 */
export function relaxToward(
  g: FluidGrid,
  s: number,
  target: Float32Array,
  rate: number,
  dt: number,
): void {
  const k = Math.min(1, rate * dt);
  const f = g.fields[s]!;
  for (let i = 0; i < f.length; i++) {
    if (!g.solid[i]) f[i] = f[i]! + (target[i]! - f[i]!) * k;
  }
}

/**
 * The settled target for a layer band: 1 in rows
 * bandTopRow..bandBottomRow (inclusive), 0 elsewhere — GUI-058's
 * picture on the grid.
 */
export function layerTarget(
  g: FluidGrid,
  s: number,
  bandTopRow: number,
  bandBottomRow: number,
): Float32Array {
  const t = new Float32Array(g.w * g.h);
  for (let y = 0; y < g.h; y++) {
    if (y < bandTopRow || y > bandBottomRow) continue;
    for (let x = 0; x < g.w; x++) {
      const i = cix(g, x, y);
      if (!g.solid[i]) t[i] = 1;
    }
  }
  return t;
}
