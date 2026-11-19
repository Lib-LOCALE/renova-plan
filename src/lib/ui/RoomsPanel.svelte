<script lang="ts">
  import { chipFor } from '../model/catalog';
  import { computeEstimate, fmtArea } from '../model/estimate';
  import { polygonArea } from '../model/geometry';
  import { visibleIn } from '../model/types';
  import { app } from '../state/app.svelte';
  import { t } from '../state/i18n.svelte';

  const est = $derived(computeEstimate(app.project));
  const groups = $derived(
    app.project.levels.map((l) => {
      const rooms = app.project.rooms.filter((r) => r.level === l.id && visibleIn(r.status, app.phase));
      return { level: l, rooms, area: rooms.reduce((s, r) => s + polygonArea(r.points), 0) };
    }),
  );
</script>

<div class="rooms">
  <div class="total"><div><span>{t('Surface habitable')}</span><b class="num">{fmtArea(est.carrezArea)}</b></div><div><span>{t('Pièces')}</span><b class="num">{app.project.rooms.length}</b></div></div>
  {#each groups as g}
    <section>
      <div class="rowb"><h2>{g.level.name}</h2><span class="muted num">{fmtArea(g.area)}</span></div>
      {#if g.rooms.length}
        <div class="deck">
          {#each g.rooms as r, i}
            <button
              class="chip"
              style={`--c:${r.color}`}
              aria-pressed={app.selection.some((s) => s.id === r.id)}
              onmouseenter={() => (app.hoverRoom = r.id)}
              onmouseleave={() => (app.hoverRoom = null)}
              onfocus={() => (app.hoverRoom = r.id)}
              onblur={() => (app.hoverRoom = null)}
              onclick={() => { app.level = r.level; app.select([{ kind: 'room', id: r.id }]); }}
            >
              <i>{#if r.status === 'new'}<em>{t('À créer')}</em>{:else if r.status === 'demolish'}<em>{t('À démolir')}</em>{/if}</i>
              <b>{t(r.name)}</b>
              <span class="num">{fmtArea(polygonArea(r.points))}</span>
              <small>{chipFor(r.color)?.code ?? `P ${i + 1}`}</small>
            </button>
          {/each}
        </div>
      {:else}
        <p class="muted">{t('Aucune pièce visible dans cette phase. Tracez-en une avec l’outil Pièce.')}</p>
      {/if}
    </section>
  {/each}
</div>

<style>
  .rooms { display: grid; gap: 16px; padding: 12px; }
  .total { display: flex; gap: 24px; }
  .total span { display: block; font-size: 12px; color: var(--soft); }
  .total b { font-size: 22px; font-weight: 750; letter-spacing: -0.01em; }
  section { display: grid; gap: 7px; }
  .deck { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
  .chip { display: grid; grid-template-columns: 1fr auto; gap: 0 6px; padding: 0 0 6px; overflow: hidden; border: 1px solid var(--rule); border-radius: 2px; background: var(--plan-sheet); text-align: left; transition: transform 0.14s cubic-bezier(0.2, 0.8, 0.2, 1), box-shadow 0.14s; }
  .chip:hover, .chip[aria-pressed='true'] { transform: translateY(-2px) rotate(-0.7deg); box-shadow: 0 5px 12px rgb(20 32 48 / 0.2); }
  .chip[aria-pressed='true'] { border-color: var(--ink); }
  .chip i { grid-column: 1 / -1; display: flex; align-items: flex-end; height: 42px; margin-bottom: 5px; padding: 4px; background: var(--c); }
  .chip em { padding: 0 4px; font-size: 10.5px; font-style: normal; font-weight: 650; color: #1a2128; background: rgb(255 255 255 / 0.74); }
  .chip b { grid-column: 1 / -1; padding: 0 7px; font-size: 12.5px; font-weight: 650; line-height: 1.15; overflow-wrap: anywhere; }
  .chip span { padding-left: 7px; font-size: 12px; color: var(--soft); }
  .chip small { padding-right: 7px; font-size: 10.5px; color: var(--soft); align-self: end; }
</style>
