<script lang="ts">
  import { Download, Plus, Trash2 } from '@lucide/svelte';
  import { computeEstimate, fmtEur } from '../model/estimate';
  import type { Funding, WorkItem, WorkState } from '../model/types';
  import { uid } from '../model/types';
  import { app } from '../state/app.svelte';
  import { t } from '../state/i18n.svelte';

  const FUNDING: { id: Funding; label: string; color: string }[] = [
    { id: 'eco-ptz', label: 'Éco-PTZ', color: 'var(--st-new)' },
    { id: 'aid', label: 'Aides', color: '#62b7d3' },
    { id: 'bank', label: 'Prêt banque', color: 'var(--accent)' },
    { id: 'cash', label: 'Apport', color: '#f2a65a' },
    { id: 'none', label: 'À affecter', color: 'var(--rule)' },
  ];
  const STATES: { id: WorkState; label: string }[] = [
    { id: 'idea', label: 'À chiffrer' },
    { id: 'quote', label: 'Devis reçu' },
    { id: 'signed', label: 'Signé' },
    { id: 'done', label: 'Fait' },
  ];
  const e = $derived(computeEstimate(app.project));
  const set = (id: string, f: (w: WorkItem) => void) => app.commit((p) => f(p.works.find((x) => x.id === id)!));
  let openId = $state<string | null>(null);

  function add() {
    const id = uid('p');
    app.commit((p) => void p.works.push({ id, lot: t('Divers'), label: t('Nouveau poste'), amount: 0, rge: false, artisan: '', funding: 'none', state: 'idea' }));
    openId = id;
  }
</script>

