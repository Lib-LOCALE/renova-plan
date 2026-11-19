import { untrack } from 'svelte';
import { add, boxCorners, detectRoom, dist, mul, rotatePoint, sub, wallDir, wallNormal, wallPolygon, wallLength } from '../model/geometry';
import { addBoxedRoom, addWall, moveRef, moveRoomEdge, moveRoomVertex, moveWallEnd, addRoom } from '../model/ops';
import { eraseRef, placeItem, placeOpening, placeRoom, placeStair, roomAt, roomOverlaps, rotateRefs, snapRoomRect, type RoomPreset } from '../model/place';
import type { FloorMaterial, OpeningKind, Project, Pt, Ref, StairShape, WallKind } from '../model/types';
import { uid, visibleIn } from '../model/types';
import { fmtLen } from '../model/estimate';
import { bounds } from '../model/geometry';
import { dimPoints, hitTest, levelWalls, snapPoint, type SnapResult } from '../plan/hit';
import { app } from './app.svelte';
import { t } from './i18n.svelte';

export interface ViewAdapter {
  host: HTMLElement;
  toWorld(x: number, y: number): Pt | null;
  toScreen(p: Pt, height?: number): { x: number; y: number } | null;
  pxPerMeter(at: Pt): number;
  pick?(x: number, y: number): Ref | null;
}

export type Placement =
  | { type: 'item'; kind: string }
  | { type: 'opening'; kind: OpeningKind }
  | { type: 'room'; preset: RoomPreset }
  | { type: 'stair'; shape: StairShape }
  | { type: 'floor'; material: FloorMaterial }
  | { type: 'color'; color: string };

export interface OverlayShape {
  pts: Pt[];
  closed: boolean;
  tone: 'selected' | 'hover' | 'erase';
  height: number;
}

type PointerLike = { clientX: number; clientY: number; shiftKey: boolean; altKey: boolean; button: number };

class Editor {
  placing = $state.raw<Placement | null>(null);
  hover = $state.raw<Ref | null>(null);
  tip = $state.raw<{ at: Pt; text: string } | null>(null);
  dot = $state.raw<SnapResult | null>(null);
  marquee = $state.raw<{ a: Pt; b: Pt } | null>(null);
  measure = $state.raw<{ a: Pt; b: Pt } | null>(null);
  calibration = $state.raw<{ a: Pt; b: Pt } | null>(null);
  typed = $state('');
  drafting = $state(false);
  tick = $state(0);

  private views = new Set<ViewAdapter>();
  private active: ViewAdapter | null = null;
  private base: Project | null = null;
  private from: Pt | null = null;
  private to: Pt | null = null;
  private press: { x: number; y: number; world: Pt } | null = null;
  private turn = 0;
  private pending: { ref: Ref | null; x: number; y: number } | null = null;
  private gesture: ((q: Pt, e: PointerLike) => void) | null = null;
  private gestureEnd: ((q: Pt, e: PointerLike) => void) | null = null;
  private dragPlacing = false;

  constructor() {
    if (typeof window === 'undefined') return;
    window.addEventListener('pointermove', (e) => this.move(e));
    window.addEventListener('pointerup', (e) => this.up(e));
  }

  register(v: ViewAdapter) {
    this.views.add(v);
    return () => {
      this.views.delete(v);
      if (this.active === v) this.active = null;
    };
  }

  redraw() {
    this.tick = untrack(() => this.tick) + 1;
  }

  private viewAt(e: PointerLike): ViewAdapter | null {
    const el = document.elementFromPoint(e.clientX, e.clientY);
    if (!el || el.closest('[data-ui]')) return null;
    for (const v of this.views) if (v.host.contains(el)) return v;
    return null;
  }

  private tol(v: ViewAdapter, q: Pt) {
    return 12 / Math.max(v.pxPerMeter(q), 1);
  }

  private snap(v: ViewAdapter, q: Pt, from: Pt | null, e: PointerLike, project = app.project) {
    return snapPoint(project, app.level, app.phase, q, { grid: app.grid, tolerance: this.tol(v, q), from, ortho: !!from && !e.shiftKey, disabled: !app.snap || e.altKey });
  }

  private gridDelta(d: Pt): Pt {
    return app.snap ? { x: Math.round(d.x / app.grid) * app.grid, y: Math.round(d.y / app.grid) * app.grid } : d;
  }

  private restore() {
    if (this.base) app.project = this.base;
  }

