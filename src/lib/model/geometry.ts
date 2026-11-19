import type { Opening, Phase, Project, Pt, Wall } from './types';
import { visibleIn } from './types';

export const dist = (a: Pt, b: Pt) => Math.hypot(a.x - b.x, a.y - b.y);
export const add = (a: Pt, b: Pt): Pt => ({ x: a.x + b.x, y: a.y + b.y });
export const sub = (a: Pt, b: Pt): Pt => ({ x: a.x - b.x, y: a.y - b.y });
export const mul = (a: Pt, k: number): Pt => ({ x: a.x * k, y: a.y * k });
export const lerp = (a: Pt, b: Pt, t: number): Pt => ({ x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t });
export const round = (v: number, step = 0.001) => Math.round(v / step) * step;

export const wallLength = (w: Pick<Wall, 'a' | 'b'>) => dist(w.a, w.b);

export function wallDir(w: Pick<Wall, 'a' | 'b'>): Pt {
  const l = wallLength(w) || 1;
  return { x: (w.b.x - w.a.x) / l, y: (w.b.y - w.a.y) / l };
}

export const wallNormal = (w: Pick<Wall, 'a' | 'b'>): Pt => {
  const d = wallDir(w);
  return { x: -d.y, y: d.x };
};

/** Projette p sur le segment : t en mètres depuis a, distance au segment. */
export function projectOnWall(w: Pick<Wall, 'a' | 'b'>, p: Pt) {
  const d = wallDir(w);
  const len = wallLength(w);
  const t = Math.max(0, Math.min(len, (p.x - w.a.x) * d.x + (p.y - w.a.y) * d.y));
  const q = { x: w.a.x + d.x * t, y: w.a.y + d.y * t };
  return { t, point: q, distance: dist(p, q) };
}

export function polygonArea(points: Pt[]): number {
  return Math.abs(signedArea(points));
}

export function signedArea(points: Pt[]): number {
  let s = 0;
  for (let i = 0; i < points.length; i++) {
    const a = points[i];
    const b = points[(i + 1) % points.length];
    s += a.x * b.y - b.x * a.y;
  }
  return s / 2;
}

export function polygonPerimeter(points: Pt[]): number {
  return points.reduce((s, p, i) => s + dist(p, points[(i + 1) % points.length]), 0);
}

export function centroid(points: Pt[]): Pt {
  const area = signedArea(points);
  if (Math.abs(area) < 1e-6) {
    const k = 1 / (points.length || 1);
    return points.reduce((c, p) => ({ x: c.x + p.x * k, y: c.y + p.y * k }), { x: 0, y: 0 });
  }
  let x = 0;
  let y = 0;
  for (let i = 0; i < points.length; i++) {
    const a = points[i];
    const b = points[(i + 1) % points.length];
    const f = a.x * b.y - b.x * a.y;
    x += (a.x + b.x) * f;
    y += (a.y + b.y) * f;
  }
  return { x: x / (6 * area), y: y / (6 * area) };
}

export function pointInPolygon(p: Pt, points: Pt[]): boolean {
  let inside = false;
  for (let i = 0, j = points.length - 1; i < points.length; j = i++) {
    const a = points[i];
    const b = points[j];
    if (a.y > p.y !== b.y > p.y && p.x < ((b.x - a.x) * (p.y - a.y)) / (b.y - a.y) + a.x) inside = !inside;
  }
  return inside;
}

export const rectPoints = (x: number, y: number, w: number, h: number): Pt[] => [
  { x, y },
  { x: x + w, y },
  { x: x + w, y: y + h },
  { x, y: y + h },
];

export function bounds(points: Pt[]) {
  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  const minX = Math.min(...xs);
  const minY = Math.min(...ys);
  const maxX = Math.max(...xs);
  const maxY = Math.max(...ys);
  return { minX, minY, maxX, maxY, w: maxX - minX, h: maxY - minY };
}

export function rotatePoint(p: Pt, center: Pt, deg: number): Pt {
  const r = (deg * Math.PI) / 180;
  const c = Math.cos(r);
  const s = Math.sin(r);
  const dx = p.x - center.x;
  const dy = p.y - center.y;
  return { x: center.x + dx * c - dy * s, y: center.y + dx * s + dy * c };
}

/** Coins d'un rectangle centré et tourné (meuble, escalier). */
export function boxCorners(b: { x: number; y: number; w: number; d: number; rotation: number }): Pt[] {
  const c = { x: b.x, y: b.y };
  return rectPoints(b.x - b.w / 2, b.y - b.d / 2, b.w, b.d).map((p) => rotatePoint(p, c, b.rotation));
}

export const wallOpenings = (project: Project, wallId: string, phase: Phase): Opening[] =>
  project.openings
    .filter((o) => o.wall === wallId && visibleIn(o.status, phase))
    .sort((a, b) => a.offset - b.offset);

/** Portions pleines du mur entre les ouvertures, en mètres depuis a. */
export function solidSpans(length: number, openings: Pick<Opening, 'offset' | 'width'>[]): [number, number][] {
  const spans: [number, number][] = [];
  let cursor = 0;
  for (const o of [...openings].sort((a, b) => a.offset - b.offset)) {
    const from = Math.max(cursor, o.offset - o.width / 2);
    if (from > cursor + 1e-4) spans.push([cursor, from]);
    cursor = Math.max(cursor, o.offset + o.width / 2);
  }
  if (cursor < length - 1e-4) spans.push([cursor, length]);
  return spans;
}

/** Prolongement d'une extrémité de mur pour fermer proprement l'angle. */
export function endExtension(p: Pt, wall: Wall, walls: Wall[]): number {
  let ext = 0;
  for (const other of walls) {
    if (other.id === wall.id || other.level !== wall.level) continue;
    if (projectOnWall(other, p).distance < 0.02) ext = Math.max(ext, other.thickness / 2);
  }
  return ext;
}

