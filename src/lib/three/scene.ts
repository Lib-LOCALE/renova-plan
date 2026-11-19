import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { RoomEnvironment } from 'three/examples/jsm/environments/RoomEnvironment.js';
import { isGlazedKind } from '../model/catalog';
import { add, bounds, centroid, mul, projectOnWall, solidSpans, wallDir, wallLength, wallNormal, wallOpenings, endExtension, rotatePoint } from '../model/geometry';
import type { Level, Opening, Phase, Project, Pt, Ref, Stair, Wall } from '../model/types';
import { visibleIn } from '../model/types';
import { buildItem } from './furniture';
import { Mesher, type Faces } from './mesher';
import { surfaceTexture } from './textures';

export type ViewMode = 'model' | 'exploded' | 'level' | 'exterior' | 'top' | 'walk';

export interface SceneOptions {
  phase: Phase;
  mode: ViewMode;
  level: string;
  cut: number;
  hour: number;
  furniture: boolean;
  tint: boolean;
}

export interface SceneLabel {
  id: string;
  text: string;
  sub?: string;
  kind: 'room' | 'tag' | 'flag';
  pos: THREE.Vector3;
}

const FLOOR_KEYS: Record<string, string> = { oak: 'oak', tile: 'tile', slate: 'slate', concrete: 'concrete', terracotta: 'terracotta', carpet: 'carpet', vinyl: 'vinyl' };
const EYE = 1.6;

export class HouseScene {
  renderer: THREE.WebGLRenderer;
  scene = new THREE.Scene();
  camera = new THREE.PerspectiveCamera(36, 1, 0.08, 400);
  controls: OrbitControls;
  private root: THREE.Group | null = null;
  private mats = new Map<string, THREE.Material>();
  private sun = new THREE.DirectionalLight(0xfff0dc, 2.5);
  private dirty = true;
  private raf = 0;
  private obs: ResizeObserver;
  private keys = new Set<string>();
  private walkYaw = 0;
  private walkPitch = 0;
  private collide: { a: Pt; b: Pt }[] = [];
  private lastT = 0;
  private center = new THREE.Vector3(5, 1.2, 4);
  private radius = 9;
  private opts: SceneOptions | null = null;
  private shadowOnly: THREE.ShadowMaterial | null = null;
  labels: SceneLabel[] = [];
  onFrame: (() => void) | null = null;
  private overlay = new THREE.Group();
  private lastLevel = '';

