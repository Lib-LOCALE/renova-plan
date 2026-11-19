import { catalogEntry, OPENING_DEFAULTS, ROOM_CHIPS, WALL_DEFAULTS } from './catalog';
import {
  add,
  boxCorners,
  dist,
  mul,
  pointInPolygon,
  projectOnWall,
  rectPoints,
  round,
  sub,
  wallDir,
  wallLength,
  wallNormal,
} from './geometry';
import type { FloorMaterial, Item, Opening, OpeningKind, Project, Pt, Ref, Room, Stair, Status, Wall, WallKind } from './types';
import { uid } from './types';

const JOINT = 0.025;
const roundPt = (p: Pt): Pt => ({ x: round(p.x), y: round(p.y) });

export function addWall(
  p: Project,
  level: string,
  a: Pt,
  b: Pt,
  kind: WallKind = 'partition',
  status: Status = 'existing',
  extra: Partial<Wall> = {},
): Wall {
  const wall: Wall = {
    id: uid('w'),
    level,
    a: roundPt(a),
    b: roundPt(b),
    thickness: WALL_DEFAULTS[kind].thickness,
    kind,
    status,
    ...extra,
  };
  p.walls.push(wall);
  return wall;
}

/** Un mur existant couvre-t-il déjà le segment demandé ? */
function isCovered(p: Project, level: string, a: Pt, b: Pt): boolean {
  return p.walls.some((w) => {
    if (w.level !== level) return false;
    const tol = w.thickness / 2 + 0.16;
    return projectOnWall(w, a).distance < tol && projectOnWall(w, b).distance < tol;
  });
}

export function addRoom(
  p: Project,
  level: string,
  name: string,
  points: Pt[],
  material: FloorMaterial = 'oak',
  status: Status = 'existing',
  color?: string,
): Room {
  const room: Room = {
    id: uid('r'),
    level,
    name,
    points: points.map(roundPt),
    material,
    status,
    color: color ?? ROOM_CHIPS[p.rooms.length % ROOM_CHIPS.length].color,
  };
  p.rooms.push(room);
  return room;
}

/** Pièce rectangulaire avec ses quatre murs ; les murs déjà présents sont réutilisés. */
export function addBoxedRoom(
  p: Project,
  level: string,
  name: string,
  x: number,
  y: number,
  w: number,
  h: number,
  material: FloorMaterial = 'oak',
  status: Status = 'existing',
  kind: WallKind = 'partition',
): Room {
  const t = WALL_DEFAULTS[kind].thickness / 2;
  const [c1, c2, c3, c4] = rectPoints(x - t, y - t, w + 2 * t, h + 2 * t);
  for (const [a, b] of [[c1, c2], [c2, c3], [c3, c4], [c4, c1]] as [Pt, Pt][]) {
    if (!isCovered(p, level, a, b)) addWall(p, level, a, b, kind, status);
  }
  return addRoom(p, level, name, rectPoints(x, y, w, h), material, status);
}

export function addOpening(p: Project, wall: Wall, offset: number, kind: OpeningKind, status: Status = 'existing', extra: Partial<Opening> = {}): Opening {
  const d = OPENING_DEFAULTS[kind];
  const width = extra.width ?? d.width;
  const opening: Opening = {
    id: uid('o'),
    wall: wall.id,
    offset: clampOffset(wall, offset, width),
    width,
    height: d.height,
    sill: d.sill,
    kind,
    status,
    ...extra,
  };
  opening.offset = clampOffset(wall, opening.offset, opening.width);
  p.openings.push(opening);
  return opening;
}

export function clampOffset(wall: Wall, offset: number, width: number): number {
  const len = wallLength(wall);
  const m = width / 2 + 0.03;
  return len <= 2 * m ? len / 2 : Math.max(m, Math.min(len - m, offset));
}

export function addItem(p: Project, level: string, kind: string, x: number, y: number, status: Status = 'existing', extra: Partial<Item> = {}): Item {
  const e = catalogEntry(kind);
  const item: Item = { id: uid('i'), level, kind: e.kind, x, y, w: e.w, d: e.d, h: e.h, rotation: 0, status, color: e.color, cost: e.cost, ...extra };
  p.items.push(item);
  return item;
}

export function addStair(p: Project, level: string, x: number, y: number, extra: Partial<Stair> = {}): Stair {
  const s: Stair = { id: uid('s'), level, x, y, w: 2.1, d: 1, rotation: 0, shape: 'quarter', steps: 14, status: 'existing', ...extra };
  p.stairs.push(s);
  return s;
}

interface WallMove {
  wall: Wall;
  da: Pt;
  db: Pt;
}

