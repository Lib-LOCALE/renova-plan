import * as THREE from 'three';

type Paint = (g: CanvasRenderingContext2D, s: number, r: () => number) => void;

function seeded(seed: number) {
  let v = seed;
  return () => {
    v = (v * 16807) % 2147483647;
    return (v - 1) / 2147483646;
  };
}

function speckle(g: CanvasRenderingContext2D, s: number, r: () => number, count: number, alpha: number, size = 2) {
  for (let i = 0; i < count; i++) {
    const v = r() > 0.5 ? 255 : 0;
    g.fillStyle = `rgba(${v},${v},${v},${alpha * r()})`;
    g.fillRect(r() * s, r() * s, size * r() + 0.5, size * r() + 0.5);
  }
}

const shade = (base: [number, number, number], k: number) => `rgb(${base.map((c) => Math.round(Math.min(255, c * k))).join(',')})`;

const PAINTERS: Record<string, { size: number; paint: Paint }> = {
  oak: {
    size: 1.6,
    paint(g, s, r) {
      const rows = 8;
      const h = s / rows;
      for (let y = 0; y < rows; y++) {
        let x = -r() * s * 0.5;
        while (x < s) {
          const w = s * (0.45 + r() * 0.35);
          g.fillStyle = shade([190, 147, 100], 0.9 + r() * 0.2);
          g.fillRect(x, y * h, w, h);
          g.globalAlpha = 0.12;
          g.strokeStyle = '#5b3c20';
          for (let i = 0; i < 7; i++) {
            const yy = y * h + r() * h;
            g.beginPath();
            g.moveTo(x, yy);
            g.bezierCurveTo(x + w * 0.3, yy + r() * 5 - 2, x + w * 0.6, yy + r() * 5 - 2, x + w, yy);
            g.stroke();
          }
          g.globalAlpha = 0.45;
          g.fillStyle = '#4a321d';
          g.fillRect(x, y * h, 1.2, h);
          g.globalAlpha = 1;
          x += w;
        }
        g.fillStyle = 'rgba(60,38,20,.45)';
        g.fillRect(0, y * h, s, 1.2);
      }
    },
  },
  tile: { size: 1.2, paint: (g, s, r) => tiles(g, s, r, 2, [221, 218, 209], '#b9b5aa') },
  slate: { size: 1.2, paint: (g, s, r) => tiles(g, s, r, 2, [88, 98, 106], '#3d454c') },
  terracotta: { size: 0.9, paint: (g, s, r) => tiles(g, s, r, 3, [190, 110, 76], '#8f573f') },
  concrete: {
    size: 2.4,
    paint(g, s, r) {
      g.fillStyle = '#aaacab';
      g.fillRect(0, 0, s, s);
      for (let i = 0; i < 26; i++) {
        const x = r() * s;
        const y = r() * s;
        const c = g.createRadialGradient(x, y, 0, x, y, 40 + r() * 90);
        c.addColorStop(0, `rgba(${r() > 0.5 ? '255,255,255' : '60,60,60'},.07)`);
        c.addColorStop(1, 'rgba(0,0,0,0)');
        g.fillStyle = c;
        g.fillRect(0, 0, s, s);
      }
      speckle(g, s, r, 2500, 0.06);
    },
  },
  carpet: {
    size: 0.7,
    paint(g, s, r) {
      g.fillStyle = '#8c93a3';
      g.fillRect(0, 0, s, s);
      speckle(g, s, r, 16000, 0.12, 3);
    },
  },
  vinyl: { size: 1.4, paint: (g, s, r) => tiles(g, s, r, 4, [185, 176, 156], '#a59d89') },
  plaster: {
    size: 2,
    paint(g, s, r) {
      g.fillStyle = '#f3f1eb';
      g.fillRect(0, 0, s, s);
      speckle(g, s, r, 1800, 0.018, 3);
    },
  },
  facade: {
    size: 1.2,
    paint(g, s, r) {
      g.fillStyle = '#e4dccd';
      g.fillRect(0, 0, s, s);
      speckle(g, s, r, 11000, 0.08, 2.6);
    },
  },
  party: {
    size: 1,
    paint(g, s, r) {
      g.fillStyle = '#c9c3b6';
      g.fillRect(0, 0, s, s);
      g.strokeStyle = 'rgba(70,60,50,.18)';
      for (let y = 0; y <= 4; y++) {
        g.beginPath();
        g.moveTo(0, (y * s) / 4);
        g.lineTo(s, (y * s) / 4);
        g.stroke();
        for (let x = 0; x <= 2; x++) {
          const px = ((x + (y % 2) / 2) * s) / 2;
          g.beginPath();
          g.moveTo(px, (y * s) / 4);
          g.lineTo(px, ((y + 1) * s) / 4);
          g.stroke();
        }
      }
      speckle(g, s, r, 1200, 0.05);
    },
  },
  roof: {
    size: 0.8,
    paint(g, s, r) {
      const rows = 4;
      const cols = 4;
      for (let y = 0; y < rows; y++) {
        for (let x = -1; x <= cols; x++) {
          const w = s / cols;
          const h = s / rows;
          g.fillStyle = shade([166, 86, 58], 0.88 + r() * 0.22);
          g.fillRect(x * w + ((y % 2) * w) / 2, y * h, w, h);
          g.fillStyle = 'rgba(40,18,10,.35)';
          g.fillRect(x * w + ((y % 2) * w) / 2, y * h, 1.5, h);
        }
        g.fillStyle = 'rgba(30,14,8,.45)';
        g.fillRect(0, y * (s / rows), s, 3);
      }
    },
  },
  grass: {
    size: 3,
    paint(g, s, r) {
      g.fillStyle = '#8fa977';
      g.fillRect(0, 0, s, s);
      for (let i = 0; i < 9000; i++) {
        g.strokeStyle = `rgba(${40 + r() * 70},${90 + r() * 70},${40 + r() * 40},.28)`;
        const x = r() * s;
        const y = r() * s;
        g.beginPath();
        g.moveTo(x, y);
        g.lineTo(x + r() * 4 - 2, y - 2 - r() * 5);
        g.stroke();
      }
    },
  },
};

function tiles(g: CanvasRenderingContext2D, s: number, r: () => number, n: number, base: [number, number, number], joint: string) {
  g.fillStyle = joint;
  g.fillRect(0, 0, s, s);
  const c = s / n;
  for (let y = 0; y < n; y++) {
    for (let x = 0; x < n; x++) {
      g.fillStyle = shade(base, 0.95 + r() * 0.08);
      g.fillRect(x * c + 2, y * c + 2, c - 4, c - 4);
    }
  }
  speckle(g, s, r, 900, 0.035);
}

const cache = new Map<string, THREE.CanvasTexture>();

export function surfaceTexture(name: string): THREE.Texture {
  const hit = cache.get(name);
  if (hit) return hit;
  const def = PAINTERS[name];
  const c = document.createElement('canvas');
  c.width = c.height = 512;
  const g = c.getContext('2d')!;
  def.paint(g, 512, seeded(name.length * 7919 + 13));
  const t = new THREE.CanvasTexture(c);
  t.wrapS = t.wrapT = THREE.RepeatWrapping;
  t.repeat.set(1 / def.size, 1 / def.size);
  t.colorSpace = THREE.SRGBColorSpace;
  t.anisotropy = 8;
  cache.set(name, t);
  return t;
}