  constructor(private host: HTMLElement) {
    this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, preserveDrawingBuffer: true });
    this.renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1;
    host.prepend(this.renderer.domElement);
    const pm = new THREE.PMREMGenerator(this.renderer);
    this.scene.environment = pm.fromScene(new RoomEnvironment(), 0.04).texture;
    this.scene.environmentIntensity = 0.55;
    pm.dispose();
    this.sun.castShadow = true;
    this.sun.shadow.mapSize.set(2048, 2048);
    this.sun.shadow.bias = -0.0004;
    this.sun.shadow.normalBias = 0.025;
    this.scene.add(this.sun, this.sun.target, this.overlay);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.12;
    this.controls.maxPolarAngle = Math.PI * 0.495;
    this.controls.addEventListener('change', () => this.invalidate());
    this.obs = new ResizeObserver(() => this.resize());
    this.obs.observe(host);
    this.resize();
    const el = this.renderer.domElement;
    el.addEventListener('pointerdown', this.down);
    window.addEventListener('keydown', this.keyDown);
    window.addEventListener('keyup', this.keyUp);
    this.raf = requestAnimationFrame(this.loop);
  }

  dispose() {
    cancelAnimationFrame(this.raf);
    this.obs.disconnect();
    window.removeEventListener('keydown', this.keyDown);
    window.removeEventListener('keyup', this.keyUp);
    this.renderer.domElement.removeEventListener('pointerdown', this.down);
    this.clearRoot();
    this.mats.forEach((m) => m.dispose());
    this.controls.dispose();
    this.renderer.dispose();
    this.renderer.domElement.remove();
  }

  invalidate() {
    this.dirty = true;
  }

  private resize() {
    const w = this.host.clientWidth || 1;
    const h = this.host.clientHeight || 1;
    this.renderer.setSize(w, h);
    this.camera.aspect = w / h;
    this.camera.fov = w / h < 0.9 ? 52 : 36;
    this.camera.updateProjectionMatrix();
    this.invalidate();
  }

  private keyDown = (e: KeyboardEvent) => {
    if (this.opts?.mode !== 'walk' || (e.target as HTMLElement)?.closest?.('input,textarea,select')) return;
    this.keys.add(e.key.toLowerCase());
  };
  private keyUp = (e: KeyboardEvent) => this.keys.delete(e.key.toLowerCase());

  private down = (e: PointerEvent) => {
    if (this.opts?.mode !== 'walk') return;
    let lx = e.clientX;
    let ly = e.clientY;
    const mv = (m: PointerEvent) => {
      this.walkYaw -= (m.clientX - lx) * 0.004;
      this.walkPitch = Math.max(-1.2, Math.min(1.2, this.walkPitch - (m.clientY - ly) * 0.004));
      this.applyWalkLook();
      lx = m.clientX;
      ly = m.clientY;
    };
    const up = () => {
      window.removeEventListener('pointermove', mv);
      window.removeEventListener('pointerup', up);
    };
    window.addEventListener('pointermove', mv);
    window.addEventListener('pointerup', up);
  };

  private ray(x: number, y: number) {
    const r = this.renderer.domElement.getBoundingClientRect();
    this.camera.updateMatrixWorld();
    const ray = new THREE.Raycaster();
    ray.setFromCamera(new THREE.Vector2(((x - r.left) / r.width) * 2 - 1, -((y - r.top) / r.height) * 2 + 1), this.camera);
    return ray;
  }

  /** Point du sol du niveau actif sous le pointeur. */
  groundAt(x: number, y: number, p: Project): Pt | null {
    if (!this.opts) return null;
    const l = p.levels.find((e) => e.id === this.opts!.level);
    const plane = new THREE.Plane(new THREE.Vector3(0, 1, 0), -(l ? this.levelBase(p, l, this.opts) : 0));
    const hit = this.ray(x, y).ray.intersectPlane(plane, new THREE.Vector3());
    return hit && hit.distanceTo(this.camera.position) < 120 ? { x: hit.x, y: hit.z } : null;
  }

  worldPoint(p: Project, q: Pt, height = 0) {
    const l = this.opts && p.levels.find((e) => e.id === this.opts!.level);
    return new THREE.Vector3(q.x, (l && this.opts ? this.levelBase(p, l, this.opts) : 0) + height, q.y);
  }

  /** Dessine sélection, survol et poignées par-dessus la maquette. */
  setOverlay(p: Project, shapes: { pts: Pt[]; closed: boolean; tone: string; height: number }[], handles: Pt[]) {
    this.overlay.traverse((o) => (o as THREE.Mesh).geometry?.dispose());
    this.overlay.clear();
    const l = this.opts && p.levels.find((e) => e.id === this.opts!.level);
    const cutAt = l && this.opts ? (this.opts.mode === 'exterior' ? l.height : this.opts.cut * l.height) : 2.5;
    for (const s of shapes) {
      const color = s.tone === 'erase' ? 0xd9483e : s.tone === 'hover' ? 0x6ea2ff : 0x1d5fd8;
      const at = this.worldPoint(p, { x: 0, y: 0 }, Math.min(s.height, cutAt) + 0.02).y;
      const pts = s.pts.map((q) => new THREE.Vector3(q.x, at, q.y));
      const line = new (s.closed ? THREE.LineLoop : THREE.Line)(new THREE.BufferGeometry().setFromPoints(pts), new THREE.LineBasicMaterial({ color, depthTest: false, transparent: true }));
      line.renderOrder = 30;
      this.overlay.add(line);
      if (s.closed && s.pts.length > 2) {
        const geo = new THREE.BufferGeometry().setFromPoints(pts);
        geo.setIndex(THREE.ShapeUtils.triangulateShape(s.pts.map((q) => new THREE.Vector2(q.x, q.y)), []).flat());
        const fill = new THREE.Mesh(geo, new THREE.MeshBasicMaterial({ color, transparent: true, opacity: s.tone === 'hover' ? 0.13 : 0.26, side: THREE.DoubleSide, depthTest: false }));
        fill.renderOrder = 29;
        this.overlay.add(fill);
      }
    }
    const dot = new THREE.SphereGeometry(0.09, 16, 12);
    for (const h of handles) {
      const m = new THREE.Mesh(dot, new THREE.MeshBasicMaterial({ color: 0x1d5fd8, depthTest: false, transparent: true }));
      m.position.copy(this.worldPoint(p, h, 0.04));
      m.renderOrder = 31;
      this.overlay.add(m);
    }
    this.invalidate();
  }

  /** Vignette de catalogue : rend une petite scène et renvoie l'image. */
  capture(p: Project, o: SceneOptions, target: [number, number, number], dir: [number, number, number], distance: number): string {
    this.update(p, o);
    const l = Math.hypot(...dir);
    this.controls.target.set(...target);
    this.camera.position.set(target[0] + (dir[0] / l) * distance, target[1] + (dir[1] / l) * distance, target[2] + (dir[2] / l) * distance);
    this.controls.update();
    this.renderer.render(this.scene, this.camera);
    return this.renderer.domElement.toDataURL('image/png');
  }

  pickAt(x: number, y: number): Ref | null {
    if (!this.root) return null;
    const ray = this.ray(x, y);
    for (const hit of ray.intersectObjects(this.root.children, true)) {
      const owners = hit.object.userData.owners as (string | undefined)[] | undefined;
      const owner = owners && hit.faceIndex != null ? owners[hit.faceIndex] : undefined;
      if (owner) {
        const [kind, id] = owner.split('|');
        return { kind: kind as Ref['kind'], id };
      }
    }
    return null;
  }

  private material(key: string): THREE.Material {
    let m = this.mats.get(key);
    if (m) return m;
    const std = (o: THREE.MeshStandardMaterialParameters) => new THREE.MeshStandardMaterial({ roughness: 0.85, metalness: 0, ...o });
    if (key.startsWith('paint:')) m = std({ color: key.slice(6), roughness: 0.62 });
    else if (key.startsWith('chip:')) m = std({ color: key.slice(5), roughness: 0.9 });
    else if (key === 'glass') m = new THREE.MeshPhysicalMaterial({ color: 0xbfe3ee, transparent: true, opacity: 0.28, roughness: 0.04, metalness: 0, envMapIntensity: 1.4, side: THREE.DoubleSide, depthWrite: false });
    else if (key === 'cap') m = std({ color: 0x2b3238, roughness: 0.95 });
    else if (key === 'demolish') m = std({ color: 0xd9483e, transparent: true, opacity: 0.72, roughness: 0.7 });
    else if (key === 'new') m = std({ color: 0x4fb48b, roughness: 0.8 });
    else if (key === 'frame') m = std({ color: 0xf5f5f1, roughness: 0.45 });
    else if (key === 'leaf') m = std({ color: 0xece9e1, roughness: 0.55 });
    else if (key === 'entry') m = std({ color: 0x3f5560, roughness: 0.5 });
    else if (key === 'gdoor') m = std({ color: 0x8e969b, roughness: 0.55, metalness: 0.25 });
    else if (key === 'tread') m = std({ color: 0xb98a55, roughness: 0.55 });
    else if (key === 'slab') m = std({ color: 0xc9c7bf });
    else if (key === 'solid') m = std({ color: 0xb5b2a9 });
    else if (key === 'flat') m = std({ color: 0x4a5056, roughness: 0.92 });
    else if (key.startsWith('roof:')) {
      m = std({ map: surfaceTexture('roof'), color: key.slice(5), roughness: 0.82, side: THREE.DoubleSide });
    } else m = std({ map: surfaceTexture(key), roughness: key === 'oak' || key === 'tile' ? 0.5 : 0.9 });
    this.mats.set(key, m);
    return m;
  }

  private clearRoot() {
    if (!this.root) return;
    this.scene.remove(this.root);
    this.root.traverse((o) => (o as THREE.Mesh).geometry?.dispose());
    this.root = null;
  }

  private levelBase(p: Project, l: Level, o: SceneOptions): number {
    if (o.mode === 'level') return 0;
    const i = p.levels.findIndex((x) => x.id === l.id);
    return l.elevation + (o.mode === 'exploded' ? i * 2.1 : 0);
  }

  update(p: Project, o: SceneOptions, resetCamera = false) {
    const modeChanged = this.opts?.mode !== o.mode || (o.mode === 'level' && this.lastLevel !== o.level);
    this.lastLevel = o.level;
    this.opts = o;
    this.clearRoot();
    const m = new Mesher();
    this.labels = [];
    this.collide = [];
    const showAll = o.mode === 'model' || o.mode === 'exploded' || o.mode === 'exterior';
    const levels = showAll ? p.levels : p.levels.filter((l) => l.id === o.level);
    const all: Pt[] = [];
    for (const level of levels) this.buildLevel(m, p, level, o, all);
    if (o.mode === 'exterior') this.buildRoofs(m, p, o);
    const group = m.build((k) => this.material(k));
    const b = all.length ? bounds(all) : { minX: 0, minY: 0, maxX: 8, maxY: 6, w: 8, h: 6 };
    const cx = b.minX + b.w / 2;
    const cz = b.minY + b.h / 2;
    const reach = Math.max(b.w, b.h);
    // Maquette posée sur la planche ; le terrain n'apparaît que pour l'extérieur et la visite.
    const outdoors = o.mode === 'exterior' || o.mode === 'walk';
    const ground = new THREE.Mesh(
      new THREE.CircleGeometry(outdoors ? 140 : reach * 2 + 6, 64),
      outdoors ? this.material('grass') : (this.shadowOnly ??= new THREE.ShadowMaterial({ opacity: 0.2 })),
    );
    ground.rotation.x = -Math.PI / 2;
    ground.position.set(cx, -0.04 - (o.mode === 'level' || o.mode === 'walk' ? 0.2 : 0), cz);
    ground.receiveShadow = true;
    group.add(ground);
    this.root = group;
    this.scene.add(group);
    const top = levels.reduce((h, l) => Math.max(h, this.levelBase(p, l, o) + l.height), 2.5);
    this.center.set(cx, top * 0.42, cz);
    this.radius = Math.hypot(b.w, b.h, top) * 0.5 + 0.5;
    this.placeSun(o, p);
    this.controls.enabled = o.mode !== 'walk';
    if (resetCamera || modeChanged) this.frame(o, p);
    this.invalidate();
  }

  private placeSun(o: SceneOptions, p: Project) {
    // Trajectoire simplifiée : plein sud à 13 h, azimut tourné selon le nord du projet.
    const t = (o.hour - 13) / 7;
    const az = (t * Math.PI) / 1.7 + (p.north * Math.PI) / 180;
    const alt = Math.max(0.12, Math.cos(t * 1.45) * 1.02);
    const r = this.radius * 3 + 12;
    const dir = new THREE.Vector3(Math.sin(az) * Math.cos(alt), Math.sin(alt), Math.cos(az) * Math.cos(alt));
    this.sun.position.copy(this.center).addScaledVector(dir, r);
    this.sun.target.position.copy(this.center);
    const warm = 1 - Math.min(1, Math.abs(t));
    this.sun.color.setRGB(1, 0.86 + warm * 0.1, 0.72 + warm * 0.2);
    this.sun.intensity = 1.5 + warm * 1.4;
    const c = this.sun.shadow.camera;
    const s = this.radius + 5;
    c.left = -s;
    c.right = s;
    c.top = s;
    c.bottom = -s;
    c.near = 1;
    c.far = r * 2;
    c.updateProjectionMatrix();
  }

  private frame(o: SceneOptions, p: Project) {
    const c = this.center;
    const r = this.radius;
    const vf = (this.camera.fov * Math.PI) / 360;
    const hf = Math.atan(Math.tan(vf) * this.camera.aspect);
    const dist = (r / Math.sin(Math.min(vf, hf))) * 0.86;
    this.controls.target.copy(c);
    const set = (x: number, y: number, z: number) => {
      const l = Math.hypot(x, y, z);
      this.camera.position.set(c.x + (x / l) * dist, c.y + (y / l) * dist, c.z + (z / l) * dist);
    };
    if (o.mode === 'top') set(0.001, 1, 0.001);
    else if (o.mode === 'level') set(0.1, 0.78, 0.68);
    else if (o.mode === 'exterior') set(0.58, 0.24, 0.82);
    else if (o.mode === 'exploded') set(0.42, 0.52, 0.85);
    else if (o.mode === 'model') set(0.42, 0.62, 0.78);
    else if (o.mode === 'walk') {
      const room = p.rooms.find((x) => x.level === o.level && visibleIn(x.status, o.phase));
      const q = room ? centroid(room.points) : { x: c.x, y: c.z };
      const level = p.levels.find((l) => l.id === o.level);
      this.camera.position.set(q.x, (level?.elevation ?? 0) + EYE, q.y);
      this.walkYaw = Math.PI;
      this.walkPitch = -0.05;
      this.applyWalkLook();
      return;
    }
    this.controls.update();
  }

  private applyWalkLook() {
    const e = new THREE.Euler(this.walkPitch, this.walkYaw, 0, 'YXZ');
    this.camera.quaternion.setFromEuler(e);
    this.invalidate();
  }

  private buildLevel(m: Mesher, p: Project, l: Level, o: SceneOptions, all: Pt[]) {
    const base = this.levelBase(p, l, o);
    const walls = p.walls.filter((w) => w.level === l.id && visibleIn(w.status, o.phase));
    const pts = walls.flatMap((w) => [w.a, w.b]);
    all.push(...pts);
    const c = pts.length ? centroid(pts.length > 2 ? [{ x: bounds(pts).minX, y: bounds(pts).minY }, { x: bounds(pts).maxX, y: bounds(pts).minY }, { x: bounds(pts).maxX, y: bounds(pts).maxY }, { x: bounds(pts).minX, y: bounds(pts).maxY }] : pts) : { x: 0, y: 0 };
    const limit = o.mode === 'exterior' || o.mode === 'walk' ? 99 : o.cut * l.height;
    for (const r of p.rooms) {
      if (r.level !== l.id || !visibleIn(r.status, o.phase) || (o.phase === 'site' && r.status === 'demolish')) continue;
      m.owner = `room|${r.id}`;
      const key = o.tint ? `chip:${r.color}` : (FLOOR_KEYS[r.material] ?? 'oak');
      m.prism(r.points, base - l.slab, base, key, 'slab', 'slab');
      if (o.mode === 'walk') m.prism(r.points, base + l.height, base + l.height + 0.02, 'plaster', null, 'plaster');
      const q = centroid(r.points);
      this.labels.push({ id: r.id, text: r.name, kind: 'room', pos: new THREE.Vector3(q.x, base + 0.05, q.y) });
    }
    for (const w of walls) this.buildWall(m, p, w, walls, l, base, limit, c, o);
    for (const s of p.stairs) {
      if (s.level === l.id && s.view !== '2d' && visibleIn(s.status, o.phase)) this.buildStair(m, s, base, l);
    }
    if (o.furniture) {
      for (const it of p.items) {
        if (it.level !== l.id || !visibleIn(it.status, o.phase)) continue;
        m.owner = `item|${it.id}`;
        buildItem(m, it, base);
      }
    }
    m.owner = undefined;
  }

  private buildWall(m: Mesher, p: Project, w: Wall, walls: Wall[], l: Level, base: number, limit: number, c: Pt, o: SceneOptions) {
    const len = wallLength(w);
    if (len < 0.01) return;
    // Les murs de façade montent jusqu'au niveau supérieur : pas de jour au droit des dalles.
    const next = p.levels[p.levels.findIndex((x) => x.id === l.id) + 1];
    const facade = !w.height && (w.kind === 'exterior' || w.kind === 'party') && o.mode === 'exterior';
    const full = (w.height ?? l.height) + (facade && next ? next.slab : 0);
    const H = Math.min(full, limit);
    const d = wallDir(w);
    const n = wallNormal(w);
    const t = w.thickness / 2;
    const mid = add(w.a, mul(d, len / 2));
    const outward = (mid.x - c.x) * n.x + (mid.y - c.y) * n.y >= 0;
    const outer = w.kind === 'party' ? 'party' : 'facade';
    let inner = 'plaster';
    let outside = w.kind === 'exterior' || w.kind === 'party' ? outer : 'plaster';
    if (o.phase === 'site' && w.status === 'new') inner = outside = 'new';
    if (o.phase === 'site' && w.status === 'demolish') inner = outside = 'demolish';
    const faces: Faces = { front: outward ? outside : inner, back: outward ? inner : outside, top: H < full - 1e-3 ? 'cap' : inner, ends: inner, bottom: undefined };
    m.owner = `wall|${w.id}`;
    const ops = wallOpenings(p, w.id, o.phase);
    const spans = solidSpans(len, ops);
    const extA = endExtension(w.a, w, walls);
    const extB = endExtension(w.b, w, walls);
    for (const [from, to] of spans) {
      m.box(w.a, base, d, from < 1e-3 ? -extA : from, to > len - 1e-3 ? len + extB : to, 0, H, -t, t, faces);
    }
    for (const op of ops) this.buildOpening(m, w, op, base, H, faces, d, n, t, o);
    if (o.mode === 'walk') {
      let last = 0;
      for (const [from, to] of spans) {
        if (to > last) this.collide.push({ a: add(w.a, mul(d, from)), b: add(w.a, mul(d, to)) });
        last = to;
      }
      for (const op of ops) if (isGlazedKind(op.kind) && op.sill > 0.3) this.collide.push({ a: add(w.a, mul(d, op.offset - op.width / 2)), b: add(w.a, mul(d, op.offset + op.width / 2)) });
    }
  }

  private buildOpening(m: Mesher, w: Wall, o: Opening, base: number, H: number, f: Faces, d: Pt, n: Pt, t: number, s: SceneOptions) {
    const x0 = o.offset - o.width / 2;
    const x1 = o.offset + o.width / 2;
    const top = Math.min(o.sill + o.height, H);
    m.owner = `wall|${w.id}`;
    m.box(w.a, base, d, x0, x1, 0, Math.min(o.sill, H), -t, t, { ...f, ends: undefined, top: 'frame' });
    if (top < H) m.box(w.a, base, d, x0, x1, top, H, -t, t, { ...f, ends: undefined, bottom: 'plaster' });
    if (o.sill >= H) return;
    m.owner = `opening|${o.id}`;
    const y0 = o.sill;
    const bar = 0.05;
    if (isGlazedKind(o.kind)) {
      m.box(w.a, base, d, x0, x0 + bar, y0, top, -0.04, 0.04, 'frame');
      m.box(w.a, base, d, x1 - bar, x1, y0, top, -0.04, 0.04, 'frame');
      m.box(w.a, base, d, x0, x1, y0, y0 + bar, -0.04, 0.04, 'frame');
      if (top >= o.sill + o.height - 1e-3) m.box(w.a, base, d, x0, x1, top - bar, top, -0.04, 0.04, 'frame');
      const parts = o.width > 1.6 ? 3 : o.width > 0.7 ? 2 : 1;
      for (let i = 1; i < parts; i++) m.box(w.a, base, d, x0 + (i * o.width) / parts - 0.025, x0 + (i * o.width) / parts + 0.025, y0, top, -0.035, 0.035, 'frame');
      m.box(w.a, base, d, x0 + bar, x1 - bar, y0 + bar, top - bar * 0.5, -0.006, 0.006, { front: 'glass', back: 'glass' });
      if (o.tag) this.labels.push({ id: o.id, text: o.tag, kind: o.tag === '?' ? 'flag' : 'tag', pos: new THREE.Vector3(w.a.x + d.x * o.offset, base + Math.min(top, y0 + 1), w.a.y + d.y * o.offset) });
      return;
    }
    if (o.kind === 'passage') return;
    if (o.kind === 'garage-door') {
      if (top >= o.height) m.box(w.a, base, d, x0, x1, 0, top, -0.03, 0.03, 'gdoor');
      return;
    }
    if (o.kind === 'entry-door') {
      m.box(w.a, base, d, x0, x1, 0, top, -0.03, 0.03, 'entry');
      return;
    }
    // Vantail entrouvert : donne la profondeur sans masquer le passage.
    if (top < 1.2 && s.mode !== 'walk') return;
    const side = o.flip ? -1 : 1;
    const leaves = o.kind === 'double-door' ? 2 : 1;
    const lw = o.width / leaves;
    for (let i = 0; i < leaves; i++) {
      const left = i === 0 ? !o.hinge : false;
      const hinge = add(add(w.a, mul(d, left ? x0 : x1)), mul(n, side * t));
      const rot = rotatePoint(left ? d : mul(d, -1), { x: 0, y: 0 }, (left ? 1 : -1) * side * 62);
      m.box(hinge, base, rot, 0, lw, 0.01, Math.min(top, o.height), -0.018, 0.018, 'leaf');
    }
  }

  private buildStair(m: Mesher, s: Stair, base: number, l: Level) {
    m.owner = `stair|${s.id}`;
    const rise = (l.height + l.slab) / s.steps;
    const c = { x: s.x, y: s.y };
    const world = (p: Pt) => rotatePoint({ x: s.x + p.x, y: s.y + p.y }, c, s.rotation);
    const x = -s.w / 2;
    const y = -s.d / 2;
    const flip = s.flip ? -1 : 1;
    const step = (i: number, pts: Pt[]) => {
      const top = (i + 1) * rise;
      m.prism(pts.map(world), base + Math.max(0, top - 0.24), base + top, 'tread', 'tread', 'tread');
    };
    const rect = (x0: number, x1: number, z0: number, z1: number): Pt[] => [{ x: x0, y: z0 }, { x: x1, y: z0 }, { x: x1, y: z1 }, { x: x0, y: z1 }];
    if (s.shape === 'straight') {
      for (let i = 0; i < s.steps; i++) step(i, rect(x + (i * s.w) / s.steps, x + ((i + 1) * s.w) / s.steps, y, -y));
      return;
    }
    if (s.shape === 'quarter') {
      const run = s.w - s.d;
      const n = Math.max(2, s.steps - 4);
      for (let i = 0; i < n; i++) step(i, rect(x + (run * i) / n, x + (run * (i + 1)) / n, y, -y));
      const cx = x + run;
      const cz = flip * -y;
      for (let i = 0; i < 4; i++) {
        const a0 = (Math.PI / 2) * (i / 4);
        const a1 = (Math.PI / 2) * ((i + 1) / 4);
        const ray = (a: number) => {
          const r = s.d / Math.max(Math.cos(a), Math.sin(a));
          return { x: cx + Math.sin(a) * r, y: cz - flip * Math.cos(a) * r };
        };
        const corner = a0 < Math.PI / 4 && a1 > Math.PI / 4 ? [{ x: cx + s.d, y: cz - flip * s.d }] : [];
        step(n + i, [{ x: cx, y: cz }, ray(a0), ...corner, ray(a1)]);
      }
      return;
    }
    const lane = s.d / 2;
    const run = s.w - lane;
    const per = Math.max(2, Math.floor((s.steps - 3) / 2));
    const firstZ = flip > 0 ? [0, -y] : [y, 0];
    const lastZ = flip > 0 ? [y, 0] : [0, -y];
    for (let i = 0; i < per; i++) step(i, rect(x + (run * i) / per, x + (run * (i + 1)) / per, firstZ[0], firstZ[1]));
    const cx = x + run;
    const turn = s.steps - 2 * per;
    for (let i = 0; i < turn; i++) {
      const sweep = (a: number) => (flip > 0 ? Math.PI / 2 - (Math.PI * a) / turn : -Math.PI / 2 + (Math.PI * a) / turn);
      const pts: Pt[] = [{ x: cx, y: 0 }];
      for (let k = 0; k <= 4; k++) {
        const a = sweep(i + k / 4);
        const r = Math.min(lane / Math.max(Math.abs(Math.cos(a)), 1e-3), (s.d / 2) / Math.max(Math.abs(Math.sin(a)), 1e-3));
        pts.push({ x: cx + Math.cos(a) * r, y: Math.sin(a) * r });
      }
      step(per + i, pts);
    }
    for (let i = 0; i < per; i++) step(per + turn + i, rect(x + run - (run * (i + 1)) / per, x + run - (run * i) / per, lastZ[0], lastZ[1]));
  }

  private buildRoofs(m: Mesher, p: Project, o: SceneOptions) {
    m.owner = undefined;
    for (const r of p.roofs) {
      const l = p.levels.find((x) => x.id === r.level);
      if (!l) continue;
      const y0 = this.levelBase(p, l, o) + (r.base ?? l.height);
      const ox = r.overhang;
      const key = `roof:${r.color}`;
      const x0 = r.x - ox;
      const x1 = r.x + r.w + ox;
      const z0 = r.y - ox;
      const z1 = r.y + r.d + ox;
      if (r.kind === 'flat') {
        m.box({ x: x0, y: z0 }, y0, { x: 1, y: 0 }, 0, x1 - x0, 0, r.rise || 0.18, 0, z1 - z0, 'flat');
        continue;
      }
      const H = y0 + r.rise;
      if (r.ridge === 'x') {
        const zm = (z0 + z1) / 2;
        const lx = x1 - x0;
        const sl = Math.hypot(zm - z0, r.rise);
        m.quad(key, [x0, y0, z0], [x0, H, zm], [x1, H, zm], [x1, y0, z0], [[0, 0], [0, sl], [lx, sl], [lx, 0]]);
        m.quad(key, [x1, y0, z1], [x1, H, zm], [x0, H, zm], [x0, y0, z1], [[0, 0], [0, sl], [lx, sl], [lx, 0]]);
        for (const [x, s] of [[r.x, 1], [r.x + r.w, -1]] as [number, number][]) {
          const k = (r.d / 2 / (zm - z0)) * r.rise;
          const a: [number, number, number] = [x, y0, r.y];
          const b: [number, number, number] = [x, y0 + k, zm];
          const c: [number, number, number] = [x, y0, r.y + r.d];
          if (s > 0) m.triangle('facade', a, c, b);
          else m.triangle('facade', a, b, c);
        }
      } else {
        const xm = (x0 + x1) / 2;
        const lz = z1 - z0;
        const sl = Math.hypot(xm - x0, r.rise);
        m.quad(key, [x0, y0, z1], [xm, H, z1], [xm, H, z0], [x0, y0, z0], [[0, 0], [0, sl], [lz, sl], [lz, 0]]);
        m.quad(key, [x1, y0, z0], [xm, H, z0], [xm, H, z1], [x1, y0, z1], [[0, 0], [0, sl], [lz, sl], [lz, 0]]);
      }
    }
  }

  private walkStep(dt: number) {
    const k = this.keys;
    const f = (k.has('z') || k.has('w') || k.has('arrowup') ? 1 : 0) - (k.has('s') || k.has('arrowdown') ? 1 : 0);
    const r = (k.has('d') || k.has('arrowright') ? 1 : 0) - (k.has('q') || k.has('a') || k.has('arrowleft') ? 1 : 0);
    if (!f && !r) return;
    const speed = 1.7 * dt * (k.has('shift') ? 1.8 : 1);
    const dx = (-Math.sin(this.walkYaw) * f + Math.cos(this.walkYaw) * r) * speed;
    const dz = (-Math.cos(this.walkYaw) * f - Math.sin(this.walkYaw) * r) * speed;
    const free = (x: number, y: number) => this.collide.every((s) => projectOnWall(s, { x, y }).distance > 0.28);
    const c = this.camera.position;
    if (free(c.x + dx, c.z)) c.x += dx;
    if (free(c.x, c.z + dz)) c.z += dz;
    this.invalidate();
  }

  private loop = (t: number) => {
    this.raf = requestAnimationFrame(this.loop);
    const dt = Math.min(0.05, (t - this.lastT) / 1000 || 0);
    this.lastT = t;
    if (this.opts?.mode === 'walk') this.walkStep(dt);
    else this.controls.update();
    if (!this.dirty) return;
    this.dirty = false;
    this.renderer.render(this.scene, this.camera);
    this.onFrame?.();
  };

  project(v: THREE.Vector3) {
    this.camera.updateMatrixWorld();
    const p = v.clone().project(this.camera);
    return { x: ((p.x + 1) / 2) * this.host.clientWidth, y: ((1 - p.y) / 2) * this.host.clientHeight, visible: p.z < 1 && Math.abs(p.x) < 1 && Math.abs(p.y) < 1 };
  }

  screenshot(): string {
    this.renderer.render(this.scene, this.camera);
    return this.renderer.domElement.toDataURL('image/png');
  }
}