export function wallPolygon(w: Wall, from: number, to: number, startExt = 0, endExt = 0): Pt[] {
  const d = wallDir(w);
  const n = wallNormal(w);
  const h = w.thickness / 2;
  const s = add(w.a, mul(d, from - startExt));
  const e = add(w.a, mul(d, to + endExt));
  return [add(s, mul(n, h)), add(e, mul(n, h)), add(e, mul(n, -h)), add(s, mul(n, -h))];
}

interface GraphEdge {
  from: string;
  to: string;
  thickness: number;
}

const nodeKey = (p: Pt) => `${Math.round(p.x * 100)}:${Math.round(p.y * 100)}`;

function segmentIntersection(a: Pt, b: Pt, c: Pt, d: Pt): Pt | null {
  const r = sub(b, a);
  const s = sub(d, c);
  const denom = r.x * s.y - r.y * s.x;
  if (Math.abs(denom) < 1e-9) return null;
  const t = ((c.x - a.x) * s.y - (c.y - a.y) * s.x) / denom;
  const u = ((c.x - a.x) * r.y - (c.y - a.y) * r.x) / denom;
  return t >= -0.001 && t <= 1.001 && u >= -0.001 && u <= 1.001 ? add(a, mul(r, t)) : null;
}

function offsetIntersection(a: Pt, b: Pt, c: Pt, d: Pt): Pt | null {
  const r = sub(b, a);
  const s = sub(d, c);
  const denom = r.x * s.y - r.y * s.x;
  if (Math.abs(denom) < 1e-9) return null;
  const t = ((c.x - a.x) * s.y - (c.y - a.y) * s.x) / denom;
  return add(a, mul(r, t));
}

/** Trouve la pièce fermée par les murs autour de p, nez de cloison déduits. */
export function detectRoom(walls: Wall[], p: Pt): Pt[] | null {
  const points = new Map<string, Pt>();
  const edges: GraphEdge[] = [];
  for (const w of walls) {
    const cuts: Pt[] = [w.a, w.b];
    for (const o of walls) {
      if (o.id === w.id) continue;
      const hit = segmentIntersection(w.a, w.b, o.a, o.b);
      if (hit) cuts.push(hit);
      for (const end of [o.a, o.b]) {
        const q = projectOnWall(w, end);
        if (q.distance < 0.03) cuts.push(q.point);
      }
    }
    const d = wallDir(w);
    const sorted = cuts
      .map((c) => ({ c, t: (c.x - w.a.x) * d.x + (c.y - w.a.y) * d.y }))
      .sort((m, n) => m.t - n.t);
    for (let i = 1; i < sorted.length; i++) {
      const from = nodeKey(sorted[i - 1].c);
      const to = nodeKey(sorted[i].c);
      if (from === to) continue;
      points.set(from, sorted[i - 1].c);
      points.set(to, sorted[i].c);
      edges.push({ from, to, thickness: w.thickness });
    }
  }
  const out = new Map<string, { to: string; angle: number; thickness: number }[]>();
  const link = (from: string, to: string, thickness: number) => {
    const a = points.get(from)!;
    const b = points.get(to)!;
    const list = out.get(from) ?? [];
    if (!list.some((e) => e.to === to)) list.push({ to, thickness, angle: Math.atan2(b.y - a.y, b.x - a.x) });
    out.set(from, list);
  };
  for (const e of edges) {
    link(e.from, e.to, e.thickness);
    link(e.to, e.from, e.thickness);
  }
  out.forEach((list) => list.sort((a, b) => a.angle - b.angle));

  const seen = new Set<string>();
  let best: { nodes: string[]; thick: number[]; area: number } | null = null;
  for (const [start, list] of out) {
    for (const first of list) {
      if (seen.has(`${start}>${first.to}`)) continue;
      const nodes: string[] = [];
      const thick: number[] = [];
      let from = start;
      let edge = first;
      for (let guard = 0; guard < 2000; guard++) {
        seen.add(`${from}>${edge.to}`);
        nodes.push(from);
        thick.push(edge.thickness);
        const around = out.get(edge.to)!;
        const back = around.findIndex((e) => e.to === from);
        from = edge.to;
        edge = around[(back - 1 + around.length) % around.length];
        if (from === start && edge.to === first.to) break;
      }
      const poly = nodes.map((n) => points.get(n)!);
      const area = signedArea(poly);
      if (area > 0.05 && pointInPolygon(p, poly) && (!best || area < best.area)) best = { nodes, thick, area };
    }
  }
  if (!best) return null;

  // Retire les sommets alignés puis décale chaque côté de la demi-épaisseur vers l'intérieur.
  const raw = best.nodes.map((n, i) => ({ p: points.get(n)!, t: best!.thick[i] }));
  const clean = raw.filter((v, i) => {
    const prev = raw[(i - 1 + raw.length) % raw.length];
    const next = raw[(i + 1) % raw.length];
    const u = sub(v.p, prev.p);
    const w = sub(next.p, v.p);
    return Math.abs(u.x * w.y - u.y * w.x) > 1e-4 || prev.t !== v.t;
  });
  const lines = clean.map((v, i) => {
    const n = clean[(i + 1) % clean.length].p;
    const e = wallDir({ a: v.p, b: n });
    const off = mul({ x: -e.y, y: e.x }, v.t / 2);
    return { a: add(v.p, off), b: add(n, off) };
  });
  return lines.map((l, i) => {
    const prev = lines[(i - 1 + lines.length) % lines.length];
    const hit = offsetIntersection(prev.a, prev.b, l.a, l.b) ?? l.a;
    return { x: round(hit.x), y: round(hit.y) };
  });
}