  // ---------- Pose depuis le catalogue ----------
  startPlacing(p: Placement, e?: PointerEvent) {
    this.cancel();
    app.tool = 'select';
    app.selection = [];
    this.placing = p;
    this.base = app.project;
    this.turn = 0;
    this.dragPlacing = !!e;
    if (e) this.press = { x: e.clientX, y: e.clientY, world: { x: 0, y: 0 } };
  }

  private applyPlacement(project: Project, q: Pt): Ref | null {
    const p = this.placing!;
    const { level, phase, drawStatus: status } = app;
    if (p.type === 'item') return { kind: 'item', id: placeItem(project, level, phase, p.kind, q, status, this.turn).id };
    if (p.type === 'opening') {
      const o = placeOpening(project, level, phase, p.kind, q, status);
      return o && { kind: 'opening', id: o.id };
    }
    if (p.type === 'room') return { kind: 'room', id: placeRoom(project, level, phase, p.preset, q, status, project.walls.some((w) => w.level === level) ? 'partition' : 'exterior').id };
    if (p.type === 'stair') return { kind: 'stair', id: placeStair(project, level, p.shape, q, status, this.turn).id };
    const room = roomAt(project, level, phase, q);
    if (!room) return null;
    if (p.type === 'floor') room.material = p.material;
    else room.color = p.color;
    return { kind: 'room', id: room.id };
  }

  private placeAt(q: Pt) {
    let ref: Ref | null = null;
    app.preview(this.base!, (project) => {
      ref = this.applyPlacement(project, q);
    });
    if (!ref) this.restore();
    return ref as Ref | null;
  }

  private commitPlacement(q: Pt) {
    const p = this.placing!;
    if (p.type === 'room') {
      const s = snapRoomRect(this.base!, app.level, app.phase, q.x - p.preset.w / 2, q.y - p.preset.h / 2, p.preset.w, p.preset.h);
      if (roomOverlaps(this.base!, app.level, app.phase, s.x, s.y, p.preset.w, p.preset.h)) {
        this.restore();
        app.notify(t('Cette place est déjà prise : posez la pièce à côté.'), 'err');
        return;
      }
    }
    const ref = this.placeAt(q);
    if (!ref) {
      if (p.type === 'opening') app.notify(t('Posez-la sur un mur.'), 'err');
      if (p.type === 'floor' || p.type === 'color') app.notify(t('Cliquez dans une pièce.'), 'err');
      return;
    }
    app.endPreview(this.base!);
    if (p.type === 'opening' || p.type === 'floor' || p.type === 'color') {
      this.base = app.project;
      return;
    }
    this.placing = null;
    this.base = null;
    if (p.type !== 'room') app.selection = [ref];
    else app.selection = [ref];
  }

  turnPlacing() {
    this.turn = (this.turn + 90) % 360;
  }

