<script lang="ts">
  import { onMount } from 'svelte';
  import type { Pt, Ref } from '../model/types';
  import { app } from '../state/app.svelte';
  import { editor, type ViewAdapter } from '../state/editor.svelte';
  import { t } from '../state/i18n.svelte';
  import { HouseScene, type ViewMode } from '../three/scene';
  import SelectionBar from './SelectionBar.svelte';

  const MODES: { id: ViewMode; label: string }[] = [
    { id: 'level', label: 'Ce niveau' },
    { id: 'model', label: 'Toute la maison' },
    { id: 'exploded', label: 'Étages séparés' },
    { id: 'exterior', label: 'Extérieur' },
    { id: 'walk', label: 'Se promener' },
  ];
  const WALLS = [
    { id: 1, label: 'Hauts' },
    { id: 0.55, label: 'Mi-hauteur' },
    { id: 0.12, label: 'Bas' },
  ];
  let host: HTMLDivElement;
  let scene: HouseScene | null = $state(null);
  let failed = $state(false);
  let mode = $state<ViewMode>('level');
  let cut = $state(0.55);
  let hour = $state(15);
  let tags = $state<{ id: string; text: string; x: number; y: number }[]>([]);
  let timer: ReturnType<typeof setTimeout> | undefined;
  const fullHeight = $derived(mode === 'exterior' || mode === 'walk');

  function levelOf(r: Ref): string | undefined {
    const p = app.project;
    if (r.kind === 'wall') return p.walls.find((e) => e.id === r.id)?.level;
    if (r.kind === 'room') return p.rooms.find((e) => e.id === r.id)?.level;
    if (r.kind === 'item') return p.items.find((e) => e.id === r.id)?.level;
    if (r.kind === 'stair') return p.stairs.find((e) => e.id === r.id)?.level;
    if (r.kind === 'opening') {
      const o = p.openings.find((e) => e.id === r.id);
      return p.walls.find((e) => e.id === o?.wall)?.level;
    }
    return undefined;
  }

  const adapter: ViewAdapter = {
    get host() {
      return host;
    },
    toWorld: (x, y) => scene?.groundAt(x, y, app.project) ?? null,
    toScreen(p: Pt, height = 0) {
      if (!scene) return null;
      const q = scene.project(scene.worldPoint(app.project, p, height));
      return q.visible ? q : null;
    },
    pxPerMeter(at) {
      if (!scene) return 40;
      const a = scene.project(scene.worldPoint(app.project, at));
      const b = scene.project(scene.worldPoint(app.project, { x: at.x + 1, y: at.y }));
      return Math.max(4, Math.hypot(a.x - b.x, a.y - b.y));
    },
    pick(x, y) {
      const r = scene?.pickAt(x, y) ?? null;
      return r && levelOf(r) === app.level ? r : null;
    },
  };

  function refreshTags() {
    editor.redraw();
    if (!scene) return;
    const out: typeof tags = [];
    if (mode === 'model' || mode === 'exploded') {
      for (const l of scene.labels) {
        if (l.kind !== 'room') continue;
        const q = scene.project(l.pos);
        if (q.visible) out.push({ id: l.id, text: l.text, x: q.x, y: q.y });
      }
    }
    tags = out;
  }

  onMount(() => {
    try {
      scene = new HouseScene(host);
      scene.onFrame = refreshTags;
      app.shot = () => scene?.screenshot() ?? null;
    } catch {
      failed = true;
    }
    const off = editor.register(adapter);
    return () => {
      clearTimeout(timer);
      off();
      app.shot = null;
      scene?.dispose();
    };
  });

  $effect(() => {
    const p = app.project;
    const o = { phase: app.phase, level: app.level, mode, cut: fullHeight ? 1 : cut, hour, furniture: app.showFurniture, tint: false };
    if (!scene) return;
    const s = scene;
    clearTimeout(timer);
    timer = setTimeout(() => s.update(p, o), 12);
  });

  $effect(() => {
    const p = app.project;
    const shapes = mode === 'walk' ? [] : editor.overlay();
    const m = editor.measure;
    const all = m ? [...shapes, { pts: [m.a, m.b], closed: false, tone: 'selected' as const, height: 0 }] : shapes;
    const handles = mode === 'walk' ? [] : editor.handles(55).map((h) => h.p);
    if (editor.dot) handles.push(editor.dot.point);
    void app.level;
    void cut;
    scene?.setOverlay(p, all, handles);
  });

  function onDown(e: PointerEvent) {
    if (mode === 'walk' || mode === 'exterior' || (e.target as HTMLElement).closest('[data-ui]')) return;
    (document.activeElement as HTMLElement | null)?.blur?.();
    if (editor.down(adapter, e)) {
      e.stopPropagation();
      e.preventDefault();
    }
  }
  const tip = $derived.by(() => {
    void editor.tick;
    return editor.tip ? adapter.toScreen(editor.tip.at) : null;
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="v3" class:drawing={app.tool !== 'select' || editor.placing} class:grab={!!editor.hover} bind:this={host} onpointerdowncapture={onDown} oncontextmenu={(e) => e.preventDefault()}>
  {#if failed}
    <p class="fail">{t('La 3D ne peut pas démarrer : WebGL est indisponible sur cet appareil.')}</p>
  {:else}
    <div class="tags" aria-hidden="true">{#each tags as l (l.id)}<span style={`left:${l.x}px;top:${l.y}px`}>{t(l.text)}</span>{/each}</div>
    {#if tip && editor.tip}<output class="tip" style={`left:${tip.x + 14}px;top:${tip.y + 14}px`}>{editor.tip.text}</output>{/if}
    <SelectionBar view={adapter} />
    <div class="modes" data-ui>
      <div class="seg" role="group" aria-label={t('Vue 3D')}>{#each MODES as m}<button aria-pressed={mode === m.id} onclick={() => (mode = m.id)}>{t(m.label)}</button>{/each}</div>
      {#if !fullHeight}<div class="seg" role="group" aria-label={t('Hauteur des murs')}><span>{t('Murs')}</span>{#each WALLS as w}<button aria-pressed={cut === w.id} onclick={() => (cut = w.id)}>{t(w.label)}</button>{/each}</div>{/if}
    </div>
    <label class="sun" data-ui><span>{t('Soleil')} {Math.floor(hour)} h</span><input type="range" min="7" max="20" step="0.25" bind:value={hour} /></label>
    {#if mode === 'walk'}<div class="walk" data-ui>{t('Z Q S D ou flèches pour marcher · glisser pour regarder')}</div>{/if}
  {/if}
</div>

<style>
  .v3 { position: absolute; inset: 0; overflow: hidden; background: var(--view3d); cursor: grab; touch-action: none; }
  .v3.grab { cursor: pointer; }
  .v3.drawing { cursor: crosshair; }
  .v3 :global(canvas) { position: absolute; inset: 0; display: block; outline: none; cursor: inherit; }
  .tags { position: absolute; inset: 0; pointer-events: none; overflow: hidden; }
  .tags span { position: absolute; transform: translate(-50%, -50%); padding: 2px 7px; white-space: nowrap; font-size: 13px; font-weight: 650; background: color-mix(in srgb, var(--panel) 90%, transparent); border-radius: 3px; }
  .tip { position: absolute; padding: 4px 9px; font-size: 14px; font-weight: 650; background: var(--ink); color: var(--panel); pointer-events: none; white-space: nowrap; border-radius: 3px; }
  /* Bande centrée entre la liste des niveaux (gauche) et les bascules Meubles/Cotes (droite). */
  .modes { position: absolute; top: 10px; left: 140px; right: 140px; display: flex; flex-wrap: wrap; justify-content: center; gap: 8px; pointer-events: none; }
  .modes > * { pointer-events: auto; }
  /* Volet étroit : la bande passe sous les bascules au lieu de les chevaucher. */
  :global(.views[data-layout='split']) .modes { top: 56px; left: 12px; right: 12px; }
  .modes .seg { background: color-mix(in srgb, var(--panel) 96%, transparent); box-shadow: 0 1px 6px rgb(20 32 48 / 0.18); }
  .modes span { align-self: center; padding: 0 5px 0 8px; color: var(--soft); font-size: 13.5px; }
  .sun { position: absolute; right: 12px; bottom: 10px; display: grid; gap: 1px; width: 150px; padding: 6px 10px; font-size: 13px; color: var(--soft); background: color-mix(in srgb, var(--panel) 94%, transparent); border-radius: 6px; }
  .walk { position: absolute; left: 50%; bottom: 14px; transform: translateX(-50%); padding: 6px 11px; background: var(--ink); color: var(--panel); white-space: nowrap; border-radius: 4px; }
  .fail { padding: 24px; max-width: 42ch; }
</style>
