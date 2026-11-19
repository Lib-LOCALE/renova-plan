import { catalogEntry, OPENING_DEFAULTS } from './catalog';
import { add, bounds, mul, pointInPolygon, projectOnWall, round, wallLength, wallNormal } from './geometry';
import { addBoxedRoom, addItem, addOpening, addStair, clampOffset, deleteRefs } from './ops';
import type { FloorMaterial, Item, Opening, OpeningKind, Phase, Project, Pt, Ref, Room, Stair, StairShape, Status, Wall, WallKind } from './types';
import { visibleIn } from './types';

export interface RoomPreset {
  id: string;
  name: string;
  w: number;
  h: number;
  material: FloorMaterial;
  color: string;
}

export const ROOM_PRESETS: RoomPreset[] = [
  { id: 'living', name: 'Séjour', w: 5, h: 4, material: 'oak', color: '#f3c969' },
  { id: 'bedroom', name: 'Chambre', w: 3.5, h: 3, material: 'oak', color: '#a98ccf' },
  { id: 'kitchen', name: 'Cuisine', w: 3.5, h: 2.8, material: 'tile', color: '#5fbfa8' },
  { id: 'bath', name: 'Salle de bain', w: 2.5, h: 2, material: 'tile', color: '#62b7d3' },
  { id: 'wc', name: 'WC', w: 1.2, h: 1.6, material: 'tile', color: '#7f9fdc' },
  { id: 'office', name: 'Bureau', w: 3, h: 2.6, material: 'oak', color: '#f2a65a' },
  { id: 'hall', name: 'Entrée', w: 2, h: 1.8, material: 'tile', color: '#cdb596' },
  { id: 'garage', name: 'Garage', w: 3, h: 6, material: 'concrete', color: '#b4bcc4' },
];

// Ces meubles se posent librement ; les autres viennent s'adosser au mur le plus proche.
const FREE = new Set(['coffee-table', 'dining-table', 'rug', 'island', 'plant', 'office-chair', 'bathtub-free']);
const BACK_AT_FRONT = new Set(['sofa', 'armchair', 'chair']);
const grid = (v: number, s = 0.05) => Math.round(v / s) * s;
const walls = (p: Project, level: string, phase: Phase) => p.walls.filter((w) => w.level === level && visibleIn(w.status, phase));

function nearestWall(list: Wall[], q: Pt, reach: (w: Wall) => number) {
  let best: { wall: Wall; t: number; point: Pt; distance: number } | null = null;
  for (const wall of list) {
    const r = projectOnWall(wall, q);
    if (r.distance <= reach(wall) && (!best || r.distance < best.distance)) best = { wall, ...r };
  }
  return best;
}

export function placeItem(p: Project, level: string, phase: Phase, kind: string, q: Pt, status: Status, turn = 0): Item {
  const e = catalogEntry(kind);
  let at = { x: grid(q.x), y: grid(q.y) };
  let rotation = turn;
  if (!FREE.has(kind)) {
    const near = nearestWall(walls(p, level, phase), q, (w) => w.thickness / 2 + e.d / 2 + 0.45);
    if (near) {
      const n = wallNormal(near.wall);
      const side = (q.x - near.point.x) * n.x + (q.y - near.point.y) * n.y >= 0 ? 1 : -1;
      at = add(near.point, mul(n, side * (near.wall.thickness / 2 + e.d / 2 + 0.015)));
      const toWall = mul(n, -side);
      const deg = BACK_AT_FRONT.has(kind) ? Math.atan2(-toWall.x, toWall.y) : Math.atan2(toWall.x, -toWall.y);
      rotation = Math.round((deg * 180) / Math.PI) + turn;
    }
  }
  return addItem(p, level, kind, at.x, at.y, status, { rotation });
}

export function placeOpening(p: Project, level: string, phase: Phase, kind: OpeningKind, q: Pt, status: Status): Opening | null {
  const near = nearestWall(walls(p, level, phase), q, () => 1.1);
  if (!near || wallLength(near.wall) < OPENING_DEFAULTS[kind].width + 0.06) return null;
  return addOpening(p, near.wall, clampOffset(near.wall, grid(near.t), OPENING_DEFAULTS[kind].width), kind, status);
}