  // ---------- Souris ----------
  /** Renvoie vrai si l'éditeur prend le geste ; sinon la vue déplace la caméra. */
  down(v: ViewAdapter, e: PointerLike): boolean {
    const q = v.toWorld(e.clientX, e.clientY);
    if (!q || e.button !== 0) return false;
    this.active = v;
    if (this.placing) {
      this.commitPlacement(q);
      return true;
    }
    const tool = app.tool;
    if (tool === 'select') return this.downSelect(v, q, e);
    if (tool === 'erase') {
      const hit = this.hitAt(v, q, e);
      if (hit) {
        app.commit((p) => eraseRef(p, hit, app.phase));
        this.hover = null;
      }
      return true;
    }
    if (tool === 'detect') {
      const poly = detectRoom(levelWalls(app.project, app.level, app.phase), q);
      if (!poly) app.notify(t('Aucun espace fermé ici : reliez d’abord les murs.'), 'err');
      else {
        let id = '';
        app.commit((p) => (id = addRoom(p, app.level, t('Pièce'), poly, 'oak', app.drawStatus).id));
        app.selection = [{ kind: 'room', id }];
        app.tool = 'select';
      }
      return true;
    }
    if (tool === 'note') {
      const s = this.snap(v, q, null, e).point;
      const id = uid('n');
      app.commit((p) => void p.notes.push({ id, level: app.level, x: s.x, y: s.y, text: t('Note') }));
      app.selection = [{ kind: 'note', id }];
      app.panel = 'props';
      app.tool = 'select';
      return true;
    }
    if (tool === 'wall') {
      if (!this.drafting) {
        this.base = app.project;
        this.from = this.to = this.snap(v, q, null, e).point;
        this.drafting = true;
        this.press = { x: e.clientX, y: e.clientY, world: this.from };
      } else this.commitWall();
      return true;
    }
    if (tool === 'room') {
      const base = app.project;
      const a = this.snap(v, q, null, e).point;
      let last = a;
      this.gesture = (c, m) => {
        last = this.snap(v, c, null, m, base).point;
        const w = Math.abs(a.x - last.x);
        const h = Math.abs(a.y - last.y);
        this.tip = { at: last, text: `${fmtLen(w)} × ${fmtLen(h)}` };
        if (w < 0.4 || h < 0.4) app.project = base;
        else app.preview(base, (p) => void addBoxedRoom(p, app.level, t('Pièce'), Math.min(a.x, last.x), Math.min(a.y, last.y), w, h, 'oak', app.drawStatus, base.walls.some((x) => x.level === app.level) ? 'partition' : 'exterior'));
      };
      this.gestureEnd = () => {
        this.tip = null;
        if (app.project === base) return app.notify(t('Glissez pour tracer une pièce d’au moins 40 cm de côté.'), 'err');
        const room = app.project.rooms.at(-1)!;
        app.endPreview(base);
        app.selection = [{ kind: 'room', id: room.id }];
        app.tool = 'select';
      };
      return true;
    }
    if (tool === 'dim' || tool === 'measure' || tool === 'calibrate') {
      const a = this.snap(v, q, null, e).point;
      this.measure = { a, b: a };
      this.gesture = (c, m) => {
        const b = this.snap(v, c, a, m).point;
        this.measure = { a, b };
        this.tip = { at: b, text: fmtLen(dist(a, b), app.project.dimUnit) };
      };
      this.gestureEnd = () => {
        const b = this.measure!.b;
        this.tip = tool === 'measure' ? this.tip : null;
        if (tool === 'dim' && dist(a, b) > 0.05) app.commit((p) => void p.dims.push({ id: uid('d'), level: app.level, a, b, offset: -0.2 }));
        if (tool === 'calibrate' && dist(a, b) > 0.05) this.calibration = { a, b };
        if (tool !== 'measure') this.measure = null;
      };
      return true;
    }
    return false;
  }

  private hitAt(v: ViewAdapter, q: Pt, e: PointerLike): Ref | null {
    const picked = v.pick?.(e.clientX, e.clientY);
    if (picked !== undefined && picked !== null) return picked;
    return hitTest(app.project, app.level, app.phase, q, this.tol(v, q));
  }

  private edit(apply: (p: Project, q: Pt, e: PointerLike) => void) {
    const base = app.project;
    this.gesture = (q, e) => app.preview(base, (p) => apply(p, q, e));
    this.gestureEnd = () => app.endPreview(base);
  }

  private downSelect(v: ViewAdapter, q: Pt, e: PointerLike): boolean {
    const handle = this.handleAt(v, q, e);
    if (handle) {
      handle();
      return true;
    }
    const hit = this.hitAt(v, q, e);
    if (e.shiftKey && !hit) {
      this.marquee = { a: q, b: q };
      this.gesture = (c) => (this.marquee = { a: q, b: c });
      this.gestureEnd = (c) => {
        if (dist(q, c) > 0.12) app.select(this.refsInBox(q, c));
        this.marquee = null;
      };
      return true;
    }
    const chosen = hit && app.selection.some((r) => r.kind === hit.kind && r.id === hit.id);
    // Le sol sert à déplacer la vue tant que la pièce n'est pas sélectionnée.
    if (!hit || (hit.kind === 'room' && !chosen)) {
      this.pending = { ref: hit, x: e.clientX, y: e.clientY };
      return false;
    }
    if (e.shiftKey) app.select(chosen ? app.selection.filter((r) => r.id !== hit.id) : [...app.selection, hit]);
    else if (!chosen) app.select([hit]);
    const refs = app.selection;
    this.edit((p, c, m) => {
      let raw = sub(c, q);
      const only = refs.length === 1 && refs[0].kind === 'wall' ? p.walls.find((w) => w.id === refs[0].id) : null;
      if (only && !m.shiftKey) {
        const n = wallNormal(only);
        raw = mul(n, raw.x * n.x + raw.y * n.y);
      }
      const delta = m.altKey ? raw : this.gridDelta(raw);
      for (const r of refs) moveRef(p, r, delta);
    });
    return true;
  }