/** Fait suivre les sommets de pièce collés aux murs déplacés, composante perpendiculaire seule. */
function followRooms(p: Project, level: string, moves: WallMove[], before: Map<string, Pick<Wall, 'a' | 'b'>>) {
  for (const room of p.rooms) {
    if (room.level !== level) continue;
    room.points = room.points.map((v) => {
      let shift = { x: 0, y: 0 };
      for (const m of moves) {
        const old = before.get(m.wall.id)!;
        const q = projectOnWall(old, v);
        if (q.distance > m.wall.thickness / 2 + 0.04) continue;
        const f = q.t / (wallLength(old) || 1);
        const delta = add(mul(m.da, 1 - f), mul(m.db, f));
        const n = wallNormal(old);
        shift = add(shift, mul(n, delta.x * n.x + delta.y * n.y));
      }
      return roundPt(add(v, shift));
    });
  }
}

function applyWallMoves(p: Project, level: string, requests: Map<string, { da?: Pt; db?: Pt }>) {
  const zero = { x: 0, y: 0 };
  const before = new Map(p.walls.filter((w) => w.level === level).map((w) => [w.id, { a: { ...w.a }, b: { ...w.b } }]));
  const endpoint = new Map<string, Pt>();
  const walls = p.walls.filter((w) => w.level === level);
  const queue: [Pt, Pt][] = [];
  for (const [id, r] of requests) {
    const w = walls.find((x) => x.id === id);
    if (!w) continue;
    if (r.da) queue.push([w.a, r.da]);
    if (r.db) queue.push([w.b, r.db]);
  }
  const keyOf = (w: Wall, end: 'a' | 'b') => `${w.id}:${end}`;
  for (const [pt, delta] of queue) {
    for (const w of walls) {
      for (const end of ['a', 'b'] as const) {
        if (dist(w[end], pt) < JOINT && !endpoint.has(keyOf(w, end))) endpoint.set(keyOf(w, end), delta);
      }
    }
  }
  const moves: WallMove[] = [];
  for (const w of walls) {
    const da = endpoint.get(keyOf(w, 'a')) ?? zero;
    const db = endpoint.get(keyOf(w, 'b')) ?? zero;
    if (da === zero && db === zero) continue;
    w.a = roundPt(add(w.a, da));
    w.b = roundPt(add(w.b, db));
    moves.push({ wall: w, da, db });
    for (const o of p.openings) if (o.wall === w.id) o.offset = clampOffset(w, o.offset, o.width);
  }
  followRooms(p, level, moves, before);
}

export function moveWall(p: Project, id: string, delta: Pt) {
  const w = p.walls.find((x) => x.id === id);
  if (w) applyWallMoves(p, w.level, new Map([[id, { da: delta, db: delta }]]));
}

export function moveWallEnd(p: Project, id: string, end: 'a' | 'b', to: Pt) {
  const w = p.walls.find((x) => x.id === id);
  if (!w) return;
  const delta = sub(to, w[end]);
  applyWallMoves(p, w.level, new Map([[id, end === 'a' ? { da: delta } : { db: delta }]]));
}

export function moveRoomVertex(p: Project, id: string, index: number, to: Pt) {
  const r = p.rooms.find((x) => x.id === id);
  if (r) r.points[index] = roundPt(to);
}

