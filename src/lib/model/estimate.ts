import { catalogEntry, isGlazedKind, MATERIALS } from './catalog';
import { polygonArea, polygonPerimeter, wallLength } from './geometry';
import type { Funding, Project } from './types';

export interface EstimateLine {
  lot: string;
  label: string;
  quantity: number;
  unit: string;
  price: number;
  total: number;
}

export interface Estimate {
  floorArea: number;
  carrezArea: number;
  demolishLength: number;
  demolishArea: number;
  demolishVolume: number;
  newWallLength: number;
  newWallArea: number;
  paintArea: number;
  lines: EstimateLine[];
  metered: number;
  works: number;
  rge: number;
  byFunding: Record<Funding, number>;
  byLot: { lot: string; total: number }[];
  total: number;
}

export const levelHeight = (p: Project, id: string) => p.levels.find((l) => l.id === id)?.height ?? 2.5;

/** Quantitatifs tirés du plan et postes saisis à la main, sans double compte. */
export function computeEstimate(p: Project): Estimate {
  const lines: EstimateLine[] = [];
  const push = (lot: string, label: string, quantity: number, unit: string, price: number) => {
    if (quantity > 0.0001) lines.push({ lot, label, quantity, unit, price, total: quantity * price });
  };
  let floorArea = 0;
  let carrezArea = 0;
  let paintArea = 0;
  const floors = new Map<string, number>();
  for (const r of p.rooms) {
    if (r.status === 'demolish') continue;
    const area = polygonArea(r.points);
    const ceiling = r.ceiling ?? levelHeight(p, r.level);
    floorArea += area;
    if (ceiling >= 1.8) carrezArea += area;
    if (r.status === 'new') {
      floors.set(r.material, (floors.get(r.material) ?? 0) + area);
      paintArea += polygonPerimeter(r.points) * ceiling;
    }
  }
  let demolishLength = 0;
  let demolishArea = 0;
  let demolishVolume = 0;
  let newWallLength = 0;
  let newWallArea = 0;
  let newBearingArea = 0;
  for (const w of p.walls) {
    const len = wallLength(w);
    const area = len * (w.height ?? levelHeight(p, w.level));
    if (w.status === 'demolish') {
      demolishLength += len;
      demolishArea += area;
      demolishVolume += area * w.thickness;
    } else if (w.status === 'new') {
      newWallLength += len;
      newWallArea += area;
      if (w.kind !== 'partition') newBearingArea += area;
    }
  }
  push('Démolition', 'Dépose de murs et cloisons', demolishArea, 'm²', p.prices.demolishWall);
  push('Gros œuvre', 'Murs neufs', newBearingArea, 'm²', p.prices.newBearing);
  push('Plâtrerie', 'Cloisons neuves', newWallArea - newBearingArea, 'm²', p.prices.newPartition);
  for (const m of MATERIALS) push('Sols', m.label, floors.get(m.id) ?? 0, 'm²', p.prices.flooring[m.id]);
  push('Peinture', 'Murs des pièces créées', paintArea, 'm²', p.prices.paint);
  const doors = p.openings.filter((o) => o.status === 'new' && !isGlazedKind(o.kind)).length;
  const windows = p.openings.filter((o) => o.status === 'new' && isGlazedKind(o.kind)).length;
  push('Menuiseries', 'Portes neuves', doors, 'u', p.prices.door);
  push('Menuiseries', 'Fenêtres neuves', windows, 'u', p.prices.window);
  for (const i of p.items) {
    const price = i.cost ?? catalogEntry(i.kind).cost;
    if (i.status === 'new' && price > 0) push('Équipement', i.label || catalogEntry(i.kind).label, 1, 'u', price);
  }
  const metered = lines.reduce((s, l) => s + l.total, 0);
  const byFunding: Record<Funding, number> = { 'eco-ptz': 0, bank: 0, cash: 0, aid: 0, none: 0 };
  const lots = new Map<string, number>();
  let works = 0;
  let rge = 0;
  for (const w of p.works) {
    works += w.amount;
    if (w.rge) rge += w.amount;
    byFunding[w.funding] += w.amount;
    lots.set(w.lot, (lots.get(w.lot) ?? 0) + w.amount);
  }
  return {
    floorArea,
    carrezArea,
    demolishLength,
    demolishArea,
    demolishVolume,
    newWallLength,
    newWallArea,
    paintArea,
    lines,
    metered,
    works,
    rge,
    byFunding,
    byLot: [...lots].map(([lot, total]) => ({ lot, total })).sort((a, b) => b.total - a.total),
    // Les postes saisis priment : ils remplacent le métré dès qu'ils existent.
    total: p.works.length ? works : metered,
  };
}

const num = (v: number, digits: number) => v.toLocaleString('fr-FR', { minimumFractionDigits: digits, maximumFractionDigits: digits });
export const fmtEur = (v: number) => `${Math.round(v).toLocaleString('fr-FR')} €`;
export const fmtArea = (v: number) => `${num(v, 1)} m²`;
export const fmtLen = (v: number, unit: 'm' | 'cm' = 'm') => (unit === 'cm' ? `${Math.round(v * 100)}` : `${num(v, 2)} m`);

export function estimateCsv(p: Project): string {
  const e = computeEstimate(p);
  const q = (s: string) => `"${s.replaceAll('"', '""')}"`;
  const rows = [['Source', 'Lot', 'Poste', 'Quantité', 'Unité', 'Prix unitaire', 'Total', 'RGE', 'Artisan', 'Financement', 'État'].map(q).join(';')];
  for (const l of e.lines) rows.push([q('Plan'), q(l.lot), q(l.label), num(l.quantity, 2), l.unit, num(l.price, 2), num(l.total, 2), '', '', '', ''].join(';'));
  for (const w of p.works) rows.push([q('Poste'), q(w.lot), q(w.label), '1', 'forfait', num(w.amount, 2), num(w.amount, 2), w.rge ? 'oui' : 'non', q(w.artisan), w.funding, w.state].join(';'));
  return `﻿${rows.join('\r\n')}\r\n`;
}
