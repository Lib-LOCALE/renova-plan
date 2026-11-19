import * as THREE from 'three';
import type { Pt } from '../model/types';

export interface Faces {
  top?: string;
  bottom?: string;
  front?: string;
  back?: string;
  ends?: string;
}

interface Bucket {
  pos: number[];
  nor: number[];
  uv: number[];
  idx: number[];
  owners: (string | undefined)[];
}

type V3 = [number, number, number];

const crossN = (a: V3, b: V3, c: V3): V3 => {
  const ux = b[0] - a[0];
  const uy = b[1] - a[1];
  const uz = b[2] - a[2];
  const vx = c[0] - a[0];
  const vy = c[1] - a[1];
  const vz = c[2] - a[2];
  const x = uy * vz - uz * vy;
  const y = uz * vx - ux * vz;
  const z = ux * vy - uy * vx;
  const l = Math.hypot(x, y, z) || 1;
  return [x / l, y / l, z / l];
};

/** Regroupe toute la géométrie par matériau : peu d'appels de rendu, sélection par triangle. */
export class Mesher {
  buckets = new Map<string, Bucket>();
  owner: string | undefined;

  private bucket(key: string): Bucket {
    let b = this.buckets.get(key);
    if (!b) {
      b = { pos: [], nor: [], uv: [], idx: [], owners: [] };
      this.buckets.set(key, b);
    }
    return b;
  }

  quad(key: string, a: V3, b: V3, c: V3, d: V3, uv: [number, number][]) {
    const m = this.bucket(key);
    const n = crossN(a, b, c);
    const i = m.pos.length / 3;
    for (const v of [a, b, c, d]) {
      m.pos.push(...v);
      m.nor.push(...n);
    }
    for (const t of uv) m.uv.push(...t);
    m.idx.push(i, i + 1, i + 2, i, i + 2, i + 3);
    m.owners.push(this.owner, this.owner);
  }

  /** Boîte orientée : x le long de dir, y vertical, z selon la normale. */
  box(origin: Pt, base: number, dir: Pt, x0: number, x1: number, y0: number, y1: number, z0: number, z1: number, f: Faces | string) {
    if (x1 - x0 < 1e-5 || y1 - y0 < 1e-5 || z1 - z0 < 1e-5) return;
    const k: Faces = typeof f === 'string' ? { top: f, bottom: f, front: f, back: f, ends: f } : f;
    const nx = -dir.y;
    const nz = dir.x;
    const p = (x: number, y: number, z: number): V3 => [origin.x + dir.x * x + nx * z, base + y, origin.y + dir.y * x + nz * z];
    if (k.front) this.quad(k.front, p(x0, y0, z1), p(x1, y0, z1), p(x1, y1, z1), p(x0, y1, z1), [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]);
    if (k.back) this.quad(k.back, p(x1, y0, z0), p(x0, y0, z0), p(x0, y1, z0), p(x1, y1, z0), [[x1, y0], [x0, y0], [x0, y1], [x1, y1]]);
    if (k.top) this.quad(k.top, p(x0, y1, z1), p(x1, y1, z1), p(x1, y1, z0), p(x0, y1, z0), [[x0, z1], [x1, z1], [x1, z0], [x0, z0]]);
    if (k.bottom) this.quad(k.bottom, p(x0, y0, z0), p(x1, y0, z0), p(x1, y0, z1), p(x0, y0, z1), [[x0, z0], [x1, z0], [x1, z1], [x0, z1]]);
    if (k.ends) {
      this.quad(k.ends, p(x1, y0, z1), p(x1, y0, z0), p(x1, y1, z0), p(x1, y1, z1), [[z1, y0], [z0, y0], [z0, y1], [z1, y1]]);
      this.quad(k.ends, p(x0, y0, z0), p(x0, y0, z1), p(x0, y1, z1), p(x0, y1, z0), [[z0, y0], [z1, y0], [z1, y1], [z0, y1]]);
    }
  }