/** Recale un rectangle sur les murs voisins pour que les pièces s'emboîtent seules. */
export function snapRoomRect(p: Project, level: string, phase: Phase, x: number, y: number, w: number, h: number) {
  let dx = 0.4;
  let dy = 0.4;
  let sx = grid(x);
  let sy = grid(y);
  const nearSpan = (a: number, b: number, c: number, d: number) => Math.min(a, b) < d + 0.4 && Math.max(a, b) > c - 0.4;
  for (const wall of walls(p, level, phase)) {
    const t = wall.thickness / 2;
    if (Math.abs(wall.a.x - wall.b.x) < 0.01 && nearSpan(wall.a.y, wall.b.y, y, y + h)) {
      for (const face of [wall.a.x - t, wall.a.x + t]) {
        for (const candidate of [face, face - w]) {
          if (Math.abs(candidate - x) < dx) {
            dx = Math.abs(candidate - x);
            sx = candidate;
          }
        }
      }
    }
    if (Math.abs(wall.a.y - wall.b.y) < 0.01 && nearSpan(wall.a.x, wall.b.x, x, x + w)) {
      for (const face of [wall.a.y - t, wall.a.y + t]) {
        for (const candidate of [face, face - h]) {
          if (Math.abs(candidate - y) < dy) {
            dy = Math.abs(candidate - y);
            sy = candidate;
          }
        }
      }
    }
  }
  return { x: round(sx), y: round(sy) };
}

export function roomOverlaps(p: Project, level: string, phase: Phase, x: number, y: number, w: number, h: number, ignore?: string): boolean {
  return p.rooms.some((r) => {
    if (r.id === ignore || r.level !== level || r.status === 'demolish' || !visibleIn(r.status, phase)) return false;
    const b = bounds(r.points);
    return x < b.maxX - 0.05 && x + w > b.minX + 0.05 && y < b.maxY - 0.05 && y + h > b.minY + 0.05;
  });
}

export function placeRoom(p: Project, level: string, phase: Phase, preset: RoomPreset, q: Pt, status: Status, kind: WallKind = 'partition'): Room {
  const s = snapRoomRect(p, level, phase, q.x - preset.w / 2, q.y - preset.h / 2, preset.w, preset.h);
  const room = addBoxedRoom(p, level, preset.name, s.x, s.y, preset.w, preset.h, preset.material, status, kind);
  room.color = preset.color;
  return room;
}

export function placeStair(p: Project, level: string, shape: StairShape, q: Pt, status: Status, turn = 0): Stair {
  const size = shape === 'straight' ? { w: 2.8, d: 0.9 } : shape === 'quarter' ? { w: 2.3, d: 1 } : { w: 2.1, d: 1.8 };
  return addStair(p, level, grid(q.x), grid(q.y), { ...size, shape, rotation: turn, status, steps: shape === 'half' ? 15 : 14 });
}

export function roomAt(p: Project, level: string, phase: Phase, q: Pt): Room | undefined {
  return p.rooms.find((r) => r.level === level && visibleIn(r.status, phase) && pointInPolygon(q, r.points));
}

type WithStatus = { id: string; status: Status };

function statusList(p: Project, ref: Ref): WithStatus[] | null {
  return ref.kind === 'wall' ? p.walls : ref.kind === 'opening' ? p.openings : ref.kind === 'room' ? p.rooms : ref.kind === 'item' ? p.items : ref.kind === 'stair' ? p.stairs : null;
}

/** Retire ce qui est nouveau ; ce qui existait passe à démolir pour garder la trace du chantier. */
export function eraseRef(p: Project, ref: Ref, phase: Phase) {
  const target = statusList(p, ref)?.find((e) => e.id === ref.id);
  if (!target || target.status === 'new' || phase === 'before') deleteRefs(p, [ref]);
  else target.status = target.status === 'demolish' ? 'existing' : 'demolish';
}

export function rotateRefs(p: Project, refs: Ref[], deg: number) {
  for (const r of refs) {
    const list = r.kind === 'item' ? p.items : r.kind === 'stair' ? p.stairs : null;
    const e = list?.find((x) => x.id === r.id);
    if (e) e.rotation = (e.rotation + deg + 360) % 360;
  }
}