  private handleAt(v: ViewAdapter, q: Pt, e: PointerLike): null | (() => void) {
    if (app.selection.length !== 1) return null;
    const r = app.selection[0];
    const p = app.project;
    const px = Math.max(v.pxPerMeter(q), 1);
    const near = (h: Pt) => dist(h, q) < 12 / px;
    if (r.kind === 'wall') {
      const w = p.walls.find((x) => x.id === r.id);
      for (const end of ['a', 'b'] as const) {
        if (w && near(w[end])) return () => this.edit((d, c, m) => moveWallEnd(d, w.id, end, snapPoint(d, w.level, app.phase, c, { grid: app.grid, tolerance: this.tol(v, c), ignoreWall: w.id, disabled: !app.snap || m.altKey }).point));
      }
    }
    if (r.kind === 'room') {
      const room = p.rooms.find((x) => x.id === r.id);
      if (!room) return null;
      for (let i = 0; i < room.points.length; i++) {
        const a = room.points[i];
        const b = room.points[(i + 1) % room.points.length];
        if (near({ x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 })) return () => this.edit((d, c) => moveRoomEdge(d, room.id, i, this.gridDelta(sub(c, q))));
        if (near(a)) return () => this.edit((d, c, m) => moveRoomVertex(d, room.id, i, this.snap(v, c, null, m).point));
      }
    }
    if (r.kind === 'item' || r.kind === 'stair') {
      const it = (r.kind === 'item' ? p.items : p.stairs).find((x) => x.id === r.id);
      if (!it) return null;
      if (near(rotatePoint({ x: it.x, y: it.y - it.d / 2 - 28 / px }, it, it.rotation))) {
        return () =>
          this.edit((d, c, m) => {
            const target = (r.kind === 'item' ? d.items : d.stairs).find((x) => x.id === r.id)!;
            const deg = (Math.atan2(c.y - it.y, c.x - it.x) * 180) / Math.PI + 90;
            target.rotation = m.shiftKey ? Math.round(deg) : Math.round(deg / 15) * 15;
          });
      }
    }
    if (r.kind === 'dim') {
      const dm = p.dims.find((x) => x.id === r.id);
      if (!dm) return null;
      const pts = dimPoints(dm);
      for (const end of ['a', 'b'] as const) if (near(pts[end])) return () => this.edit((d, c, m) => (d.dims.find((x) => x.id === r.id)![end] = this.snap(v, c, null, m).point));
    }
    void e;
    return null;
  }

  private refsInBox(a: Pt, b: Pt): Ref[] {
    const x0 = Math.min(a.x, b.x);
    const x1 = Math.max(a.x, b.x);
    const y0 = Math.min(a.y, b.y);
    const y1 = Math.max(a.y, b.y);
    const inside = (q: Pt) => q.x >= x0 && q.x <= x1 && q.y >= y0 && q.y <= y1;
    const p = app.project;
    const out: Ref[] = [];
    for (const w of levelWalls(p, app.level, app.phase)) if (inside(w.a) && inside(w.b)) out.push({ kind: 'wall', id: w.id });
    for (const i of p.items) if (i.level === app.level && visibleIn(i.status, app.phase) && inside(i)) out.push({ kind: 'item', id: i.id });
    for (const s of p.stairs) if (s.level === app.level && inside(s)) out.push({ kind: 'stair', id: s.id });
    return out;
  }

  private previewWall() {
    const from = this.from!;
    const to = this.to!;
    if (dist(from, to) < 0.03) return this.restore();
    app.preview(this.base!, (p) => void addWall(p, app.level, from, to, app.wallKind, app.drawStatus));
    this.tip = { at: to, text: this.typed ? `${this.typed} ${app.project.dimUnit}` : fmtLen(dist(from, to), app.project.dimUnit) };
  }

  private commitWall(stop = false) {
    const ok = this.from && this.to && dist(this.from, this.to) >= 0.03;
    if (ok) {
      this.previewWall();
      app.endPreview(this.base!);
      this.base = app.project;
      this.from = this.to;
    }
    this.typed = '';
    if (stop) this.stopDraft();
  }

  private stopDraft() {
    this.drafting = false;
    this.from = this.to = this.press = null;
    this.tip = null;
    this.base = null;
  }

