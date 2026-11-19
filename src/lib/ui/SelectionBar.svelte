<script lang="ts">
  import { Copy, FlipHorizontal2, RotateCw, SlidersHorizontal, Trash2 } from '@lucide/svelte';
  import { catalogEntry, MATERIALS, OPENING_DEFAULTS, ROOM_CHIPS, WALL_DEFAULTS } from '../model/catalog';
  import { fmtArea } from '../model/estimate';
  import { polygonArea } from '../model/geometry';
  import { app } from '../state/app.svelte';
  import { editor, type ViewAdapter } from '../state/editor.svelte';
  import { t } from '../state/i18n.svelte';

  let { view }: { view: ViewAdapter } = $props();
  let picker = $state<null | 'floor' | 'color'>(null);
  let barWidth = $state(460);
  const ref = $derived(app.selection.length === 1 ? app.selection[0] : null);
  const room = $derived(ref?.kind === 'room' ? app.project.rooms.find((r) => r.id === ref.id) : undefined);
  const item = $derived(ref?.kind === 'item' ? app.project.items.find((r) => r.id === ref.id) : undefined);
  const opening = $derived(ref?.kind === 'opening' ? app.project.openings.find((r) => r.id === ref.id) : undefined);
  const wall = $derived(ref?.kind === 'wall' ? app.project.walls.find((r) => r.id === ref.id) : undefined);
  const turnable = $derived(ref?.kind === 'item' || ref?.kind === 'stair');
  const title = $derived(
    room ? `${t(room.name)} · ${fmtArea(polygonArea(room.points))}` : item ? item.label || t(catalogEntry(item.kind).label) : opening ? t(OPENING_DEFAULTS[opening.kind].label) : wall ? t(WALL_DEFAULTS[wall.kind].label) : ref?.kind === 'stair' ? t('Escalier') : ref?.kind === 'dim' ? t('Cote') : ref?.kind === 'note' ? t('Note') : `${app.selection.length} ${t('éléments')}`,
  );
  const pos = $derived.by(() => {
    void editor.tick;
    const a = editor.anchor();
    if (!a || editor.placing || !view.host) return null;
    const p = view.toScreen(a.at, a.height);
    if (!p) return null;
    const r = view.host.getBoundingClientRect();
    const half = Math.min(barWidth / 2 + 8, r.width / 2);
    return { x: Math.max(half, Math.min(r.width - half, p.x)), y: Math.max(68, Math.min(r.height - 12, p.y - 16)), width: r.width };
  });
  $effect(() => {
    void ref?.id;
    picker = null;
  });
  const details = () => (app.panel = 'props');
  const flip = () => app.commit((p) => { const o = p.openings.find((x) => x.id === ref!.id)!; if (o.flip) o.hinge = !o.hinge; o.flip = !o.flip; });
</script>

{#if pos}
  <div class="bar" class:compact={pos.width < 620} bind:clientWidth={barWidth} data-ui style={`left:${pos.x}px;top:${pos.y}px`} role="toolbar" aria-label={title}>
    {#if picker === 'floor' && room}
      <div class="pick">{#each MATERIALS as m}<button class:on={room.material === m.id} title={t(m.label)} aria-label={t(m.label)} style={`background:${m.color}`} onclick={() => app.commit((p) => (p.rooms.find((r) => r.id === room.id)!.material = m.id))}></button>{/each}</div>
    {:else if picker === 'color' && room}
      <div class="pick">{#each ROOM_CHIPS as c}<button class:on={room.color === c.color} title={c.name} aria-label={c.name} style={`background:${c.color}`} onclick={() => app.commit((p) => (p.rooms.find((r) => r.id === room.id)!.color = c.color))}></button>{/each}</div>
    {/if}
    <div class="row">
      <b>{title}</b>
      {#if turnable}<button onclick={() => editor.rotateSelection()}><RotateCw size={17} />{t('Tourner')}</button>{/if}
      {#if opening}<button onclick={flip}><FlipHorizontal2 size={17} />{t('Retourner')}</button>{/if}
      {#if room}
        <button aria-pressed={picker === 'floor'} onclick={() => (picker = picker === 'floor' ? null : 'floor')}><i style={`background:${MATERIALS.find((m) => m.id === room.material)?.color}`}></i>{t('Sol')}</button>
        <button aria-pressed={picker === 'color'} onclick={() => (picker = picker === 'color' ? null : 'color')}><i style={`background:${room.color}`}></i>{t('Couleur')}</button>
      {/if}
      {#if ref?.kind !== 'dim' && ref?.kind !== 'note'}<button onclick={() => app.duplicateSelection()}><Copy size={17} />{t('Copier')}</button>{/if}
      <button onclick={details}><SlidersHorizontal size={17} />{t('Détails')}</button>
      <button class="danger" onclick={() => editor.eraseSelection()}><Trash2 size={17} />{t('Supprimer')}</button>
    </div>
  </div>
{/if}

<style>
  .bar { position: absolute; z-index: 20; display: grid; gap: 4px; justify-items: center; transform: translate(-50%, -100%); pointer-events: none; }
  .row, .pick { pointer-events: auto; display: flex; align-items: center; gap: 2px; padding: 4px; background: var(--panel); border: 1px solid var(--rule); border-radius: 8px; box-shadow: var(--shadow); }
  b { padding: 0 9px 0 7px; font-size: 14.5px; font-weight: 700; white-space: nowrap; }
  .row button { display: inline-flex; align-items: center; gap: 5px; height: 36px; padding: 0 9px; border: 0; border-radius: 5px; background: transparent; font-size: 14px; white-space: nowrap; }
  .row button:hover, .row button[aria-pressed='true'] { background: var(--panel-2); }
  .compact b { max-width: 110px; overflow: hidden; text-overflow: ellipsis; }
  .compact .row button { gap: 0; font-size: 0; padding: 0 8px; }
  .row .danger:hover { color: var(--st-demolish); }
  i { width: 16px; height: 16px; border: 1px solid var(--rule); border-radius: 3px; }
  .pick button { width: 30px; height: 30px; border: 2px solid transparent; border-radius: 5px; }
  .pick button.on { border-color: var(--ink); }
</style>
