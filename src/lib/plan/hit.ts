import { boxCorners, dist, lerp, pointInPolygon, polygonArea, projectOnWall, wallDir, wallLength, wallNormal, add, mul } from '../model/geometry';
import type { Phase, Project, Pt, Ref } from '../model/types';
import { visibleIn } from '../model/types';

export const levelWalls = (p: Project, level: string, phase: Phase) => p.walls.filter((w) => w.level === level && visibleIn(w.status, phase));

export function openingCenter(p: Project, id: string): Pt | null {
  const o = p.openings.find((e) => e.id === id);
  const w = o && p.walls.find((e) => e.id === o.wall);
  return o && w ? add(w.a, mul(wallDir(w), o.offset)) : null;
}

export function dimPoints(d: { a: Pt; b: Pt; offset: number }) {
  const n = wallNormal(d);
  return { a: add(d.a, mul(n, d.offset)), b: add(d.b, mul(n, d.offset)) };
}

export function hitTest(p: Project, level: string, phase: Phase, q: Pt, tol: number): Ref | null {
  for (const n of p.notes) if (n.level === level && dist(n, q) < tol * 1.8) return { kind: 'note', id: n.id };
  for (const d of p.dims) {
    if (d.level === level && projectOnWall(dimPoints(d), q).distance < tol * 0.8) return { kind: 'dim', id: d.id };
  }
  const walls = levelWalls(p, level, phase);
  for (const o of p.openings) {
    if (!visibleIn(o.status, phase)) continue;
    const w = walls.find((e) => e.id === o.wall);
    if (!w) continue;
    const c = add(w.a, mul(wallDir(w), o.offset));
    if (projectOnWall({ a: add(c, mul(wallDir(w), -o.width / 2)), b: add(c, mul(wallDir(w), o.width / 2)) }, q).distance < w.thickness / 2 + tol * 0.5) {
      return { kind: 'opening', id: o.id };
    }
  }
  const items = p.items.filter((i) => i.level === level && visibleIn(i.status, phase) && pointInPolygon(q, boxCorners(i)));
  items.sort((a, b) => a.w * a.d - b.w * b.d);
  if (items[0]) return { kind: 'item', id: items[0].id };
  for (const s of p.stairs) {
    if (s.level === level && s.view !== '3d' && visibleIn(s.status, phase) && pointInPolygon(q, boxCorners(s))) return { kind: 'stair', id: s.id };
  }
  for (const w of walls) if (projectOnWall(w, q).distance < w.thickness / 2 + tol * 0.35) return { kind: 'wall', id: w.id };
  const rooms = p.rooms.filter((r) => r.level === level && visibleIn(r.status, phase) && pointInPolygon(q, r.points));
  rooms.sort((a, b) => polygonArea(a.points) - polygonArea(b.points));
  return rooms[0] ? { kind: 'room', id: rooms[0].id } : null;
}

export function nearestWall(p: Project, level: string, phase: Phase, q: Pt, max: number) {
  let best: { wall: (typeof p.walls)[number]; t: number; distance: number } | null = null;
  for (const w of levelWalls(p, level, phase)) {
    const r = projectOnWall(w, q);
    if (r.distance < max && (!best || r.distance < best.distance)) best = { wall: w, t: r.t, distance: r.distance };
  }
  return best;
}

export interface SnapOptions {
  grid: number;
  tolerance: number;
  from?: Pt | null;
  ortho?: boolean;
  disabled?: boolean;
  ignoreWall?: string;
}

export interface SnapResult {
  point: Pt;
  kind: 'end' | 'wall' | 'guide' | 'grid' | 'free';
  guides: { a: Pt; b: Pt }[];
}

const onGrid = (v: number, g: number) => Math.round(v / g) * g;
const clean = (p: Pt): Pt => ({ x: Math.round(p.x * 1000) / 1000, y: Math.round(p.y * 1000) / 1000 });

export function snapPoint(p: Project, level: string, phase: Phase, raw: Pt, o: SnapOptions): SnapResult {
  if (o.disabled) return { point: clean(raw), kind: 'free', guides: [] };
  let q = raw;
  if (o.from && o.ortho) {
    const a = Math.atan2(raw.y - o.from.y, raw.x - o.from.x);
    const s = (Math.round(a / (Math.PI / 4)) * Math.PI) / 4;
    const l = dist(raw, o.from);
    q = { x: o.from.x + Math.cos(s) * l, y: o.from.y + Math.sin(s) * l };
  }
  const walls = levelWalls(p, level, phase).filter((w) => w.id !== o.ignoreWall);
  let end: Pt | null = null;
  let bestEnd = o.tolerance;
  for (const w of walls) {
    for (const e of [w.a, w.b]) {
      const d = dist(e, q);
      if (d < bestEnd) {
        bestEnd = d;
        end = e;
      }
    }
  }
  if (end) return { point: { ...end }, kind: 'end', guides: [] };
  let onWall: Pt | null = null;
  let bestWall = o.tolerance * 0.7;
  for (const w of walls) {
    const r = projectOnWall(w, q);
    if (r.distance < bestWall && wallLength(w) > 0) {
      bestWall = r.distance;
      onWall = r.point;
    }
  }
  if (onWall) return { point: clean(onWall), kind: 'wall', guides: [] };

  const guides: SnapResult['guides'] = [];
  let x = onGrid(q.x, o.grid);
  let y = onGrid(q.y, o.grid);
  let kind: SnapResult['kind'] = 'grid';
  let dx = o.tolerance * 0.6;
  let dy = o.tolerance * 0.6;
  for (const w of walls) {
    for (const e of [w.a, w.b]) {
      if (Math.abs(e.x - q.x) < dx) {
        dx = Math.abs(e.x - q.x);
        x = e.x;
        guides[0] = { a: e, b: { x: e.x, y: q.y } };
        kind = 'guide';
      }
      if (Math.abs(e.y - q.y) < dy) {
        dy = Math.abs(e.y - q.y);
        y = e.y;
        guides[1] = { a: e, b: { x: q.x, y: e.y } };
        kind = 'guide';
      }
    }
  }
  let point = clean({ x, y });
  if (o.from && o.ortho) {
    // Garde la direction imposée : seule la longueur s'aligne sur la grille.
    const l = dist(o.from, q);
    const step = Math.max(o.grid, 0.01);
    point = clean(lerp(o.from, q, l ? onGrid(l, step) / l : 0));
    kind = 'grid';
  }
  return { point, kind, guides: guides.filter(Boolean).map((g) => ({ a: g.a, b: g === guides[0] ? { x: point.x, y: point.y } : { x: point.x, y: point.y } })) };
}
