import type { FloorMaterial, Opening, OpeningKind, WallKind } from './types';

export interface CatalogEntry {
  kind: string;
  label: string;
  group: 'Séjour' | 'Cuisine' | 'Chambre' | 'Bain' | 'Bureau' | 'Technique';
  w: number;
  d: number;
  h: number;
  cost: number;
  color: string;
}

export const CATALOG: CatalogEntry[] = [
  { kind: 'sofa', label: 'Canapé', group: 'Séjour', w: 2.2, d: 0.95, h: 0.82, cost: 950, color: '#6f7f8f' },
  { kind: 'armchair', label: 'Fauteuil', group: 'Séjour', w: 0.85, d: 0.85, h: 0.8, cost: 340, color: '#b5714b' },
  { kind: 'coffee-table', label: 'Table basse', group: 'Séjour', w: 1.1, d: 0.6, h: 0.4, cost: 180, color: '#a97c50' },
  { kind: 'tv-unit', label: 'Meuble TV', group: 'Séjour', w: 1.8, d: 0.42, h: 0.5, cost: 290, color: '#40464d' },
  { kind: 'bookcase', label: 'Bibliothèque', group: 'Séjour', w: 1.2, d: 0.32, h: 2, cost: 260, color: '#c9a877' },
  { kind: 'dining-table', label: 'Table à manger', group: 'Séjour', w: 1.8, d: 0.9, h: 0.75, cost: 520, color: '#a97c50' },
  { kind: 'chair', label: 'Chaise', group: 'Séjour', w: 0.45, d: 0.48, h: 0.86, cost: 70, color: '#3f4750' },
  { kind: 'plant', label: 'Plante', group: 'Séjour', w: 0.5, d: 0.5, h: 1.3, cost: 60, color: '#4f8a55' },
  { kind: 'rug', label: 'Tapis', group: 'Séjour', w: 2, d: 1.4, h: 0.01, cost: 160, color: '#c9bba4' },
  { kind: 'counter', label: 'Plan de travail', group: 'Cuisine', w: 2.4, d: 0.62, h: 0.9, cost: 2200, color: '#e8e6e0' },
  { kind: 'island', label: 'Îlot', group: 'Cuisine', w: 1.8, d: 0.9, h: 0.92, cost: 1800, color: '#34464f' },
  { kind: 'fridge', label: 'Réfrigérateur', group: 'Cuisine', w: 0.7, d: 0.68, h: 1.9, cost: 780, color: '#d6d9dc' },
  { kind: 'hob', label: 'Cuisson et four', group: 'Cuisine', w: 0.6, d: 0.62, h: 0.9, cost: 720, color: '#2d3136' },
  { kind: 'sink-unit', label: 'Évier', group: 'Cuisine', w: 1.2, d: 0.62, h: 0.9, cost: 420, color: '#e8e6e0' },
  { kind: 'bed-double', label: 'Lit double', group: 'Chambre', w: 1.6, d: 2.05, h: 0.6, cost: 780, color: '#dfe3e8' },
  { kind: 'bed-single', label: 'Lit simple', group: 'Chambre', w: 0.9, d: 2, h: 0.55, cost: 340, color: '#dfe3e8' },
  { kind: 'wardrobe', label: 'Armoire', group: 'Chambre', w: 1.8, d: 0.6, h: 2.2, cost: 620, color: '#c9bba4' },
  { kind: 'nightstand', label: 'Chevet', group: 'Chambre', w: 0.45, d: 0.4, h: 0.5, cost: 80, color: '#a97c50' },
  { kind: 'dresser', label: 'Commode', group: 'Chambre', w: 1, d: 0.48, h: 0.85, cost: 210, color: '#c9a877' },
  { kind: 'basin', label: 'Meuble vasque', group: 'Bain', w: 0.9, d: 0.5, h: 0.85, cost: 420, color: '#f0efea' },
  { kind: 'toilet', label: 'WC', group: 'Bain', w: 0.4, d: 0.68, h: 0.8, cost: 380, color: '#f4f4f1' },
  { kind: 'handbasin', label: 'Lave-mains', group: 'Bain', w: 0.4, d: 0.25, h: 0.85, cost: 150, color: '#f4f4f1' },
  { kind: 'shower', label: 'Douche', group: 'Bain', w: 1.2, d: 0.9, h: 2, cost: 980, color: '#bcd6dc' },
  { kind: 'bathtub', label: 'Baignoire', group: 'Bain', w: 1.7, d: 0.75, h: 0.58, cost: 860, color: '#f4f4f1' },
  { kind: 'washer', label: 'Lave-linge', group: 'Bain', w: 0.6, d: 0.6, h: 0.85, cost: 480, color: '#e9ebec' },
  { kind: 'desk', label: 'Bureau', group: 'Bureau', w: 1.4, d: 0.7, h: 0.74, cost: 280, color: '#c9a877' },
  { kind: 'office-chair', label: 'Siège de bureau', group: 'Bureau', w: 0.6, d: 0.6, h: 1.05, cost: 190, color: '#30363c' },
  { kind: 'water-heater', label: 'Chauffe-eau', group: 'Technique', w: 0.58, d: 0.58, h: 1.6, cost: 900, color: '#e8e9ea' },
  { kind: 'heat-pump', label: 'Unité murale', group: 'Technique', w: 0.9, d: 0.25, h: 0.3, cost: 1200, color: '#f1f2f2' },
  { kind: 'outdoor-unit', label: 'Groupe extérieur', group: 'Technique', w: 0.9, d: 0.35, h: 0.7, cost: 1600, color: '#c5c8ca' },
  { kind: 'panel', label: 'Tableau électrique', group: 'Technique', w: 0.6, d: 0.15, h: 0.9, cost: 950, color: '#dfe1e3' },
  { kind: 'radiator', label: 'Radiateur', group: 'Technique', w: 0.9, d: 0.12, h: 0.6, cost: 260, color: '#eceded' },
  { kind: 'stove', label: 'Poêle', group: 'Technique', w: 0.55, d: 0.5, h: 1.1, cost: 1850, color: '#2b2d30' },
];