  /** Prisme vertical à partir d'un contour de plan. */
  prism(points: Pt[], y0: number, y1: number, top: string, side: string | null = top, bottom: string | null = null) {
    let area = 0;
    points.forEach((p, i) => {
      const q = points[(i + 1) % points.length];
      area += p.x * q.y - q.x * p.y;
    });
    const pts = area > 0 ? points : [...points].reverse();
    const contour = pts.map((p) => new THREE.Vector2(p.x, p.y));
    const tris = THREE.ShapeUtils.triangulateShape(contour, []);
    const m = this.bucket(top);
    const base = m.pos.length / 3;
    for (const p of pts) {
      m.pos.push(p.x, y1, p.y);
      m.nor.push(0, 1, 0);
      m.uv.push(p.x, p.y);
    }
    for (const [a, b, c] of tris) {
      m.idx.push(base + a, base + c, base + b);
      m.owners.push(this.owner);
    }
    if (bottom) {
      const b2 = this.bucket(bottom);
      const start = b2.pos.length / 3;
      for (const p of pts) {
        b2.pos.push(p.x, y0, p.y);
        b2.nor.push(0, -1, 0);
        b2.uv.push(p.x, p.y);
      }
      for (const [a, b, c] of tris) {
        b2.idx.push(start + a, start + b, start + c);
        b2.owners.push(this.owner);
      }
    }
    if (side && y1 > y0) {
      pts.forEach((p, i) => {
        const q = pts[(i + 1) % pts.length];
        const l = Math.hypot(q.x - p.x, q.y - p.y);
        this.quad(side, [p.x, y0, p.y], [p.x, y1, p.y], [q.x, y1, q.y], [q.x, y0, q.y], [[0, y0], [0, y1], [l, y1], [l, y0]]);
      });
    }
  }

  triangle(key: string, a: V3, b: V3, c: V3) {
    const m = this.bucket(key);
    const n = crossN(a, b, c);
    const i = m.pos.length / 3;
    for (const v of [a, b, c]) {
      m.pos.push(...v);
      m.nor.push(...n);
      m.uv.push(v[0] + v[2], v[1]);
    }
    m.idx.push(i, i + 1, i + 2);
    m.owners.push(this.owner);
  }

  build(material: (key: string) => THREE.Material): THREE.Group {
    const g = new THREE.Group();
    for (const [key, b] of this.buckets) {
      if (!b.idx.length) continue;
      const geo = new THREE.BufferGeometry();
      geo.setAttribute('position', new THREE.Float32BufferAttribute(b.pos, 3));
      geo.setAttribute('normal', new THREE.Float32BufferAttribute(b.nor, 3));
      geo.setAttribute('uv', new THREE.Float32BufferAttribute(b.uv, 2));
      geo.setIndex(b.idx);
      const mat = material(key);
      const mesh = new THREE.Mesh(geo, mat);
      mesh.castShadow = !mat.transparent;
      mesh.receiveShadow = !mat.transparent;
      mesh.userData.owners = b.owners;
      g.add(mesh);
    }
    return g;
  }
}

export function roundedRect(x0: number, x1: number, z0: number, z1: number, r: number, seg = 3): Pt[] {
  const R = Math.min(r, (x1 - x0) / 2, (z1 - z0) / 2);
  const out: Pt[] = [];
  const corners: [number, number, number][] = [[x1 - R, z0 + R, -Math.PI / 2], [x1 - R, z1 - R, 0], [x0 + R, z1 - R, Math.PI / 2], [x0 + R, z0 + R, Math.PI]];
  for (const [cx, cz, a] of corners) {
    for (let i = 0; i <= seg; i++) {
      const t = a + (Math.PI / 2) * (i / seg);
      out.push({ x: cx + Math.cos(t) * R, y: cz + Math.sin(t) * R });
    }
  }
  return out;
}

export const circlePts = (cx: number, cz: number, r: number, n = 18): Pt[] =>
  Array.from({ length: n }, (_, i) => ({ x: cx + Math.cos((i / n) * Math.PI * 2) * r, y: cz + Math.sin((i / n) * Math.PI * 2) * r }));
