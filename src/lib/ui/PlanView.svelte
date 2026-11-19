<script lang="ts">
  import { onMount } from 'svelte';
  import { LocateFixed, Minus, Plus } from '@lucide/svelte';
  import { app } from '../state/app.svelte';
  import { editor, type ViewAdapter } from '../state/editor.svelte';
  import { t } from '../state/i18n.svelte';
  import { PLAN_CSS, PLAN_DEFS, planBounds, planMarkup } from '../plan/markup';
  import SelectionBar from './SelectionBar.svelte';

  let host: HTMLDivElement;
  let size = $state({ w: 800, h: 600 });
  let cam = $state({ x: -2, y: -2, k: 60 });
  let spaceDown = $state(false);
  let panning = $state(false);

  const adapter: ViewAdapter = {
    get host() {
      return host;
    },
    toWorld(x, y) {
      const r = host.getBoundingClientRect();
      return { x: cam.x + (x - r.left) / cam.k, y: cam.y + (y - r.top) / cam.k };
    },
    toScreen: (p) => ({ x: (p.x - cam.x) * cam.k, y: (p.y - cam.y) * cam.k }),
    pxPerMeter: () => cam.k,
  };

  const markup = $derived(planMarkup(app.project, app.level, app.phase, { highlight: app.hoverRoom, labels: app.showLabels, furniture: app.showFurniture, dims: app.showDims, tr: t }));
  const viewBox = $derived(`${cam.x} ${cam.y} ${size.w / cam.k} ${size.h / cam.k}`);
  const gridStep = $derived.by(() => {
    for (const s of [0.1, 0.25, 0.5, 1, 2, 5]) if (s * cam.k >= 16) return s;
    return 10;
  });
  const shapes = $derived(editor.overlay());
  const handles = $derived(editor.handles(cam.k));
  const screen = (p: { x: number; y: number }) => adapter.toScreen(p)!;

  function fit() {
    const b = planBounds(app.project, app.level, app.phase);
    const k = Math.max(8, Math.min(180, Math.min(size.w / (b.w + 2.4), size.h / (b.h + 2.4))));
    cam = { k, x: b.minX + b.w / 2 - size.w / k / 2, y: b.minY + b.h / 2 - size.h / k / 2 };
  }
  function zoomAt(f: number, sx = size.w / 2, sy = size.h / 2) {
    const k = Math.max(8, Math.min(400, cam.k * f));
    cam = { k, x: cam.x + sx / cam.k - sx / k, y: cam.y + sy / cam.k - sy / k };
  }

  onMount(() => {
    let origin = host.getBoundingClientRect();
    let first = true;
    const ro = new ResizeObserver(([e]) => {
      const r = host.getBoundingClientRect();
      cam = { ...cam, x: cam.x + (origin.left - r.left) / cam.k, y: cam.y + (origin.top - r.top) / cam.k };
      origin = r;
      size = { w: e.contentRect.width || 1, h: e.contentRect.height || 1 };
      if (first || refit) fit();
      first = refit = false;
    });
    ro.observe(host);
    const off = editor.register(adapter);
    return () => {
      ro.disconnect();
      off();
    };
  });

  // Passage plan <-> plan + 3D : on recadre au prochain redimensionnement du volet.
  let refit = false;
  $effect(() => {
    void app.layout;
    refit = true;
  });

  let lastFit = app.fitTick;
  let lastLevel = app.level;
  $effect(() => {
    if (app.fitTick === lastFit && app.level === lastLevel) return;
    lastFit = app.fitTick;
    lastLevel = app.level;
    queueMicrotask(fit);
  });
  $effect(() => {
    void cam;
    editor.redraw();
  });

  function onDown(e: PointerEvent) {
    host.focus({ preventScroll: true });
    if (!spaceDown && editor.down(adapter, e)) return;
    const start = { x: e.clientX, y: e.clientY, cx: cam.x, cy: cam.y };
    panning = true;
    const mv = (m: PointerEvent) => (cam = { ...cam, x: start.cx - (m.clientX - start.x) / cam.k, y: start.cy - (m.clientY - start.y) / cam.k });
    const up = () => {
      panning = false;
      window.removeEventListener('pointermove', mv);
      window.removeEventListener('pointerup', up);
    };
    window.addEventListener('pointermove', mv);
    window.addEventListener('pointerup', up);
  }
  function onWheel(e: WheelEvent) {
    e.preventDefault();
    const r = host.getBoundingClientRect();
    zoomAt(Math.exp(-e.deltaY * (e.deltaMode ? 0.05 : 0.0016)), e.clientX - r.left, e.clientY - r.top);
  }
  function onSpace(e: KeyboardEvent) {
    if (e.key === ' ') {
      spaceDown = e.type === 'keydown';
      e.preventDefault();
    }
  }
</script>

<svelte:head>{@html `<style>${PLAN_CSS}</style>`}</svelte:head>

