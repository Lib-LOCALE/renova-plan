/** Symboles de mobilier en vue de dessus, dessinés dans un repère centré, en mètres. */
const n = (v: number) => +v.toFixed(3);
const rect = (x: number, y: number, w: number, h: number, c = 'fd', r = 0) =>
  `<rect class="${c}" x="${n(x)}" y="${n(y)}" width="${n(w)}" height="${n(h)}"${r ? ` rx="${n(r)}"` : ''}/>`;
const circle = (x: number, y: number, r: number, c = 'fd') => `<circle class="${c}" cx="${n(x)}" cy="${n(y)}" r="${n(r)}"/>`;
const line = (a: number, b: number, c: number, d: number) => `<path class="fd" d="M${n(a)} ${n(b)}L${n(c)} ${n(d)}"/>`;

export function itemSymbol(kind: string, w: number, d: number): string {
  const x = -w / 2;
  const y = -d / 2;
  const base = (r = 0.02) => rect(x, y, w, d, 'fu', r);
  switch (kind) {
    case 'sofa':
      return base(0.08) + rect(x + 0.04, y + d * 0.68, w - 0.08, d * 0.28, 'fd', 0.05) + rect(x + 0.04, y + 0.04, 0.16, d * 0.62, 'fd', 0.04) + rect(-x - 0.2, y + 0.04, 0.16, d * 0.62, 'fd', 0.04) + line(0, y + 0.05, 0, y + d * 0.66);
    case 'armchair':
      return base(0.08) + rect(x + 0.05, y + d * 0.68, w - 0.1, d * 0.26, 'fd', 0.04) + rect(x + 0.05, y + 0.05, w - 0.1, d * 0.58, 'fd', 0.05);
    case 'dining-table':
      return base(0.03) + [-0.3, 0.3].flatMap((k) => [circle(w * k, y - 0.14, 0.15), circle(w * k, -y + 0.14, 0.15)]).join('');
    case 'chair':
    case 'office-chair':
      return circle(0, 0, Math.min(w, d) * 0.46, 'fu') + line(x + 0.07, -y - 0.03, -x - 0.07, -y - 0.03);
    case 'plant':
      return circle(0, 0, w / 2, 'fu') + circle(0, 0, w * 0.2) + line(-w * 0.4, 0, w * 0.4, 0) + line(0, -w * 0.4, 0, w * 0.4);
    case 'rug':
      return base() + rect(x + 0.08, y + 0.08, w - 0.16, d - 0.16);
    case 'bookcase':
    case 'tv-unit':
    case 'wardrobe':
    case 'dresser':
      return base() + line(x, 0, -x, 0) + (kind === 'wardrobe' ? line(0, y, 0, -y) : '');
    case 'counter':
      return base() + line(x, -y - 0.06, -x, -y - 0.06);
    case 'island':
      return base(0.03) + rect(x + 0.08, y + 0.08, w - 0.16, d - 0.16);
    case 'fridge':
    case 'washer':
      return base() + line(x, y, -x, -y) + line(-x, y, x, -y);
    case 'hob':
      return base() + [-1, 1].flatMap((a) => [-1, 1].map((b) => circle((a * w) / 4.6, (b * d) / 4.6, Math.min(w, d) * 0.13))).join('');
    case 'sink-unit':
      return base() + rect(x + w * 0.16, y + 0.1, w * 0.38, d - 0.2, 'fd', 0.04) + circle(x + w * 0.35, 0, 0.025);
    case 'bed-double':
    case 'bed-single': {
      const pillows = kind === 'bed-double' ? [x + 0.08, 0.04] : [x + 0.08];
      const pw = kind === 'bed-double' ? w / 2 - 0.12 : w - 0.16;
      return base(0.03) + line(x, y + 0.52, -x, y + 0.52) + pillows.map((px) => rect(px, y + 0.08, pw, 0.36, 'fd', 0.05)).join('');
    }
    case 'nightstand':
      return base() + circle(0, 0, Math.min(w, d) * 0.22);
    case 'basin':
    case 'handbasin':
      return base(0.02) + `<ellipse class="fd" cx="0" cy="${n(d * 0.06)}" rx="${n(w * 0.3)}" ry="${n(d * 0.3)}"/>`;
    case 'toilet':
      return rect(x + 0.03, y, w - 0.06, d * 0.26, 'fu', 0.03) + `<ellipse class="fu" cx="0" cy="${n(d * 0.14)}" rx="${n(w * 0.46)}" ry="${n(d * 0.34)}"/>`;
    case 'shower':
      return base() + circle(0, 0, 0.05) + line(x, y, -x, -y) + line(-x, y, x, -y);
    case 'bathtub':
      return base(0.08) + rect(x + 0.08, y + 0.08, w - 0.16, d - 0.16, 'fd', 0.18) + circle(x + 0.24, 0, 0.03);
    case 'desk':
      return base() + rect(x + w * 0.34, y + 0.06, w * 0.32, 0.08);
    case 'water-heater':
    case 'stove':
      return circle(0, 0, Math.min(w, d) / 2, 'fu') + circle(0, 0, Math.min(w, d) * 0.26);
    case 'outdoor-unit':
      return base() + circle(0, 0, d * 0.36);
    default:
      return base() + line(x + 0.04, 0, -x - 0.04, 0);
  }
}

