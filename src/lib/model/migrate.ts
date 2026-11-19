import { catalogEntry, ROOM_CHIPS } from './catalog';
import { rectPoints } from './geometry';
import type { FloorMaterial, OpeningKind, Project, Status, WallKind } from './types';
import { DEFAULT_PRICES, emptyProject } from './types';

type Legacy = Record<string, any>;

const OPENINGS: Record<string, OpeningKind> = {
  single_door: 'door',
  double_door: 'double-door',
  sliding_door: 'sliding-door',
  open_passage: 'passage',
  single_window: 'window',
  double_window: 'window',
  bay_window: 'bay-window',
};
const FLOORS: Record<string, FloorMaterial> = {
  oak_parquet: 'oak',
  light_tile: 'tile',
  slate_tile: 'slate',
  polished_concrete: 'concrete',
  terracotta: 'terracotta',
  warm_carpet: 'carpet',
};
const ITEMS: Record<string, string> = {
  sofa: 'sofa',
  dining_table: 'dining-table',
  coffee_table: 'coffee-table',
  kitchen_island: 'island',
  kitchen_counter: 'counter',
  fridge: 'fridge',
  double_bed: 'bed-double',
  wardrobe: 'wardrobe',
  desk: 'desk',
  bathtub: 'bathtub',
  walk_in_shower: 'shower',
  vanity_sink: 'basin',
  toilet: 'toilet',
  electrical_panel: 'panel',
  water_heater: 'water-heater',
  wood_stove: 'stove',
};
const status = (s: unknown): Status => (s === 'demolish' || s === 'new' ? s : 'existing');

/** Convertit un fichier de l'ancienne version Slint. */
function fromLegacy(old: Legacy): Project {
  const p = emptyProject(old.title_fr || 'Projet importé');
  const names: string[] = old.levels?.length ? old.levels : ['Rez-de-chaussée'];
  p.levels = names.map((name, i) => ({ id: `l${i}`, name, elevation: i * 2.7, height: 2.5, slab: 0.2 }));
  const level = (i: number) => `l${i ?? 0}`;
  p.walls = (old.walls ?? []).map((w: Legacy) => ({
    id: `w${w.id}`,
    level: level(w.level),
    a: { x: w.x1, y: w.y1 },
    b: { x: w.x2, y: w.y2 },
    thickness: w.thickness,
    height: w.height,
    kind: (w.load_bearing ? 'bearing' : 'partition') as WallKind,
    status: status(w.status),
    name: w.name || undefined,
  }));
  p.openings = (old.openings ?? []).flatMap((o: Legacy) => {
    const wall = p.walls.find((w) => w.id === `w${o.wall_id}`);
    if (!wall) return [];
    const len = Math.hypot(wall.b.x - wall.a.x, wall.b.y - wall.a.y);
    return [{
      id: `o${o.id}`,
      wall: wall.id,
      offset: o.offset_t * len,
      width: o.width,
      height: o.height,
      sill: o.sill_height,
      kind: OPENINGS[o.kind] ?? 'door',
      status: status(o.status),
      flip: !!o.flip_swing,
    }];
  });
  p.rooms = (old.rooms ?? []).map((r: Legacy, i: number) => ({
    id: `r${r.id}`,
    level: level(r.level),
    name: r.name_fr || 'Pièce',
    points: rectPoints(r.x, r.y, r.width, r.depth),
    material: FLOORS[r.material] ?? 'oak',
    color: ROOM_CHIPS[i % ROOM_CHIPS.length].color,
    ceiling: r.ceiling_height,
    status: status(r.status),
  }));
  p.items = (old.furniture ?? []).map((f: Legacy) => {
    const kind = ITEMS[f.kind] ?? 'sofa';
    return {
      id: `i${f.id}`,
      level: level(f.level),
      kind,
      x: f.x,
      y: f.y,
      w: f.width,
      d: f.depth,
      h: f.height,
      rotation: f.rotation_deg ?? 0,
      status: status(f.status),
      color: catalogEntry(kind).color,
      label: f.label_custom || undefined,
      cost: f.cost,
    };
  });
  return p;
}

/** Lit un projet depuis du JSON et refuse clairement ce qui n'en est pas un. */
export function parseProject(text: string): Project {
  let data: Legacy;
  try {
    data = JSON.parse(text);
  } catch {
    throw new Error("Ce fichier n'est pas un projet lisible : le JSON est invalide.");
  }
  if (data?.format === 'renovaplan') {
    if (data.version !== 2) throw new Error(`Version de projet ${data.version} non prise en charge.`);
    const base = emptyProject(data.name);
    for (const [key, value] of Object.entries(base)) {
      if (Array.isArray(value) && key in data && !Array.isArray(data[key])) {
        throw new Error(`Liste de projet invalide (${key}).`);
      }
    }
    if (data.levels?.length === 0) throw new Error('Le projet doit contenir un niveau.');
    return {
      ...base, ...data,
      prices: { ...DEFAULT_PRICES, ...data.prices, flooring: { ...DEFAULT_PRICES.flooring, ...data.prices?.flooring } },
    } as Project;
  }
  if (Array.isArray(data?.walls) && 'title_fr' in data) return fromLegacy(data);
  throw new Error("Ce fichier ne ressemble pas à un projet RenovaPlan.");
}