<div class="bud stack">
  <section class="stack">
    <div class="sum"><span>{app.project.works.length ? t('Total des postes') : t('Estimation du plan')}</span><b class="num">{fmtEur(e.total)}</b></div>
    {#if e.works > 0}
      <div class="bar" role="img" aria-label={t('Répartition du financement')}>
        {#each FUNDING as f}{#if e.byFunding[f.id] > 0}<i style={`flex:${e.byFunding[f.id]};background:${f.color}`} title={`${t(f.label)} : ${fmtEur(e.byFunding[f.id])}`}></i>{/if}{/each}
      </div>
      <dl>
        {#each FUNDING as f}{#if e.byFunding[f.id] > 0}<div><dt><i style={`background:${f.color}`}></i>{t(f.label)}</dt><dd class="num">{fmtEur(e.byFunding[f.id])}</dd></div>{/if}{/each}
        <div class="sep"><dt>{t('Dont entreprises RGE')}</dt><dd class="num">{fmtEur(e.rge)}</dd></div>
      </dl>
    {/if}
  </section>

  <section class="stack">
    <div class="rowb"><h2>{t('Postes de travaux')}</h2><button class="btn" onclick={add}><Plus size={14} />{t('Ajouter')}</button></div>
    {#if !app.project.works.length}<p class="muted">{t('Ajoutez vos devis ici : ils remplaceront l’estimation calculée depuis le plan.')}</p>{/if}
    <ul>
      {#each app.project.works as w (w.id)}
        <li class:open={openId === w.id}>
          <button class="line" aria-expanded={openId === w.id} onclick={() => (openId = openId === w.id ? null : w.id)}>
            <span><b>{w.label}</b><small>{w.lot}{w.artisan ? ` · ${w.artisan}` : ''}{w.rge ? ' · RGE' : ''}</small></span>
            <span class="num amt">{fmtEur(w.amount)}<small>{t(STATES.find((s) => s.id === w.state)!.label)}</small></span>
          </button>
          {#if openId === w.id}
            <div class="edit stack">
              <label class="field"><span>{t('Intitulé')}</span><input type="text" value={w.label} onchange={(ev) => { const v = ev.currentTarget.value; set(w.id, (x) => (x.label = v)); }} /></label>
              <div class="grid2">
                <label class="field"><span>{t('Lot')}</span><input type="text" value={w.lot} onchange={(ev) => { const v = ev.currentTarget.value; set(w.id, (x) => (x.lot = v)); }} /></label>
                <label class="field"><span>{t('Montant TTC')}</span><input type="number" min="0" value={w.amount} onchange={(ev) => { const v = +ev.currentTarget.value; set(w.id, (x) => (x.amount = Math.max(0, v))); }} /></label>
                <label class="field"><span>{t('Financement')}</span><select value={w.funding} onchange={(ev) => { const v = ev.currentTarget.value as Funding; set(w.id, (x) => (x.funding = v)); }}>{#each FUNDING as f}<option value={f.id}>{t(f.label)}</option>{/each}</select></label>
                <label class="field"><span>{t('Avancement')}</span><select value={w.state} onchange={(ev) => { const v = ev.currentTarget.value as WorkState; set(w.id, (x) => (x.state = v)); }}>{#each STATES as s}<option value={s.id}>{t(s.label)}</option>{/each}</select></label>
              </div>
              <label class="field"><span>{t('Artisan')}</span><input type="text" value={w.artisan} onchange={(ev) => { const v = ev.currentTarget.value; set(w.id, (x) => (x.artisan = v)); }} /></label>
              <div class="rowb">
                <label class="check"><input type="checkbox" checked={w.rge} onchange={(ev) => { const v = ev.currentTarget.checked; set(w.id, (x) => (x.rge = v)); }} />{t('Entreprise RGE exigée')}</label>
                <button class="icon" title={t('Supprimer')} aria-label={t('Supprimer')} onclick={() => app.commit((p) => (p.works = p.works.filter((x) => x.id !== w.id)))}><Trash2 size={15} /></button>
              </div>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  </section>

  <section class="stack">
    <div class="rowb"><h2>{t('Métré du plan')}</h2><span class="num muted">{fmtEur(e.metered)}</span></div>
    {#if e.lines.length}
      <table>
        <tbody>{#each e.lines as l}<tr><td>{t(l.label)}</td><td class="n">{l.quantity.toLocaleString('fr-FR', { maximumFractionDigits: 1 })} {l.unit}</td><td class="n">{fmtEur(l.total)}</td></tr>{/each}</tbody>
      </table>
    {:else}<p class="muted">{t('Marquez des éléments À créer ou À démolir pour obtenir un métré.')}</p>{/if}
    <button class="btn" onclick={() => app.exportCsv()}><Download size={14} />{t('Exporter en CSV')}</button>
  </section>
</div>

<style>
  .bud { padding: 12px; gap: 18px; }
  .sum span { display: block; font-size: 12px; color: var(--soft); }
  .sum b { font-size: 26px; font-weight: 750; letter-spacing: -0.015em; }
  .bar { display: flex; height: 12px; gap: 1px; overflow: hidden; border-radius: 1px; }
  dl { display: grid; gap: 3px; margin: 0; }
  dl div { display: flex; justify-content: space-between; gap: 8px; }
  dt { display: flex; align-items: center; gap: 6px; color: var(--soft); }
  dt i { width: 9px; height: 9px; border-radius: 1px; }
  dd { margin: 0; font-weight: 600; }
  .sep { margin-top: 3px; padding-top: 4px; border-top: 1px solid var(--rule); }
  ul { display: grid; margin: 0; padding: 0; list-style: none; border-top: 1px solid var(--rule); }
  li { border-bottom: 1px solid var(--rule); }
  .line { display: flex; justify-content: space-between; gap: 10px; width: 100%; padding: 7px 2px; border: 0; background: transparent; text-align: left; }
  .line:hover { background: var(--panel-2); }
  .line b { display: block; font-weight: 600; }
  small { display: block; font-size: 11.5px; color: var(--soft); }
  .amt { text-align: right; white-space: nowrap; font-weight: 600; }
  .edit { padding: 6px 2px 12px; }
  table { width: 100%; border-collapse: collapse; font-size: 12.5px; }
  td { padding: 3px 0; border-bottom: 1px solid var(--rule); }
  .n { padding-left: 8px; text-align: right; white-space: nowrap; font-variant-numeric: tabular-nums; }
</style>