export function stairSymbol(w: number, d: number, shape: string, steps: number, flip = false): string {
  const x = -w / 2;
  const y = -d / 2;
  let out = rect(x, y, w, d, 'st');
  let path = '';
  const s = flip ? -1 : 1;
  const tread = (a: number, b: number, c: number, e: number) => `M${n(a)} ${n(b)}L${n(c)} ${n(e)}`;
  if (shape === 'straight') {
    for (let i = 1; i < steps; i++) path += tread(x + (w * i) / steps, y, x + (w * i) / steps, -y);
    out += `<path class="sd" d="${path}"/><path class="sa" d="M${n(x + 0.1)} 0H${n(-x - 0.1)}m-.12 -.07l.12 .07l-.12 .07"/>`;
    return out;
  }
  if (shape === 'quarter') {
    const run = w - d;
    const straight = Math.max(2, steps - 4);
    for (let i = 1; i <= straight; i++) path += tread(x + (run * i) / straight, y, x + (run * i) / straight, -y);
    const cx = x + run;
    const cy = s * -y;
    for (let i = 1; i < 4; i++) {
      const a = (Math.PI / 2) * (i / 4);
      const r = d / Math.max(Math.cos(a), Math.sin(a));
      path += tread(cx, cy, cx + Math.sin(a) * r, cy - s * Math.cos(a) * r);
    }
    out += `<path class="sd" d="${path}"/><path class="sa" d="M${n(x + 0.1)} 0H${n(cx + d / 2)}V${n(s * y)}"/>`;
    return out;
  }
  const lane = d / 2;
  const run = w - lane;
  const per = Math.max(2, Math.floor((steps - 3) / 2));
  for (let i = 1; i <= per; i++) {
    const px = x + (run * i) / per;
    path += tread(px, 0, px, -y) + tread(px, y, px, 0);
  }
  const cx = x + run;
  for (let i = 1; i < 4; i++) {
    const a = -Math.PI / 2 + (Math.PI * i) / 4;
    const r = Math.min(lane / Math.max(Math.abs(Math.cos(a)), 1e-3), d / 2 / Math.max(Math.abs(Math.sin(a)), 1e-3));
    path += tread(cx, 0, cx + Math.cos(a) * r, Math.sin(a) * r);
  }
  out += `<path class="sd" d="${path}"/><path class="sd" d="M${n(x)} 0H${n(cx)}"/><path class="sa" d="M${n(x + 0.1)} ${n(s * lane / 2)}H${n(cx + lane / 2)}V${n((-s * lane) / 2)}H${n(x + 0.1)}"/>`;
  return out;
}
