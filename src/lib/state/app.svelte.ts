import { estimateCsv } from '../model/estimate';
import { parseProject } from '../model/migrate';
import { deleteRefs, duplicateRefs, setStatus } from '../model/ops';
import { blankProject, sampleFlat } from '../model/templates';
import type { Phase, Project, Ref, Status, WallKind } from '../model/types';
import { standaloneSvg } from '../plan/markup';
import { t } from './i18n.svelte';
import { dataUrlBytes, exportFile, openProjectFile, saveProjectFile } from './platform';

export type Tool = 'select' | 'erase' | 'wall' | 'room' | 'detect' | 'dim' | 'note' | 'measure' | 'calibrate';
export type ViewLayout = 'plan' | 'split' | '3d';
export type Panel = 'props' | 'rooms' | 'budget';
export type Theme = 'light' | 'dark' | 'dracula';

const AUTOSAVE = 'renovaplan.autosave.v2';
const PREFS = 'renovaplan.prefs.v3';
const MAX_HISTORY = 120;

function store(key: string, value?: string): string | null {
  try {
    if (value !== undefined) localStorage.setItem(key, value);
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function seedProject(): Project | null {
  const node = typeof document !== 'undefined' ? document.getElementById('renova-seed') : null;
  if (!node?.textContent) return null;
  try {
    return parseProject(node.textContent);
  } catch {
    return null;
  }
}

/** Les modèles sont écrits en français : on nomme projet, niveaux et pièces dans la langue de l'interface. */
function localized(p: Project): Project {
  p.name = t(p.name);
  for (const l of p.levels) l.name = t(l.name);
  for (const r of p.rooms) r.name = t(r.name);
  for (const w of p.works) [w.lot, w.label] = [t(w.lot), t(w.label)];
  return p;
}

class AppState {
  project = $state.raw<Project>(localized(blankProject()));
  level = $state('l0');
  phase = $state<Phase>('after');
  layout = $state<ViewLayout>('plan');
  panel = $state<Panel | null>(null);
  tool = $state<Tool>('select');
  wallKind = $state<WallKind>('partition');
  drawStatus = $state<Status>('new');
  selection = $state.raw<Ref[]>([]);
  hoverRoom = $state<string | null>(null);
  grid = $state(0.05);
  snap = $state(true);
  showFurniture = $state(true);
  showLabels = $state(true);
  showDims = $state(false);
  theme = $state<Theme>('light');
  dialog = $state<null | 'new' | 'help' | 'report'>(null);
  toast = $state<{ text: string; kind: 'ok' | 'err' } | null>(null);
  filePath = $state<string | null>(null);
  dirty = $state(false);
  fitTick = $state(0);
  past = $state.raw<Project[]>([]);
  future = $state.raw<Project[]>([]);
  shot: (() => string | null) | null = null;
  private toastTimer: ReturnType<typeof setTimeout> | undefined;

  constructor() {
    const prefs = store(PREFS);
    if (prefs) {
      try {
        const v = JSON.parse(prefs);
        this.theme = v.theme ?? 'light';
        this.grid = v.grid ?? 0.05;
        this.layout = v.layout ?? 'plan';
      } catch {
        /* préférences illisibles : valeurs par défaut */
      }
    }
    const seeded = seedProject();
    const saved = store(AUTOSAVE);
    if (seeded) this.adopt(seeded);
    else if (saved) {
      try {
        this.adopt(parseProject(saved));
      } catch {
        /* sauvegarde automatique illisible : on repart du plan vierge */
      }
    }
  }

  get levelInfo() {
    return this.project.levels.find((l) => l.id === this.level) ?? this.project.levels[0];
  }

  savePrefs() {
    store(PREFS, JSON.stringify({ theme: this.theme, grid: this.grid, layout: this.layout }));
  }

  adopt(p: Project, path: string | null = null) {
    this.project = p;
    this.level = p.levels[0]?.id ?? 'l0';
    this.selection = [];
    this.past = [];
    this.future = [];
    this.filePath = path;
    this.dirty = false;
    this.fitTick++;
    store(AUTOSAVE, JSON.stringify(p));
  }

  private touched(previous: Project) {
    this.past = [...this.past.slice(-MAX_HISTORY + 1), previous];
    this.future = [];
    this.markChanged();
  }

  private markChanged() {
    this.dirty = true;
    store(AUTOSAVE, JSON.stringify(this.project));
  }

  /** Modification unitaire : copie, applique, empile dans l'historique. */
  commit(fn: (p: Project) => void) {
    const before = this.project;
    const next = structuredClone(before);
    fn(next);
    this.project = next;
    this.touched(before);
  }

  /** Geste en cours : repart toujours de l'état initial pour éviter les dérives. */
  preview(base: Project, fn: (p: Project) => void) {
    const next = structuredClone(base);
    fn(next);
    this.project = next;
  }

  endPreview(base: Project) {
    if (base !== this.project) this.touched(base);
  }

  undo() {
    const prev = this.past.at(-1);
    if (!prev) return;
    this.future = [this.project, ...this.future];
    this.past = this.past.slice(0, -1);
    this.project = prev;
    this.dropMissing();
    this.markChanged();
  }

  redo() {
    const next = this.future[0];
    if (!next) return;
    this.past = [...this.past, this.project];
    this.future = this.future.slice(1);
    this.project = next;
    this.dropMissing();
    this.markChanged();
  }

  private dropMissing() {
    const p = this.project;
    const has = (r: Ref) => {
      const list: { id: string }[] = { wall: p.walls, opening: p.openings, room: p.rooms, item: p.items, stair: p.stairs, dim: p.dims, note: p.notes }[r.kind];
      return list.some((e) => e.id === r.id);
    };
    this.selection = this.selection.filter(has);
    if (!p.levels.some((l) => l.id === this.level)) this.level = p.levels[0].id;
  }

  select(refs: Ref[]) {
    this.selection = refs;
  }

  setTool(tool: Tool) {
    this.tool = tool;
    if (tool !== 'select') this.selection = [];
  }

  setPhase(phase: Phase) {
    this.phase = phase;
    this.drawStatus = phase === 'before' ? 'existing' : 'new';
    this.selection = [];
  }

  deleteSelection() {
    if (!this.selection.length) return;
    const refs = this.selection;
    this.commit((p) => deleteRefs(p, refs));
    this.selection = [];
  }

  duplicateSelection() {
    if (!this.selection.length) return;
    let out: Ref[] = [];
    const refs = this.selection;
    this.commit((p) => {
      out = duplicateRefs(p, refs);
    });
    this.selection = out;
  }

  statusSelection(status: Status) {
    const refs = this.selection;
    this.commit((p) => setStatus(p, refs, status));
  }

  notify(text: string, kind: 'ok' | 'err' = 'ok') {
    this.toast = { text, kind };
    clearTimeout(this.toastTimer);
    this.toastTimer = setTimeout(() => (this.toast = null), kind === 'err' ? 6000 : 2600);
  }

  private async canReplaceProject() {
    return !this.dirty || window.confirm(t('Abandonner les modifications non enregistrées du projet actuel ?'));
  }

  async newFrom(kind: 'blank' | 'flat') {
    if (!(await this.canReplaceProject())) return;
    this.adopt(localized(kind === 'blank' ? blankProject() : sampleFlat()));
    this.dialog = null;
    this.tool = 'select';
  }

  async open() {
    try {
      const f = await openProjectFile();
      if (!f) return;
      const project = parseProject(f.text);
      if (!(await this.canReplaceProject())) return;
      this.adopt(project, f.path);
      this.notify(`${f.name} ${t('ouvert')}`);
    } catch (e) {
      this.notify(e instanceof Error ? e.message : t("Impossible d'ouvrir ce fichier."), 'err');
    }
  }

  async save(as = false) {
    try {
      const snapshot = this.project;
      const path = await saveProjectFile(this.safeName, JSON.stringify(snapshot, null, 1), as ? null : this.filePath);
      if (!path || this.project !== snapshot) return;
      this.filePath = path.includes('\\') || path.includes('/') ? path : null;
      this.dirty = false;
      this.notify(t('Projet enregistré'));
    } catch (e) {
      this.notify(`${t('Enregistrement impossible')} : ${e instanceof Error ? e.message : e}`, 'err');
    }
  }

  get safeName() {
    return this.project.name.replace(/[^\p{L}\p{N} _-]+/gu, '').trim() || 'projet';
  }

  private async exported(run: () => Promise<string | null>) {
    try {
      const out = await run();
      if (out) this.notify(`${t('Exporté')} : ${out.split(/[\\/]/).pop()}`);
    } catch (e) {
      this.notify(`${t('Export impossible')} : ${e instanceof Error ? e.message : e}`, 'err');
    }
  }

  exportSvg() {
    const name = `${this.safeName} - ${this.levelInfo.name}.svg`;
    return this.exported(() => exportFile(name, standaloneSvg(this.project, this.level, this.phase), 'image/svg+xml'));
  }

  exportCsv() {
    return this.exported(() => exportFile(`${this.safeName} - chiffrage.csv`, estimateCsv(this.project), 'text/csv'));
  }

  exportPng() {
    const url = this.shot?.();
    if (!url) return this.notify(t('Ouvrez la vue 3D pour exporter une image.'), 'err');
    return this.exported(() => exportFile(`${this.safeName} - 3D.png`, dataUrlBytes(url), 'image/png'));
  }
}

export const app = new AppState();
