import { rotatePoint } from '../model/geometry';
import type { Item, Pt } from '../model/types';
import { circlePts, type Mesher, roundedRect } from './mesher';

const paint = (c: string) => `paint:${c}`;
const WOOD = '#a97c50';
const METAL = '#8e959b';
const DARK = '#30353a';
const WHITE = '#f3f3ef';

/** Dessine un meuble dans le repère centré de l'objet, directement dans le mesher. */
export function buildItem(m: Mesher, it: Item, base: number) {
  const w = it.w;
  const d = it.d;
  const h = it.h;
  const color = paint(it.color ?? '#bbbbbb');
  const hw = w / 2;
  const hd = d / 2;
  const c = { x: it.x, y: it.y };
  const local = (p: Pt) => rotatePoint({ x: it.x + p.x, y: it.y + p.y }, c, it.rotation);
  const rb = (x0: number, x1: number, z0: number, z1: number, y0: number, y1: number, col = color, r = 0.04) =>
    m.prism(roundedRect(x0, x1, z0, z1, r).map(local), base + y0, base + y1, col, col, col);
  const cyl = (x: number, z: number, r: number, y0: number, y1: number, col = color) => m.prism(circlePts(x, z, r).map(local), base + y0, base + y1, col, col, col);
  const legs = (inset: number, top: number, col = paint(DARK), r = 0.018) => {
    for (const a of [-1, 1]) for (const b of [-1, 1]) cyl(a * (hw - inset), b * (hd - inset), r, 0, top, col);
  };
  const tone = (hex: string, k: number) => {
    const n = parseInt(hex.slice(1), 16);
    const f = (v: number) => Math.max(0, Math.min(255, Math.round(v * k))).toString(16).padStart(2, '0');
    return paint(`#${f(n >> 16)}${f((n >> 8) & 255)}${f(n & 255)}`);
  };
  const base2 = it.color ?? '#bbbbbb';
  switch (it.kind) {
    case 'sofa':
      rb(-hw, hw, -hd, hd, 0.1, 0.36, color, 0.08);
      rb(-hw, hw, hd - 0.24, hd, 0.3, h, color, 0.08);
      rb(-hw, -hw + 0.2, -hd, hd, 0.3, 0.6, color, 0.07);
      rb(hw - 0.2, hw, -hd, hd, 0.3, 0.6, color, 0.07);
      rb(-hw + 0.22, -0.01, -hd + 0.03, hd - 0.25, 0.36, 0.47, tone(base2, 1.12), 0.06);
      rb(0.01, hw - 0.22, -hd + 0.03, hd - 0.25, 0.36, 0.47, tone(base2, 1.12), 0.06);
      legs(0.08, 0.1);
      break;
    case 'armchair':
      rb(-hw, hw, -hd, hd, 0.16, 0.4, color, 0.08);
      rb(-hw, hw, hd - 0.18, hd, 0.36, h, color, 0.07);
      rb(-hw, -hw + 0.14, -hd, hd, 0.36, 0.58, color, 0.06);
      rb(hw - 0.14, hw, -hd, hd, 0.36, 0.58, color, 0.06);
      legs(0.08, 0.16, paint(WOOD));
      break;
    case 'coffee-table':
    case 'dining-table':
    case 'desk':
      rb(-hw, hw, -hd, hd, h - 0.04, h, color, 0.02);
      legs(0.07, h - 0.04, paint(it.kind === 'desk' ? DARK : WOOD), 0.022);
      if (it.kind === 'desk') rb(-0.25, 0.25, -hd + 0.06, -hd + 0.09, h, h + 0.3, paint(DARK), 0.005);
      break;
    case 'tv-unit':
      rb(-hw, hw, -hd, hd, 0.08, h, color, 0.015);
      rb(-hw * 0.72, hw * 0.72, -0.02, 0.02, h + 0.06, h + 0.72, paint('#111417'), 0.01);
      break;
    case 'bookcase':
      rb(-hw, hw, -hd, hd, 0, h, tone(base2, 0.72), 0.005);
      for (let i = 0; i < 5; i++) {
        const y = 0.06 + (i * (h - 0.12)) / 5;
        rb(-hw + 0.03, hw - 0.03, -hd - 0.012, hd * 0.2, y, y + 0.025, color, 0.003);
        for (let j = 0; j < 7; j++) {
          const x = -hw + 0.08 + (j * (w - 0.16)) / 7;
          rb(x, x + 0.09, -hd - 0.006, hd * 0.1, y + 0.03, y + 0.2 + (j % 3) * 0.03, paint(['#7f9fdc', '#e9806e', '#f3c969', '#5fbfa8', '#a98ccf'][(i + j) % 5]), 0.004);
        }
      }
      break;
    case 'chair':
      rb(-hw, hw, -hd, hd, 0.42, 0.46, color, 0.04);
      rb(-hw, hw, hd - 0.04, hd, 0.46, h, color, 0.015);
      legs(0.04, 0.42, paint(WOOD), 0.014);
      break;
    case 'office-chair':
      cyl(0, 0, 0.26, 0.03, 0.07, paint(DARK));
      cyl(0, 0, 0.025, 0.07, 0.42, paint(METAL));
      rb(-0.24, 0.24, -0.24, 0.22, 0.42, 0.5, color, 0.08);
      rb(-0.22, 0.22, 0.2, 0.25, 0.55, h, color, 0.05);
      break;
    case 'plant':
      cyl(0, 0, hw * 0.55, 0, 0.32, paint('#c47a55'));
      cyl(0, 0, 0.02, 0.32, h * 0.65, paint('#5a4632'));
      for (let i = 0; i < 9; i++) {
        const a = i * 2.4;
        const r = hw * (0.25 + (i % 3) * 0.24);
        cyl(Math.cos(a) * r, Math.sin(a) * r, hw * 0.42, h * (0.48 + (i % 4) * 0.13), h * (0.6 + (i % 4) * 0.13), i % 2 ? color : tone(base2, 1.2));
      }
      break;
    case 'rug':
      rb(-hw, hw, -hd, hd, 0.002, 0.012, color, 0.01);
      break;
    case 'counter':
    case 'sink-unit':
    case 'hob':
    case 'island': {
      const top = paint(it.kind === 'island' ? '#ece9e1' : '#6a6e70');
      rb(-hw + 0.01, hw - 0.01, -hd + 0.03, hd, 0.1, h - 0.04, color, 0.004);
      rb(-hw, hw, -hd, hd, h - 0.04, h, top, 0.006);
      const fronts = Math.max(1, Math.round(w / 0.6));
      if (it.kind !== 'island') for (let i = 1; i < fronts; i++) rb(-hw + (i * w) / fronts - 0.003, -hw + (i * w) / fronts + 0.003, hd - 0.002, hd + 0.002, 0.12, h - 0.06, tone(base2, 0.72), 0.001);
      if (it.kind === 'sink-unit') {
        rb(-hw * 0.5, hw * 0.1, -hd + 0.1, hd - 0.1, h - 0.005, h + 0.001, paint(METAL), 0.03);
        cyl(hw * 0.25, -hd + 0.08, 0.014, h, h + 0.28, paint(METAL));
      }
      if (it.kind === 'hob') {
        rb(-hw + 0.03, hw - 0.03, -hd + 0.04, hd - 0.04, h, h + 0.006, paint('#111417'), 0.01);
        rb(-hw + 0.04, hw - 0.04, hd - 0.002, hd + 0.004, 0.2, 0.74, paint('#17191c'), 0.003);
      }
      break;
    }
    case 'fridge':
    case 'washer':
      rb(-hw, hw, -hd, hd, 0.02, h, color, 0.03);
      if (it.kind === 'fridge') {
        rb(-hw + 0.01, hw - 0.01, hd - 0.005, hd + 0.003, h * 0.64, h * 0.645, tone(base2, 0.7), 0.001);
        rb(hw - 0.08, hw - 0.055, hd, hd + 0.03, h * 0.45, h * 0.6, paint(METAL), 0.006);
      } else cyl(0, hd, 0.18, 0.28, 0.285, paint(DARK));
      break;
    case 'bed-double':
    case 'bed-single': {
      rb(-hw, hw, -hd, hd, 0.08, 0.28, paint(WOOD), 0.03);
      rb(-hw - 0.02, hw + 0.02, -hd - 0.05, -hd, 0, 1, paint(WOOD), 0.012);
      rb(-hw + 0.02, hw - 0.02, -hd + 0.02, hd - 0.02, 0.28, h - 0.04, paint(WHITE), 0.06);
      rb(-hw - 0.005, hw + 0.005, -hd + 0.55, hd, 0.3, h, color, 0.05);
      const n = it.kind === 'bed-double' ? 2 : 1;
      for (let i = 0; i < n; i++) {
        const pw = (w - 0.2) / n;
        const x = -hw + 0.1 + i * pw;
        rb(x + 0.02, x + pw - 0.02, -hd + 0.08, -hd + 0.46, h - 0.04, h + 0.08, paint('#fbfbf7'), 0.08);
      }
      break;
    }
    case 'wardrobe':
    case 'dresser':
    case 'nightstand': {
      rb(-hw, hw, -hd, hd, 0.04, h, color, 0.008);
      const n = it.kind === 'wardrobe' ? Math.max(2, Math.round(w / 0.6)) : 1;
      for (let i = 1; i < n; i++) rb(-hw + (i * w) / n - 0.003, -hw + (i * w) / n + 0.003, hd - 0.001, hd + 0.003, 0.06, h - 0.02, tone(base2, 0.65), 0.001);
      if (it.kind !== 'wardrobe') for (const k of [0.35, 0.7]) rb(-hw + 0.02, hw - 0.02, hd - 0.001, hd + 0.003, h * k, h * k + 0.006, tone(base2, 0.65), 0.001);
      break;
    }
    case 'basin':
    case 'handbasin':
      if (it.kind === 'basin') rb(-hw, hw, -hd, hd, 0.25, h - 0.05, paint(WOOD), 0.01);
      else cyl(0, -hd + 0.05, 0.03, 0.2, h - 0.05, paint(METAL));
      rb(-hw, hw, -hd, hd, h - 0.05, h, color, 0.03);
      m.prism(circlePts(0, 0.03, Math.min(hw, hd) * 0.68).map(local), base + h, base + h + 0.002, paint('#d9dcdc'), null);
      cyl(0, -hd + 0.05, 0.012, h, h + 0.17, paint(METAL));
      break;
    case 'toilet':
      rb(-hw + 0.02, hw - 0.02, -hd, -hd + 0.17, 0.4, h, color, 0.03);
      cyl(0, 0.05, hw * 0.92, 0, 0.4, color);
      cyl(0, 0.05, hw * 0.98, 0.4, 0.43, paint('#ffffff'));
      break;
    case 'shower':
      rb(-hw, hw, -hd, hd, 0, 0.04, paint('#e8eaea'), 0.02);
      rb(-hw, -hw + 0.012, -hd, hd, 0.04, h, 'glass', 0.002);
      rb(-hw, hw, hd - 0.012, hd, 0.04, h, 'glass', 0.002);
      cyl(hw - 0.12, -hd + 0.05, 0.012, 0.04, 2, paint(METAL));
      cyl(hw - 0.12, -hd + 0.2, 0.1, 1.98, 2, paint(METAL));
      break;
    case 'bathtub':
      rb(-hw, hw, -hd, hd, 0, h, color, 0.1);
      rb(-hw + 0.07, hw - 0.07, -hd + 0.07, hd - 0.07, h - 0.002, h + 0.002, paint('#cfe0e4'), 0.16);
      break;
    case 'water-heater':
      cyl(0, 0, Math.min(hw, hd), 0.2, h, color);
      break;
    case 'stove':
      cyl(0, 0, Math.min(hw, hd), 0.1, h * 0.72, color);
      cyl(0, 0, 0.07, h * 0.72, h + 1, paint(DARK));
      break;
    case 'heat-pump':
      rb(-hw, hw, -hd, hd, 2.05, 2.05 + h, color, 0.05);
      break;
    case 'panel':
      rb(-hw, hw, -hd, hd, 1.05, 1.05 + h, color, 0.012);
      break;
    case 'radiator': {
      const n = Math.floor(w / 0.07);
      for (let i = 0; i < n; i++) rb(-hw + i * 0.07, -hw + i * 0.07 + 0.05, -hd, hd, 0.14, 0.14 + h, color, 0.012);
      break;
    }
    default:
      rb(-hw, hw, -hd, hd, 0, h, color, 0.02);
  }
}
