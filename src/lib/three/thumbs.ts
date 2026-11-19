import { CATALOG, OPENING_DEFAULTS } from '../model/catalog';
import { addBoxedRoom, addItem, addOpening, addStair, addWall } from '../model/ops';
import { ROOM_PRESETS } from '../model/place';
import type { OpeningKind, StairShape } from '../model/types';
import { emptyProject } from '../model/types';
import { HouseScene, type SceneOptions } from './scene';

const OPTS: SceneOptions = { phase: 'after', mode: 'level', level: 'l0', cut: 1, hour: 14, furniture: true, tint: false };
const cache: Record<string, string> = {};

/** Calcule les vignettes une fois, par petites touches pour ne pas figer l'interface. */
export async function renderThumbs(onThumb: (key: string, src: string) => void, stopped: () => boolean) {
  for (const [k, v] of Object.entries(cache)) onThumb(k, v);
  if (Object.keys(cache).length) return;
  const host = document.createElement('div');
  host.style.cssText = 'position:fixed;left:-400px;top:0;width:176px;height:132px;pointer-events:none';
  document.body.append(host);
  let scene: HouseScene;
  try {
    scene = new HouseScene(host);
  } catch {
    host.remove();
    return;
  }
  const jobs: [string, () => string][] = [];
  for (const c of CATALOG) {
    jobs.push([`item:${c.kind}`, () => {
      const p = emptyProject();
      addItem(p, 'l0', c.kind, 0, 0);
      const r = Math.hypot(c.w, c.d, Math.min(c.h, 2.4)) / 2;
      return scene.capture(p, OPTS, [0, Math.min(c.h, 2.4) / 2 + (c.kind === 'heat-pump' || c.kind === 'panel' ? 1.5 : 0), 0], [0.55, 0.5, 0.85], Math.max(r * 3.5, 0.9));
    }]);
  }
  for (const kind of Object.keys(OPENING_DEFAULTS) as OpeningKind[]) {
    jobs.push([`opening:${kind}`, () => {
      const p = emptyProject();
      const len = OPENING_DEFAULTS[kind].width + 1.2;
      const w = addWall(p, 'l0', { x: -len / 2, y: 0 }, { x: len / 2, y: 0 }, 'partition');
      w.thickness = 0.2;
      addOpening(p, w, len / 2, kind);
      return scene.capture(p, OPTS, [0, 1.15, 0], [0.32, 0.12, 1], Math.max(len * 1.55, 4.6));
    }]);
  }
  for (const r of ROOM_PRESETS) {
    jobs.push([`room:${r.id}`, () => {
      const p = emptyProject();
      addBoxedRoom(p, 'l0', r.name, -r.w / 2, -r.h / 2, r.w, r.h, r.material).color = r.color;
      return scene.capture(p, { ...OPTS, cut: 0.42, tint: true }, [0, 0.3, 0], [0.32, 1, 0.72], Math.hypot(r.w, r.h) * 1.75 + 1);
    }]);
  }
  for (const shape of ['straight', 'quarter', 'half'] as StairShape[]) {
    jobs.push([`stair:${shape}`, () => {
      const p = emptyProject();
      addStair(p, 'l0', 0, 0, shape === 'straight' ? { shape, w: 2.8, d: 0.9 } : shape === 'quarter' ? { shape, w: 2.3, d: 1 } : { shape, w: 2.1, d: 1.8, steps: 15 });
      return scene.capture(p, OPTS, [0, 1.1, 0], [0.45, 0.65, 0.9], 6.4);
    }]);
  }
  for (const [key, make] of jobs) {
    if (stopped()) break;
    await new Promise((r) => requestAnimationFrame(r));
    cache[key] = make();
    onThumb(key, cache[key]);
  }
  scene.dispose();
  host.remove();
}