export const catalogEntry = (kind: string) => CATALOG.find((c) => c.kind === kind) ?? CATALOG[0];
export const CATALOG_GROUPS = ['Séjour', 'Cuisine', 'Chambre', 'Bain', 'Bureau', 'Technique'] as const;

export const OPENING_DEFAULTS: Record<OpeningKind, Pick<Opening, 'width' | 'height' | 'sill'> & { label: string }> = {
  door: { label: 'Porte', width: 0.83, height: 2.04, sill: 0 },
  'double-door': { label: 'Porte double', width: 1.4, height: 2.04, sill: 0 },
  'sliding-door': { label: 'Porte coulissante', width: 0.93, height: 2.04, sill: 0 },
  passage: { label: 'Passage libre', width: 0.9, height: 2.04, sill: 0 },
  'entry-door': { label: "Porte d'entrée", width: 0.9, height: 2.1, sill: 0 },
  'garage-door': { label: 'Porte de garage', width: 2.4, height: 2, sill: 0 },
  window: { label: 'Fenêtre', width: 1.2, height: 1.25, sill: 0.9 },
  'french-window': { label: 'Porte-fenêtre', width: 1.4, height: 2.15, sill: 0 },
  'bay-window': { label: 'Baie vitrée', width: 2.15, height: 2.15, sill: 0 },
};

export const isDoorKind = (k: OpeningKind) => !['window', 'french-window', 'bay-window'].includes(k);
export const isGlazedKind = (k: OpeningKind) => ['window', 'french-window', 'bay-window'].includes(k);

export const WALL_DEFAULTS: Record<WallKind, { label: string; thickness: number }> = {
  exterior: { label: 'Mur extérieur', thickness: 0.25 },
  bearing: { label: 'Mur porteur', thickness: 0.2 },
  partition: { label: 'Cloison', thickness: 0.1 },
  party: { label: 'Mur mitoyen', thickness: 0.25 },
};

export const MATERIALS: { id: FloorMaterial; label: string; code: string; color: string }[] = [
  { id: 'oak', label: 'Parquet chêne', code: 'SOL 01', color: '#c99f6e' },
  { id: 'tile', label: 'Carrelage clair', code: 'SOL 02', color: '#d8d6cf' },
  { id: 'slate', label: 'Ardoise', code: 'SOL 03', color: '#59636b' },
  { id: 'concrete', label: 'Béton ciré', code: 'SOL 04', color: '#a9abaa' },
  { id: 'terracotta', label: 'Terre cuite', code: 'SOL 05', color: '#bd6f4c' },
  { id: 'carpet', label: 'Moquette', code: 'SOL 06', color: '#8c93a3' },
  { id: 'vinyl', label: 'Sol souple', code: 'SOL 07', color: '#b9b09c' },
];

/** Teintes de pièces : un nuancier plutôt qu'une palette de thème. */
export const ROOM_CHIPS: { color: string; name: string; code: string }[] = [
  { color: '#f3c969', name: 'Maïs', code: 'N 101' },
  { color: '#f2a65a', name: 'Abricot', code: 'N 118' },
  { color: '#e9806e', name: 'Corail', code: 'N 124' },
  { color: '#d87f9f', name: 'Bruyère', code: 'N 136' },
  { color: '#a98ccf', name: 'Glycine', code: 'N 142' },
  { color: '#7f9fdc', name: 'Ardoise bleue', code: 'N 155' },
  { color: '#62b7d3', name: 'Lagon', code: 'N 163' },
  { color: '#5fbfa8', name: 'Céladon', code: 'N 170' },
  { color: '#8cc06b', name: 'Tilleul', code: 'N 178' },
  { color: '#c2c46a', name: 'Olive', code: 'N 186' },
  { color: '#cdb596', name: 'Lin', code: 'N 193' },
  { color: '#b4bcc4', name: 'Zinc', code: 'N 199' },
];

export const chipFor = (color: string) => ROOM_CHIPS.find((c) => c.color === color);
