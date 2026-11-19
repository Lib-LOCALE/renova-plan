import { isGlazedKind } from '../model/catalog';
import { fmtArea, fmtLen } from '../model/estimate';
import {
  add,
  bounds,
  centroid,
  dist,
  endExtension,
  mul,
  polygonArea,
  rectPoints,
  solidSpans,
  wallDir,
  wallLength,
  wallNormal,
  wallOpenings,
  wallPolygon,
} from '../model/geometry';
import type { Dimension, Opening, Phase, Project, Pt, Wall } from '../model/types';
import { visibleIn } from '../model/types';
import { dimPoints, levelWalls } from './hit';
import { itemSymbol, stairSymbol } from './symbols';

export interface MarkupOptions {
  highlight?: string | null;
  labels?: boolean;
  furniture?: boolean;
  dims?: boolean;
  tr?: (s: string) => string;
}

const n = (v: number) => +v.toFixed(3);
const esc = (s: string) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]!);
const poly = (pts: Pt[]) => pts.map((p) => `${n(p.x)},${n(p.y)}`).join(' ');
const P = (p: Pt) => `${n(p.x)} ${n(p.y)}`;

export const PLAN_DEFS = `<defs>
<pattern id="pl-party" width=".09" height=".09" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width=".09" height=".09" fill="var(--plan-sheet)"/><rect width=".03" height=".09" fill="var(--plan-wall)"/></pattern>
<pattern id="pl-demo" width=".1" height=".1" patternUnits="userSpaceOnUse" patternTransform="rotate(-45)"><rect width=".1" height=".1" fill="var(--plan-sheet)"/><rect width=".035" height=".1" fill="var(--st-demolish)"/></pattern>
<pattern id="pl-solid" width=".07" height=".07" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width=".07" height=".07" fill="var(--plan-sheet)"/><rect width=".014" height=".07" fill="var(--plan-soft)"/></pattern>
</defs>`;

export const PLAN_CSS = `
.pl *{vector-effect:non-scaling-stroke}
.pl text{font-family:var(--font,'Hanken Grotesk Variable',system-ui,sans-serif);fill:var(--plan-ink);paint-order:stroke;stroke:var(--plan-sheet);stroke-width:3px;stroke-linejoin:round}
.pl .rm{fill-opacity:var(--room-alpha,.46)}
.pl .rm.hot{fill-opacity:.78}
.pl .rm.new{stroke:var(--st-new);stroke-width:1.5px;stroke-dasharray:6 4}
.pl .rm.demolish{fill:url(#pl-demo);fill-opacity:.32}
.pl .rn{font-size:calc(15px / var(--k));font-weight:700;text-anchor:middle}
.pl .ra{font-size:calc(12.5px / var(--k));font-variant-numeric:tabular-nums;text-anchor:middle;fill:var(--plan-soft);transform:translateY(calc(16px / var(--k)))}
.pl .wl{fill:var(--plan-wall)}
.pl .wl.party{fill:url(#pl-party);stroke:var(--plan-wall);stroke-width:1px}
.pl .wl.new{fill:var(--st-new)}
.pl .wl.demolish{fill:url(#pl-demo);stroke:var(--st-demolish);stroke-width:1.25px;stroke-dasharray:5 3}
.pl .wg{fill:var(--plan-sheet);stroke:var(--plan-glass);stroke-width:1.25px}
.pl .wg.new{stroke:var(--st-new)}.pl .wg.demolish{stroke:var(--st-demolish)}
.pl .wm{stroke:var(--plan-glass);stroke-width:1.5px;fill:none}
.pl .dr{fill:none;stroke:var(--plan-ink);stroke-width:1.1px}
.pl .dr.new{stroke:var(--st-new)}.pl .dr.demolish{stroke:var(--st-demolish)}
.pl .ds{stroke-dasharray:6 4}
.pl .tag{fill:var(--plan-glass);stroke:none}
.pl .tag.q{fill:var(--mark)}
.pl .tt{font-size:calc(10.5px / var(--k));font-weight:700;text-anchor:middle;dominant-baseline:central;fill:var(--plan-sheet);stroke:none}
.pl .fu{fill:var(--plan-item);stroke:var(--plan-ink);stroke-width:.9px}
.pl .fd{fill:none;stroke:var(--plan-soft);stroke-width:.8px}
.pl .fg.new .fu{stroke:var(--st-new);stroke-width:1.4px}
.pl .fg.demolish .fu{stroke:var(--st-demolish);stroke-dasharray:4 3}
.pl .st{fill:var(--plan-sheet);fill-opacity:.7;stroke:var(--plan-ink);stroke-width:1px}
.pl .sd{fill:none;stroke:var(--plan-soft);stroke-width:.8px}
.pl .sa{fill:none;stroke:var(--plan-ink);stroke-width:1.25px}
.pl .solid{fill:url(#pl-solid);stroke:var(--plan-wall);stroke-width:1px}
.pl .dm{fill:none;stroke:var(--plan-soft);stroke-width:.9px}
.pl .dm.f{stroke:var(--mark);stroke-width:1.2px}
.pl .dt{font-size:calc(11px / var(--k));font-variant-numeric:tabular-nums;font-weight:550;text-anchor:middle;dominant-baseline:central}
.pl .dt.f{fill:var(--mark);font-weight:700}
.pl .nt{font-size:calc(11.5px / var(--k));font-weight:550}
.pl .nt.f{fill:var(--mark)}
`;