<div class="wrap">
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div class="plan" class:panning={panning || spaceDown} class:drawing={app.tool !== 'select' || editor.placing} class:grab={!!editor.hover} bind:this={host} tabindex="0" role="application" aria-label={t('Plan 2D')} onpointerdown={onDown} onwheel={onWheel} onkeydown={onSpace} onkeyup={onSpace} oncontextmenu={(e) => e.preventDefault()}>
    <svg class="pl" {viewBox} width={size.w} height={size.h} style={`--k:${cam.k}`} aria-hidden="true">
      {@html PLAN_DEFS}
      <defs><pattern id="grid-a" width={gridStep} height={gridStep} patternUnits="userSpaceOnUse"><path d={`M${gridStep} 0H0V${gridStep}`} class="g1" /></pattern></defs>
      <rect x={cam.x} y={cam.y} width={size.w / cam.k} height={size.h / cam.k} fill="url(#grid-a)" />
      {@html markup}
      {#each shapes as s}<polygon class={`ov ${s.tone}`} points={s.pts.map((p) => `${p.x},${p.y}`).join(' ')} />{/each}
      {#each handles as h}{#if h.round}<circle class="hd" cx={h.p.x} cy={h.p.y} r={7 / cam.k} />{:else}<rect class="hd" x={h.p.x - 7 / cam.k} y={h.p.y - 7 / cam.k} width={14 / cam.k} height={14 / cam.k} />{/if}{/each}
      {#if editor.dot}<circle class="sn" class:strong={editor.dot.kind === 'end' || editor.dot.kind === 'wall'} cx={editor.dot.point.x} cy={editor.dot.point.y} r={6 / cam.k} />{/if}
      {#if editor.measure}<line class="ms" x1={editor.measure.a.x} y1={editor.measure.a.y} x2={editor.measure.b.x} y2={editor.measure.b.y} />{/if}
      {#if editor.marquee}<rect class="mq" x={Math.min(editor.marquee.a.x, editor.marquee.b.x)} y={Math.min(editor.marquee.a.y, editor.marquee.b.y)} width={Math.abs(editor.marquee.a.x - editor.marquee.b.x)} height={Math.abs(editor.marquee.a.y - editor.marquee.b.y)} />{/if}
    </svg>
    {#if editor.tip}<output class="tip" style={`left:${screen(editor.tip.at).x + 14}px;top:${screen(editor.tip.at).y + 14}px`}>{editor.tip.text}</output>{/if}
  </div>
  <SelectionBar view={adapter} />
  <div class="zoom" data-ui>
    <button title={t('Zoom avant')} aria-label={t('Zoom avant')} onclick={() => zoomAt(1.25)}><Plus size={18} /></button>
    <button title={t('Zoom arrière')} aria-label={t('Zoom arrière')} onclick={() => zoomAt(0.8)}><Minus size={18} /></button>
    <button title={t('Tout voir')} aria-label={t('Tout voir')} onclick={fit}><LocateFixed size={18} /></button>
  </div>
  <div class="scale" aria-hidden="true"><i style={`width:${cam.k}px`}></i>1 m</div>
</div>

<style>
  .wrap { position: absolute; inset: 0; overflow: hidden; }
  .plan { position: absolute; inset: 0; overflow: hidden; background: var(--plan-sheet); outline: none; cursor: grab; touch-action: none; user-select: none; }
  .plan.grab { cursor: pointer; }
  .plan.drawing { cursor: crosshair; }
  .plan.panning { cursor: grabbing; }
  .plan > svg.pl { position: absolute; inset: 0; display: block; }
  :global(.g1) { fill: none; stroke: var(--grid-1); stroke-width: 1px; vector-effect: non-scaling-stroke; }
  .ov { fill: color-mix(in srgb, var(--accent) 12%, transparent); stroke: var(--accent); stroke-width: 2.5px; vector-effect: non-scaling-stroke; }
  .ov.hover { fill: color-mix(in srgb, var(--accent) 7%, transparent); stroke-width: 1.5px; }
  .ov.erase { fill: color-mix(in srgb, var(--st-demolish) 24%, transparent); stroke: var(--st-demolish); }
  .hd { fill: var(--plan-sheet); stroke: var(--accent); stroke-width: 2.5px; vector-effect: non-scaling-stroke; }
  .sn { fill: none; stroke: var(--accent); stroke-width: 1.5px; vector-effect: non-scaling-stroke; }
  .sn.strong { fill: color-mix(in srgb, var(--accent) 30%, transparent); stroke-width: 2.5px; }
  .ms { stroke: var(--mark); stroke-width: 2px; vector-effect: non-scaling-stroke; }
  .mq { fill: color-mix(in srgb, var(--accent) 9%, transparent); stroke: var(--accent); stroke-dasharray: 5 4; stroke-width: 1px; vector-effect: non-scaling-stroke; }
  .tip { position: absolute; padding: 4px 9px; font-size: 14px; font-weight: 650; background: var(--ink); color: var(--panel); pointer-events: none; white-space: nowrap; border-radius: 3px; }
  .zoom { position: absolute; right: 12px; bottom: 34px; display: grid; gap: 2px; padding: 3px; background: var(--panel); border: 1px solid var(--rule); border-radius: 6px; }
  .zoom button { display: grid; place-items: center; width: 36px; height: 36px; border: 0; border-radius: 4px; background: transparent; }
  .zoom button:hover { background: var(--panel-2); }
  .scale { position: absolute; right: 12px; bottom: 8px; display: flex; flex-direction: column; align-items: flex-end; font-size: 12px; color: var(--soft); pointer-events: none; }
  .scale i { display: block; height: 5px; border: 1.5px solid var(--ink); border-top: 0; }
</style>