/** Déplace un côté de pièce et le mur qui le borde. */
export function moveRoomEdge(p: Project, id: string, index: number, delta: Pt) {
  const r = p.rooms.find((x) => x.id === id);
  if (!r) return;
  const j = (index + 1) % r.points.length;
  const a = r.points[index];
  const b = r.points[j];
  const e = wallDir({ a, b });
  const n = { x: -e.y, y: e.x };
  const shift = mul(n, delta.x * n.x + delta.y * n.y);
  const mid = { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
  const wall = p.walls
    .filter((w) => w.level === r.level)
    .map((w) => ({ w, q: projectOnWall(w, mid) }))
    .filter(({ w, q }) => q.distance <= w.thickness / 2 + 0.04)
    .sort((m, n2) => m.q.distance - n2.q.distance)[0]?.w;
  if (wall) moveWall(p, wall.id, shift);
  const fresh = p.rooms.find((x) => x.id === id)!;
  if (!wall) {
    fresh.points[index] = roundPt(add(a, shift));
    fresh.points[j] = roundPt(add(b, shift));
  }
}

/** Déplace une pièce avec ses murs, ouvertures et meubles. */
export function moveRoom(p: Project, id: string, delta: Pt) {
  const r = p.rooms.find((x) => x.id === id);
  if (!r) return;
  const edges = r.points.map((a, i) => ({ a, b: r.points[(i + 1) % r.points.length] }));
  const near = (q: Pt, t: number) => edges.some((e) => projectOnWall(e, q).distance <= t / 2 + 0.05);
  for (const w of p.walls) {
    if (w.level !== r.level || !near(w.a, w.thickness + 0.05) || !near(w.b, w.thickness + 0.05)) continue;
    w.a = roundPt(add(w.a, delta));
    w.b = roundPt(add(w.b, delta));
  }
  for (const item of p.items) {
    if (item.level === r.level && pointInPolygon(item, r.points)) {
      item.x = round(item.x + delta.x);
      item.y = round(item.y + delta.y);
    }
  }
  r.points = r.points.map((v) => roundPt(add(v, delta)));
}

export function moveRef(p: Project, ref: Ref, delta: Pt) {
  const box = (e: { x: number; y: number }) => {
    e.x = round(e.x + delta.x);
    e.y = round(e.y + delta.y);
  };
  if (ref.kind === 'wall') moveWall(p, ref.id, delta);
  else if (ref.kind === 'room') moveRoom(p, ref.id, delta);
  else if (ref.kind === 'item') box(p.items.find((e) => e.id === ref.id)!);
  else if (ref.kind === 'stair') box(p.stairs.find((e) => e.id === ref.id)!);
  else if (ref.kind === 'note') box(p.notes.find((e) => e.id === ref.id)!);
  else if (ref.kind === 'dim') {
    const d = p.dims.find((e) => e.id === ref.id)!;
    d.a = roundPt(add(d.a, delta));
    d.b = roundPt(add(d.b, delta));
  } else if (ref.kind === 'opening') {
    const o = p.openings.find((e) => e.id === ref.id)!;
    const w = p.walls.find((e) => e.id === o.wall)!;
    const dir = wallDir(w);
    o.offset = clampOffset(w, o.offset + delta.x * dir.x + delta.y * dir.y, o.width);
  }
}

export function deleteRefs(p: Project, refs: Ref[]) {
  const ids = (kind: Ref['kind']) => new Set(refs.filter((r) => r.kind === kind).map((r) => r.id));
  const walls = ids('wall');
  const openings = ids('opening');
  const rooms = ids('room');
  const items = ids('item');
  const stairs = ids('stair');
  const dims = ids('dim');
  const notes = ids('note');
  p.walls = p.walls.filter((e) => !walls.has(e.id));
  p.openings = p.openings.filter((e) => !openings.has(e.id) && !walls.has(e.wall));
  p.rooms = p.rooms.filter((e) => !rooms.has(e.id));
  p.items = p.items.filter((e) => !items.has(e.id));
  p.stairs = p.stairs.filter((e) => !stairs.has(e.id));
  p.dims = p.dims.filter((e) => !dims.has(e.id));
  p.notes = p.notes.filter((e) => !notes.has(e.id));
}

export function duplicateRefs(p: Project, refs: Ref[], offset: Pt = { x: 0.3, y: 0.3 }): Ref[] {
  const out: Ref[] = [];
  const shift = (q: Pt) => roundPt(add(q, offset));
  for (const r of refs) {
    if (r.kind === 'wall') {
      const e = p.walls.find((x) => x.id === r.id)!;
      const c = { ...structuredClone(e), id: uid('w'), a: shift(e.a), b: shift(e.b) };
      p.walls.push(c);
      out.push({ kind: 'wall', id: c.id });
    } else if (r.kind === 'room') {
      const e = p.rooms.find((x) => x.id === r.id)!;
      const c = { ...structuredClone(e), id: uid('r'), points: e.points.map(shift), name: `${e.name} 2` };
      p.rooms.push(c);
      out.push({ kind: 'room', id: c.id });
    } else if (r.kind === 'item') {
      const e = p.items.find((x) => x.id === r.id)!;
      const c = { ...e, id: uid('i'), x: e.x + offset.x, y: e.y + offset.y };
      p.items.push(c);
      out.push({ kind: 'item', id: c.id });
    } else if (r.kind === 'stair') {
      const e = p.stairs.find((x) => x.id === r.id)!;
      const c = { ...e, id: uid('s'), x: e.x + offset.x, y: e.y + offset.y };
      p.stairs.push(c);
      out.push({ kind: 'stair', id: c.id });
    } else if (r.kind === 'opening') {
      const e = p.openings.find((x) => x.id === r.id)!;
      const w = p.walls.find((x) => x.id === e.wall)!;
      const c = { ...e, id: uid('o'), tag: undefined, offset: clampOffset(w, e.offset + e.width + 0.2, e.width) };
      p.openings.push(c);
      out.push({ kind: 'opening', id: c.id });
    }
  }
  return out;
}

export function setStatus(p: Project, refs: Ref[], status: Status) {
  for (const r of refs) {
    const list: { id: string; status?: Status }[] | null =
      r.kind === 'wall' ? p.walls : r.kind === 'opening' ? p.openings : r.kind === 'room' ? p.rooms : r.kind === 'item' ? p.items : r.kind === 'stair' ? p.stairs : null;
    const e = list?.find((x) => x.id === r.id);
    if (e) e.status = status;
  }
}

export function levelBounds(p: Project, level?: string) {
  const pts: Pt[] = [];
  for (const w of p.walls) if (!level || w.level === level) pts.push(w.a, w.b);
  for (const r of p.rooms) if (!level || r.level === level) pts.push(...r.points);
  for (const i of p.items) if (!level || i.level === level) pts.push(...boxCorners(i));
  return pts;
}