  private move(e: PointerEvent) {
    if (this.pending && Math.hypot(e.clientX - this.pending.x, e.clientY - this.pending.y) > 5) this.pending = null;
    if (this.gesture && this.active) {
      const q = this.active.toWorld(e.clientX, e.clientY);
      if (q) this.gesture(q, e);
      return;
    }
    const v = this.viewAt(e);
    if (this.placing) {
      const q = v?.toWorld(e.clientX, e.clientY);
      if (q) {
        this.active = v;
        this.placeAt(q);
      } else this.restore();
      return;
    }
    if (!v) {
      this.hover = null;
      this.dot = null;
      return;
    }
    const q = v.toWorld(e.clientX, e.clientY);
    if (!q) return;
    this.active = v;
    if (app.tool === 'select' || app.tool === 'erase') {
      this.dot = null;
      if (e.buttons) return;
      const h = this.hitAt(v, q, e);
      if (h?.id !== this.hover?.id) this.hover = h;
      return;
    }
    this.hover = null;
    if (this.drafting && this.from) {
      this.to = this.snap(v, q, this.from, e, this.base!).point;
      this.dot = null;
      this.previewWall();
    } else this.dot = this.snap(v, q, null, e);
  }

  private up(e: PointerEvent) {
    const q = this.active?.toWorld(e.clientX, e.clientY) ?? { x: 0, y: 0 };
    const end = this.gestureEnd;
    this.gesture = null;
    this.gestureEnd = null;
    end?.(q, e);
    if (this.pending) {
      const { ref } = this.pending;
      this.pending = null;
      if (ref) app.select([ref]);
      else app.selection = [];
    }
    if (this.placing && this.dragPlacing && this.press) {
      const moved = Math.hypot(e.clientX - this.press.x, e.clientY - this.press.y) > 8;
      this.dragPlacing = false;
      this.press = null;
      const v = this.viewAt(e);
      const w = v?.toWorld(e.clientX, e.clientY);
      if (moved && w) this.commitPlacement(w);
      else if (moved) this.cancel();
      return;
    }
    if (this.drafting && this.press && this.to) {
      const far = dist(this.press.world, this.to) > 0.15;
      this.press = null;
      if (far) {
        this.commitWall(true);
      }
    }
  }

  /** Échap : abandonne l'action en cours sans toucher au reste. */
  cancel(): boolean {
    let busy = false;
    if (this.placing) {
      this.restore();
      this.placing = null;
      this.base = null;
      busy = true;
    }
    if (this.drafting) {
      this.restore();
      this.stopDraft();
      this.typed = '';
      busy = true;
    }
    if (this.measure || this.calibration) {
      this.measure = this.calibration = null;
      this.tip = null;
      busy = true;
    }
    return busy;
  }

  /** Gère les touches liées au geste en cours ; vrai si la touche est consommée. */
  key(e: KeyboardEvent): boolean {
    if (e.key === 'Escape') {
      const busy = this.cancel();
      if (!busy && app.tool !== 'select') app.tool = 'select';
      else if (!busy) app.selection = [];
      return true;
    }
    if (e.key.toLowerCase() === 'r' && !e.ctrlKey) {
      if (this.placing) {
        this.turnPlacing();
        return true;
      }
      if (app.selection.some((r) => r.kind === 'item' || r.kind === 'stair')) {
        this.rotateSelection();
        return true;
      }
    }
    if (!this.drafting) return false;
    if (/^[0-9.,]$/.test(e.key)) {
      this.typed += e.key;
      this.previewWall();
      return true;
    }
    if (e.key === 'Backspace') {
      this.typed = this.typed.slice(0, -1);
      this.previewWall();
      return true;
    }
    if (e.key === 'Enter') {
      const len = parseFloat(this.typed.replace(',', '.'));
      if (len > 0 && this.from && this.to && dist(this.from, this.to) > 0) {
        const metres = app.project.dimUnit === 'cm' && len > 20 ? len / 100 : len;
        this.to = add(this.from, mul(wallDir({ a: this.from, b: this.to }), metres));
        this.commitWall();
      } else this.commitWall(true);
      return true;
    }
    return false;
  }

  rotateSelection(deg = 90) {
    const refs = app.selection;
    app.commit((p) => rotateRefs(p, refs, deg));
  }

  eraseSelection() {
    const refs = app.selection;
    app.commit((p) => refs.forEach((r) => eraseRef(p, r, app.phase)));
    app.selection = [];
  }

