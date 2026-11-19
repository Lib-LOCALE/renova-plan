<script lang="ts">
  import { X } from '@lucide/svelte';
  import { computeEstimate, fmtArea, fmtEur } from '../model/estimate';
  import { polygonArea } from '../model/geometry';
  import type { Phase } from '../model/types';
  import { standaloneSvg } from '../plan/markup';
  import { app } from '../state/app.svelte';
  import { t } from '../state/i18n.svelte';

  let box: HTMLDialogElement | undefined = $state();
  let shot = $state<string | null>(null);
  let printing = $state(false);
  $effect(() => {
    if (!box) return;
    if (app.dialog && !box.open) box.showModal();
    if (!app.dialog && box.open) box.close();
    if (app.dialog === 'report') shot = app.shot?.() ?? null;
  });
  const close = () => (app.dialog = null);
  const est = $derived(computeEstimate(app.project));
  const KEYS: [string, string][] = [
    ['Glisser un objet', 'Le déplacer'], ['Glisser le vide', 'Tourner ou déplacer la vue'], ['Clic droit + glisser', 'Déplacer la vue en 3D'], ['Molette', 'Zoomer'], ['R', 'Tourner l’objet'], ['Suppr', 'Supprimer'], ['Échap', 'Annuler l’action en cours'], ['Ctrl+Z · Ctrl+Y', 'Annuler, rétablir'], ['Ctrl+D', 'Copier'], ['Ctrl+S', 'Enregistrer'], ['1 · 2 · 3', '3D, plan, les deux'], ['Maj + glisser', 'Sélectionner plusieurs objets'],
  ];
  const STARTS: { id: 'blank' | 'flat'; title: string; text: string }[] = [
    { id: 'blank', title: 'Plan vierge', text: 'Partez d’une grille, d’un relevé ou d’une photo de croquis.' },
    { id: 'flat', title: 'Appartement T2', text: 'Un niveau simple pour tester un réaménagement.' },
  ];
  const PHASES: Phase[] = ['before', 'after'];
  function print() {
    printing = true;
    setTimeout(() => {
      window.print();
      printing = false;
    }, 60);
  }
</script>

<dialog bind:this={box} oncancel={(e) => { e.preventDefault(); close(); }} aria-label={t('Fenêtre')}>
  {#if app.dialog}
    <div class="dlg">
      <div class="rowb hd">
        <h2>{app.dialog === 'new' ? t('Nouveau projet') : app.dialog === 'help' ? t('Raccourcis') : t('Dossier pour artisans')}</h2>
        <button class="icon" aria-label={t('Fermer')} onclick={close}><X size={16} /></button>
      </div>
      {#if app.dialog === 'new'}
        {#if app.dirty}<p class="warn">{t('Le projet ouvert a des changements non enregistrés. Enregistrez-le avant d’en créer un autre.')}</p>{/if}
        <div class="starts">
          {#each STARTS as s}<button onclick={() => app.newFrom(s.id)}><b>{t(s.title)}</b><span>{t(s.text)}</span></button>{/each}
        </div>
      {:else if app.dialog === 'help'}
        <dl class="keys">{#each KEYS as [k, v]}<div><dt><kbd>{t(k)}</kbd></dt><dd>{t(v)}</dd></div>{/each}</dl>
      {:else}
        <p>{t('Plans avant et après de chaque niveau, surfaces, postes de travaux et vue 3D, prêts à imprimer ou à enregistrer en PDF.')}</p>
        {#if !shot}<p class="muted">{t('Ouvrez la vue 3D pour ajouter une image de la maquette.')}</p>{/if}
        <div class="end"><button class="btn primary" onclick={print}>{t('Imprimer ou enregistrer en PDF')}</button></div>
      {/if}
    </div>
  {/if}
</dialog>

{#if app.dialog === 'report' || printing}
  <div id="print-root">
    <section class="pg cover">
      <h1>{app.project.name}</h1>
      <p>{fmtArea(est.carrezArea)} · {app.project.rooms.length} {t('pièces')} · {fmtEur(est.total)}</p>
      {#if shot}<img src={shot} alt={t('Vue 3D de la maquette')} />{/if}
    </section>
    {#each app.project.levels as l}
      {#each PHASES as ph}
        <section class="pg">
          <h2>{l.name} · {ph === 'before' ? t('Avant travaux') : t('Après travaux')}</h2>
          <div class="sheet">{@html standaloneSvg(app.project, l.id, ph)}</div>
        </section>
      {/each}
    {/each}
    <section class="pg">
      <h2>{t('Surfaces')}</h2>
      <table><thead><tr><th>{t('Niveau')}</th><th>{t('Pièce')}</th><th>{t('État')}</th><th class="n">m²</th></tr></thead>
        <tbody>{#each app.project.rooms as r}<tr><td>{app.project.levels.find((l) => l.id === r.level)?.name}</td><td>{r.name}</td><td>{r.status === 'new' ? t('À créer') : r.status === 'demolish' ? t('À démolir') : t('Existant')}</td><td class="n">{polygonArea(r.points).toFixed(2).replace('.', ',')}</td></tr>{/each}</tbody>
      </table>
      {#if app.project.works.length}
        <h2>{t('Postes de travaux')}</h2>
        <table><thead><tr><th>{t('Lot')}</th><th>{t('Poste')}</th><th>{t('Artisan')}</th><th>RGE</th><th class="n">{t('Montant')}</th></tr></thead>
          <tbody>{#each app.project.works as w}<tr><td>{w.lot}</td><td>{w.label}</td><td>{w.artisan}</td><td>{w.rge ? t('Oui') : t('Non')}</td><td class="n">{fmtEur(w.amount)}</td></tr>{/each}</tbody>
          <tfoot><tr><td colspan="4">{t('Total')}</td><td class="n">{fmtEur(est.works)}</td></tr></tfoot>
        </table>
      {/if}
    </section>
  </div>
{/if}

<style>
  .dlg { display: grid; gap: 12px; width: min(620px, 92vw); padding: 14px; }
  .hd { margin-bottom: 2px; }
  .warn { color: var(--mark); font-weight: 600; }
  .starts { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; }
  .starts button { display: grid; gap: 4px; align-content: start; min-height: 106px; padding: 10px; border: 1px solid var(--rule); border-radius: 3px; background: var(--panel); text-align: left; }
  .starts button:hover { border-color: var(--ink); }
  .starts span { font-size: 12.5px; color: var(--soft); }
  .keys { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 3px 18px; margin: 0; }
  .keys div { display: flex; justify-content: space-between; gap: 10px; padding: 3px 0; border-bottom: 1px solid var(--rule); }
  dd { margin: 0; text-align: right; color: var(--soft); }
  kbd { font: inherit; font-weight: 650; }
  .end { display: flex; justify-content: flex-end; }
  .pg { page-break-after: always; break-after: page; }
  .pg h1 { font-size: 26pt; }
  .pg h2 { margin: 0 0 4mm; font-size: 13pt; }
  .cover img { width: 100%; max-height: 145mm; object-fit: contain; margin-top: 6mm; }
  .sheet :global(svg) { width: 100%; height: auto; max-height: 170mm; }
  table { width: 100%; margin-bottom: 7mm; border-collapse: collapse; font-size: 10pt; }
  th, td { padding: 1.3mm 2mm 1.3mm 0; border-bottom: 0.2mm solid #bbb; text-align: left; }
  .n { text-align: right; }
  tfoot td { font-weight: 700; }
</style>