function outwardSign(w: Wall, center: Pt): number {
  const m = add(w.a, mul(wallDir(w), wallLength(w) / 2));
  const nrm = wallNormal(w);
  return (m.x - center.x) * nrm.x + (m.y - center.y) * nrm.y >= 0 ? 1 : -1;
}

function openingMarkup(o: Opening, w: Wall, center: Pt, site: boolean): string {
  const d = wallDir(w);
  const nrm = wallNormal(w);
  const t = w.thickness / 2;
  const c = add(w.a, mul(d, o.offset));
  const s = o.flip ? -1 : 1;
  const h = o.hinge ? 1 : -1;
  const a = add(c, mul(d, -o.width / 2));
  const b = add(c, mul(d, o.width / 2));
  let out = '';
  const st = !site || o.status === 'existing' ? '' : ` ${esc(o.status)}`;
  if (isGlazedKind(o.kind)) {
    const q = [add(a, mul(nrm, t)), add(b, mul(nrm, t)), add(b, mul(nrm, -t)), add(a, mul(nrm, -t))];
    out += `<polygon class="wg${st}" points="${poly(q)}"/><path class="wm" d="M${P(a)}L${P(b)}"/>`;
    if (o.kind === 'bay-window') out += `<path class="wm" d="M${P(c)}m${n(nrm.x * t)} ${n(nrm.y * t)}L${P(add(c, mul(nrm, -t)))}"/>`;
    if (o.kind === 'french-window') {
      const r = o.width / 2;
      const p1 = add(a, mul(nrm, s * t));
      const p2 = add(p1, mul(nrm, s * r));
      const p3 = add(c, mul(nrm, s * t));
      out += `<path class="dr${st}" d="M${P(p1)}L${P(p2)}A${n(r)} ${n(r)} 0 0 ${crossSign(nrm, d, s)} ${P(p3)}"/>`;
    }
  } else if (o.kind === 'passage' || o.kind === 'garage-door') {
    out += `<path class="dr ds${st}" d="M${P(add(a, mul(nrm, t)))}L${P(add(b, mul(nrm, t)))}M${P(add(a, mul(nrm, -t)))}L${P(add(b, mul(nrm, -t)))}"/>`;
  } else if (o.kind === 'sliding-door') {
    out += `<path class="dr${st}" d="M${P(add(a, mul(nrm, t * 0.35)))}L${P(add(c, mul(d, o.width * 0.08)))}M${P(add(b, mul(nrm, -t * 0.35)))}L${P(add(c, mul(d, -o.width * 0.08)))}"/>`;
  } else {
    const leaves = o.kind === 'double-door' ? [[a, 1], [b, -1]] as [Pt, number][] : [[h < 0 ? a : b, -h]] as [Pt, number][];
    const r = o.kind === 'double-door' ? o.width / 2 : o.width;
    for (const [hinge, dir] of leaves) {
      const p1 = add(hinge, mul(nrm, s * t));
      const p2 = add(p1, mul(nrm, s * r));
      const p3 = add(add(hinge, mul(d, dir * r)), mul(nrm, s * t));
      out += `<path class="dr${st}" d="M${P(p1)}L${P(p2)}A${n(r)} ${n(r)} 0 0 ${crossSign(nrm, d, s * dir)} ${P(p3)}"/>`;
    }
  }
  if (o.tag) {
    const side = outwardSign(w, center);
    const q = add(c, mul(nrm, side * (t + 0.26)));
    out += `<circle class="tag${o.tag === '?' ? ' q' : ''}" cx="${n(q.x)}" cy="${n(q.y)}" r=".13" style="r:calc(9px / var(--k))"/><text class="tt" x="${n(q.x)}" y="${n(q.y)}">${esc(o.tag)}</text>`;
  }
  return `<g data-id="${esc(o.id)}">${out}</g>`;
}

