/** Modèle de projet RenovaPlan. Toutes les longueurs sont en mètres. x vers l'est, y vers le sud. */
export interface Pt {
  x: number;
  y: number;
}

export type Status = 'existing' | 'demolish' | 'new';
export type Phase = 'before' | 'site' | 'after';
export type WallKind = 'exterior' | 'bearing' | 'partition' | 'party';
export type OpeningKind =
  | 'door'
  | 'double-door'
  | 'sliding-door'
  | 'passage'
  | 'entry-door'
  | 'garage-door'
  | 'window'
  | 'french-window'
  | 'bay-window';
export type FloorMaterial = 'oak' | 'tile' | 'slate' | 'concrete' | 'terracotta' | 'carpet' | 'vinyl';
export type StairShape = 'straight' | 'quarter' | 'half';
export type RoofKind = 'gable' | 'flat';
export type Funding = 'eco-ptz' | 'bank' | 'cash' | 'aid' | 'none';
export type WorkState = 'idea' | 'quote' | 'signed' | 'done';

export interface Level {
  id: string;
  name: string;
  elevation: number;
  height: number;
  slab: number;
}

export interface Wall {
  id: string;
  level: string;
  a: Pt;
  b: Pt;
  thickness: number;
  height?: number;
  kind: WallKind;
  status: Status;
  name?: string;
}

export interface Opening {
  id: string;
  wall: string;
  /** Distance du point a du mur au centre de l'ouverture. */
  offset: number;
  width: number;
  height: number;
  sill: number;
  kind: OpeningKind;
  status: Status;
  flip?: boolean;
  hinge?: boolean;
  tag?: string;
  note?: string;
}

export interface Room {
  id: string;
  level: string;
  name: string;
  points: Pt[];
  material: FloorMaterial;
  color: string;
  ceiling?: number;
  status: Status;
  /** Surface de référence (annonce, diagnostic) pour contrôler le relevé. */
  referenceArea?: number;
}

export interface Item {
  id: string;
  level: string;
  kind: string;
  x: number;
  y: number;
  w: number;
  d: number;
  h: number;
  rotation: number;
  status: Status;
  color?: string;
  label?: string;
  cost?: number;
}

export interface Stair {
  id: string;
  level: string;
  x: number;
  y: number;
  w: number;
  d: number;
  rotation: number;
  shape: StairShape;
  steps: number;
  flip?: boolean;
  status: Status;
  /** Limite l'escalier au plan ou à la maquette quand les deux lectures diffèrent. */
  view?: '2d' | '3d';
}

export interface Dimension {
  id: string;
  level: string;
  a: Pt;
  b: Pt;
  offset: number;
  label?: string;
  flag?: boolean;
  only?: 'before' | 'after';
}

export interface Note {
  id: string;
  level: string;
  x: number;
  y: number;
  text: string;
  flag?: boolean;
}

export interface Roof {
  id: string;
  level: string;
  x: number;
  y: number;
  w: number;
  d: number;
  kind: RoofKind;
  ridge: 'x' | 'y';
  rise: number;
  overhang: number;
  color: string;
  /** Hauteur de pose depuis le sol du niveau ; par défaut, le haut du niveau. */
  base?: number;
}

export interface Underlay {
  level: string;
  src: string;
  x: number;
  y: number;
  /** Largeur de l'image en mètres. */
  width: number;
  aspect: number;
  opacity: number;
  visible: boolean;
}

export interface WorkItem {
  id: string;
  lot: string;
  label: string;
  amount: number;
  rge: boolean;
  artisan: string;
  funding: Funding;
  state: WorkState;
  note?: string;
}

export interface Prices {
  demolishWall: number;
  newPartition: number;
  newBearing: number;
  paint: number;
  flooring: Record<FloorMaterial, number>;
  door: number;
  window: number;
}

export interface Project {
  format: 'renovaplan';
  version: 2;
  name: string;
  north: number;
  dimUnit: 'm' | 'cm';
  levels: Level[];
  walls: Wall[];
  openings: Opening[];
  rooms: Room[];
  items: Item[];
  stairs: Stair[];
  dims: Dimension[];
  notes: Note[];
  roofs: Roof[];
  underlays: Underlay[];
  works: WorkItem[];
  prices: Prices;
}

export type EntityKind = 'wall' | 'opening' | 'room' | 'item' | 'stair' | 'dim' | 'note';
export interface Ref {
  kind: EntityKind;
  id: string;
}

export const DEFAULT_PRICES: Prices = {
  demolishWall: 55,
  newPartition: 62,
  newBearing: 180,
  paint: 24,
  flooring: { oak: 68, tile: 62, slate: 85, concrete: 78, terracotta: 74, carpet: 34, vinyl: 42 },
  door: 340,
  window: 780,
};

export function visibleIn(status: Status, phase: Phase): boolean {
  if (phase === 'site') return true;
  return phase === 'before' ? status !== 'new' : status !== 'demolish';
}

let counter = 0;
export function uid(prefix = 'e'): string {
  counter += 1;
  return `${prefix}${Date.now().toString(36)}${counter.toString(36)}`;
}

export function emptyProject(name = 'Nouveau projet'): Project {
  return {
    format: 'renovaplan',
    version: 2,
    name,
    north: 0,
    dimUnit: 'm',
    levels: [{ id: 'l0', name: 'Rez-de-chaussée', elevation: 0, height: 2.5, slab: 0.2 }],
    walls: [],
    openings: [],
    rooms: [],
    items: [],
    stairs: [],
    dims: [],
    notes: [],
    roofs: [],
    underlays: [],
    works: [],
    prices: structuredClone(DEFAULT_PRICES),
  };
}
