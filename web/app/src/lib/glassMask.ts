/**
 * GUI-065c: the glass as a wall mask — the vessel's `inner` SVG path
 * rasterized onto the fluid grid, so the simulation lives inside the
 * actual curved glassware (a conical flask's fluid domain is a cone,
 * not the bounding box it swam in before).
 *
 * Pure and canvas-free on purpose: the inner paths use only M/L/Q/Z,
 * so a tiny parser + quadratic flattener + even-odd point-in-polygon
 * covers them exactly, runs identically in workers and jsdom tests,
 * and rasterizes a 50×70 grid in well under a millisecond. If a glass
 * kind ever grows an unsupported command, this THROWS rather than
 * guessing a shape.
 */

export type Point = [number, number];

/** Parse an M/L/Q/Z path into a closed polygon, quadratics flattened
 * into `segments` chords each. */
export function pathToPolygon(d: string, segments = 8): Point[] {
  const tokens = d.trim().split(/[\s,]+/);
  const pts: Point[] = [];
  let i = 0;
  const num = () => {
    const v = Number(tokens[i++]);
    if (!Number.isFinite(v)) throw new Error(`bad number in path at token ${i - 1}`);
    return v;
  };
  while (i < tokens.length) {
    const cmd = tokens[i++]!;
    switch (cmd) {
      case "M":
      case "L":
        pts.push([num(), num()]);
        break;
      case "Q": {
        const [cx, cy] = [num(), num()];
        const [ex, ey] = [num(), num()];
        const [sx, sy] = pts[pts.length - 1] ?? [cx, cy];
        for (let k = 1; k <= segments; k++) {
          const t = k / segments;
          const mt = 1 - t;
          pts.push([
            mt * mt * sx + 2 * mt * t * cx + t * t * ex,
            mt * mt * sy + 2 * mt * t * cy + t * t * ey,
          ]);
        }
        break;
      }
      case "Z":
        break; // closure is implicit in the polygon test
      default:
        throw new Error(`glass path uses unsupported command '${cmd}'`);
    }
  }
  if (pts.length < 3) throw new Error("glass path yields no polygon");
  return pts;
}

/** Even-odd point-in-polygon. */
export function inPolygon(poly: Point[], x: number, y: number): boolean {
  let inside = false;
  for (let a = 0, b = poly.length - 1; a < poly.length; b = a++) {
    const [ax, ay] = poly[a]!;
    const [bx, by] = poly[b]!;
    if (ay > y !== by > y && x < ((bx - ax) * (y - ay)) / (by - ay) + ax) {
      inside = !inside;
    }
  }
  return inside;
}

/**
 * Rasterize the glass interior onto a w×h grid spanning the given
 * viewBox region: solid[i] = 1 OUTSIDE the glass. Cell centres decide.
 * Cached per (path, w, h) by the caller.
 */
export function maskFromPath(
  d: string,
  w: number,
  h: number,
  viewW = 100,
  viewH = 140,
): Uint8Array {
  const poly = pathToPolygon(d);
  const mask = new Uint8Array(w * h);
  for (let y = 0; y < h; y++) {
    const py = ((y + 0.5) / h) * viewH;
    for (let x = 0; x < w; x++) {
      const px = ((x + 0.5) / w) * viewW;
      if (!inPolygon(poly, px, py)) mask[y * w + x] = 1;
    }
  }
  return mask;
}

const maskCache = new Map<string, Uint8Array>();

/** Cached mask for a glass kind's inner path. */
export function maskFor(d: string, w: number, h: number): Uint8Array {
  const key = `${w}x${h}:${d}`;
  let m = maskCache.get(key);
  if (!m) {
    m = maskFromPath(d, w, h);
    maskCache.set(key, m);
  }
  return m;
}