  applyCalibration(value: string) {
    const c = this.calibration;
    if (!c) return;
    const real = parseFloat(value.replace(',', '.'));
    if (!(real > 0)) return app.notify(t('Indiquez la longueur réelle en mètres.'), 'err');
    const k = real / dist(c.a, c.b);
    app.commit((p) => {
      const u = p.underlays.find((x) => x.level === app.level);
      if (!u) return;
      u.x = c.a.x + (u.x - c.a.x) * k;
      u.y = c.a.y + (u.y - c.a.y) * k;
      u.width *= k;
    });
    this.calibration = null;
    app.tool = 'select';
    app.fitTick++;
    app.notify(t('Échelle du calque réglée'));
  }

  private shape(r: Ref, tone: OverlayShape['tone']): OverlayShape | null {
    const p = app.project;
    if (r.kind === 'wall') {
      const w = p.walls.find((e) => e.id === r.id);
      return w ? { pts: wallPolygon(w, 0, wallLength(w)), closed: true, tone, height: w.height ?? app.levelInfo.height } : null;
    }
    if (r.kind === 'room') {
      const room = p.rooms.find((e) => e.id === r.id);
      return room ? { pts: room.points, closed: true, tone, height: 0 } : null;
    }
    if (r.kind === 'item' || r.kind === 'stair') {
      const e = (r.kind === 'item' ? p.items : p.stairs).find((x) => x.id === r.id);
      return e ? { pts: boxCorners(e), closed: true, tone, height: r.kind === 'item' ? (e as { h: number }).h : 0.2 } : null;
    }
    if (r.kind === 'opening') {
      const o = p.openings.find((e) => e.id === r.id);
      const w = o && p.walls.find((e) => e.id === o.wall);
      if (!o || !w) return null;
      const d = wallDir(w);
      const c = add(w.a, mul(d, o.offset));
      return { pts: wallPolygon({ ...w, a: add(c, mul(d, -o.width / 2)), b: add(c, mul(d, o.width / 2)), thickness: w.thickness + 0.06 }, 0, o.width), closed: true, tone, height: o.sill + o.height };
    }
    if (r.kind === 'dim') {
      const d = p.dims.find((e) => e.id === r.id);
      if (!d) return null;
      const q = dimPoints(d);
      return { pts: [q.a, q.b], closed: false, tone, height: 0 };
    }
    return null;
  }

  overlay(): OverlayShape[] {
    const out: OverlayShape[] = [];
    for (const r of app.selection) {
      const s = this.shape(r, 'selected');
      if (s) out.push(s);
    }
    const h = this.hover;
    if (h && !app.selection.some((r) => r.id === h.id)) {
      const s = this.shape(h, app.tool === 'erase' ? 'erase' : 'hover');
      if (s) out.push(s);
    }
    return out;
  }

  /** Poignées visibles : extrémités de mur, milieux de côté de pièce, rotation de meuble. */
  handles(pxPerMeter: number): { p: Pt; round: boolean }[] {
    if (app.selection.length !== 1) return [];
    const r = app.selection[0];
    const p = app.project;
    if (r.kind === 'wall') {
      const w = p.walls.find((e) => e.id === r.id);
      return w ? [{ p: w.a, round: false }, { p: w.b, round: false }] : [];
    }
    if (r.kind === 'room') {
      const room = p.rooms.find((e) => e.id === r.id);
      return room ? room.points.map((a, i) => { const b = room.points[(i + 1) % room.points.length]; return { p: { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }, round: true }; }) : [];
    }
    if (r.kind === 'item' || r.kind === 'stair') {
      const e = (r.kind === 'item' ? p.items : p.stairs).find((x) => x.id === r.id);
      return e ? [{ p: rotatePoint({ x: e.x, y: e.y - e.d / 2 - 28 / pxPerMeter }, e, e.rotation), round: true }] : [];
    }
    return [];
  }

  anchor(): { at: Pt; height: number } | null {
    const shapes = app.selection.map((r) => this.shape(r, 'selected')).filter(Boolean) as OverlayShape[];
    if (!shapes.length) return null;
    const b = bounds(shapes.flatMap((s) => s.pts));
    return { at: { x: b.minX + b.w / 2, y: b.minY }, height: Math.max(...shapes.map((s) => s.height)) };
  }
}

export const editor = new Editor();
export type { WallKind };