const crossSign = (nrm: Pt, d: Pt, s: number) => ((nrm.x * d.y - nrm.y * d.x) * s > 0 ? 1 : 0);

export function dimMarkup(d: Dimension, unit: 'm' | 'cm'): string {
  const { a, b } = dimPoints(d);
  const nrm = wallNormal(d);
  const dir = wallDir(d);
  const tick = 0.07;
  const tk = (p: Pt) => `M${P(add(add(p, mul(dir, -tick)), mul(nrm, tick)))}L${P(add(add(p, mul(dir, tick)), mul(nrm, -tick)))}`;
  const label = d.label ?? fmtLen(dist(d.a, d.b), unit);
  const m = { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
  let deg = (Math.atan2(dir.y, dir.x) * 180) / Math.PI;
  if (deg > 90 || deg <= -90) deg += 180;
  const f = d.flag ? ' f' : '';
  const ext = Math.abs(d.offset) > 0.02 ? `M${P(d.a)}L${P(a)}M${P(d.b)}L${P(b)}` : '';
  return `<g data-id="${esc(d.id)}"><path class="dm${f}" d="M${P(a)}L${P(b)}${tk(a)}${tk(b)}${ext}"/><text class="dt${f}" transform="translate(${P(m)}) rotate(${n(deg)})">${esc(label)}</text></g>`;
}

/** Contenu vectoriel du plan : partagé par l'éditeur, les exports et le dossier imprimé. */
export function planMarkup(p: Project, level: string, phase: Phase, opts: MarkupOptions): string {
  const tr = opts.tr ?? ((s: string) => s);
  // Les couleurs de chantier ne servent qu'en phase Chantier : Avant et Après restent de vrais plans.
  const site = phase === 'site';
  const mark = (status: string) => (site && status !== 'existing' ? ` ${esc(status)}` : '');
  const walls = levelWalls(p, level, phase);
  const pts = walls.flatMap((w) => [w.a, w.b]);
  const center = pts.length ? { x: bounds(pts).minX + bounds(pts).w / 2, y: bounds(pts).minY + bounds(pts).h / 2 } : { x: 0, y: 0 };
  let s = '';
  for (const u of p.underlays) {
    if (u.level === level && u.visible) s += `<image href="${esc(u.src)}" x="${n(u.x)}" y="${n(u.y)}" width="${n(u.width)}" height="${n(u.width / u.aspect)}" opacity="${n(u.opacity)}" preserveAspectRatio="none"/>`;
  }
  const rooms = p.rooms.filter((r) => r.level === level && visibleIn(r.status, phase));
  for (const r of rooms) {
    const cls = `rm${mark(r.status)}${opts.highlight === r.id ? ' hot' : ''}`;
    s += `<polygon data-id="${esc(r.id)}" class="${cls}" fill="${esc(r.color)}" points="${poly(r.points)}"/>`;
  }
  for (const st of p.stairs) {
    if (st.level === level && st.view !== '3d' && visibleIn(st.status, phase)) s += `<g data-id="${esc(st.id)}" transform="translate(${n(st.x)} ${n(st.y)}) rotate(${n(st.rotation)})">${stairSymbol(st.w, st.d, st.shape, st.steps, st.flip)}</g>`;
  }
  if (opts.furniture !== false) {
    for (const i of p.items) {
      if (i.level === level && visibleIn(i.status, phase)) s += `<g data-id="${esc(i.id)}" class="fg${mark(i.status)}" transform="translate(${n(i.x)} ${n(i.y)}) rotate(${n(i.rotation)})">${itemSymbol(i.kind, i.w, i.d)}</g>`;
    }
  }
  for (const w of walls) {
    const len = wallLength(w);
    const ops = wallOpenings(p, w.id, phase);
    const spans = solidSpans(len, ops);
    const extA = endExtension(w.a, w, walls);
    const extB = endExtension(w.b, w, walls);
    const cls = `wl${mark(w.status)}${w.kind === 'party' && (!site || w.status === 'existing') ? ' party' : ''}`;
    for (const [from, to] of spans) {
      s += `<polygon data-id="${esc(w.id)}" class="${cls}" points="${poly(wallPolygon(w, from, to, from < 1e-3 ? extA : 0, to > len - 1e-3 ? extB : 0))}"/>`;
    }
  }
  for (const w of walls) for (const o of wallOpenings(p, w.id, phase)) s += openingMarkup(o, w, center, site);
  if (opts.labels !== false) {
    for (const r of rooms) {
      // Au chantier, une pièce supprimée laisse la parole à celle qui la remplace.
      if (phase === 'site' && r.status === 'demolish') continue;
      const c = centroid(r.points);
      s += `<text class="rn" x="${n(c.x)}" y="${n(c.y)}">${esc(tr(r.name))}</text><text class="ra" x="${n(c.x)}" y="${n(c.y)}">${fmtArea(polygonArea(r.points))}</text>`;
    }
  }
  if (opts.dims !== false) {
    for (const d of p.dims) {
      if (d.level !== level) continue;
      if (d.only && phase !== 'site' && d.only !== phase) continue;
      s += dimMarkup(d, p.dimUnit);
    }
  }
  for (const note of p.notes) {
    if (note.level === level) s += `<text data-id="${esc(note.id)}" class="nt${note.flag ? ' f' : ''}" x="${n(note.x)}" y="${n(note.y)}">${esc(note.text)}</text>`;
  }
  return s;
}

/** Emprise dessinée d'un niveau, cotes comprises. */
export function planBounds(p: Project, level: string, phase: Phase) {
  const pts: Pt[] = [];
  for (const w of levelWalls(p, level, phase)) pts.push(w.a, w.b);
  for (const r of p.rooms) if (r.level === level) pts.push(...r.points);
  for (const i of p.items) if (i.level === level) pts.push({ x: i.x, y: i.y });
  for (const d of p.dims) if (d.level === level) pts.push(...Object.values(dimPoints(d)));
  return pts.length ? bounds(pts) : bounds(rectPoints(0, 0, 8, 6));
}

const EXPORT_VARS = `--plan-sheet:#fff;--plan-ink:#1b2127;--plan-soft:#5d6871;--plan-wall:#1f272e;--plan-glass:#1f7f9c;--plan-item:#fbfbf9;--st-new:#157a55;--st-demolish:#c4322b;--mark:#c9480f;--font:'Hanken Grotesk',Arial,sans-serif`;

export function standaloneSvg(p: Project, level: string, phase: Phase, scale = 70): string {
  const b = planBounds(p, level, phase);
  const m = 0.9;
  const x = b.minX - m;
  const y = b.minY - m;
  const w = b.w + 2 * m;
  const h = b.h + 2 * m;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${n(x)} ${n(y)} ${n(w)} ${n(h)}" width="${Math.round(w * scale)}" height="${Math.round(h * scale)}" class="pl" style="--k:${scale};${EXPORT_VARS}"><style>${PLAN_CSS}</style>${PLAN_DEFS}<rect x="${n(x)}" y="${n(y)}" width="${n(w)}" height="${n(h)}" fill="#fff"/>${planMarkup(p, level, phase, {})}</svg>`;
}
