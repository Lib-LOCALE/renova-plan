<script lang="ts">
  import { Box, Check, Columns2, Download, FilePlus2, FileText, FolderOpen, Languages, CircleHelp, Map as MapIcon, Menu, Plus, Redo2, Ruler, Save, Undo2, Wallet, X, LayoutGrid, Armchair, Palette } from '@lucide/svelte';
  import logo from '../icons/icon-static.svg?url';
  import { computeEstimate, fmtArea, fmtEur } from './lib/model/estimate';
  import type { Phase } from './lib/model/types';
  import { app, type Panel, type Theme } from './lib/state/app.svelte';
  import { editor } from './lib/state/editor.svelte';
  import { i18n, t } from './lib/state/i18n.svelte';
  import BudgetPanel from './lib/ui/BudgetPanel.svelte';
  import BuildBar from './lib/ui/BuildBar.svelte';
  import Dialogs from './lib/ui/Dialogs.svelte';
  import Inspector from './lib/ui/Inspector.svelte';
  import PlanView from './lib/ui/PlanView.svelte';
  import RoomsPanel from './lib/ui/RoomsPanel.svelte';
  import View3D from './lib/ui/View3D.svelte';

  const PHASES: { id: Phase; label: string }[] = [
    { id: 'before', label: 'Avant' },
    { id: 'site', label: 'Chantier' },
    { id: 'after', label: 'Après' },
  ];
  const THEMES: { id: Theme; label: string }[] = [
    { id: 'light', label: 'Clair' },
    { id: 'dark', label: 'Sombre' },
    { id: 'dracula', label: 'Dracula' },
  ];
  let split = $state(0.5);
  let menu = $state(false);
  let calibration = $state('');
  const est = $derived(computeEstimate(app.project));
  const levelsDesc = $derived([...app.project.levels].reverse());
  const empty = $derived(!app.project.walls.some((w) => w.level === app.level));
  const hint = $derived.by(() => {
    const p = editor.placing;
    if (p?.type === 'opening') return 'Cliquez sur un mur pour la poser · Échap pour terminer';
    if (p?.type === 'room') return 'Cliquez pour poser la pièce · elle s’accroche aux murs voisins';
    if (p?.type === 'floor' || p?.type === 'color') return 'Cliquez sur les pièces à changer · Échap pour terminer';
    if (p) return 'Cliquez pour poser · R pour tourner · Échap pour annuler';
    if (app.tool === 'wall') return editor.drafting ? 'Relâchez ou cliquez pour finir · tapez une longueur si besoin' : 'Appuyez et glissez pour tracer le mur';
    if (app.tool === 'room') return 'Appuyez et glissez pour tracer la pièce';
    if (app.tool === 'erase') return 'Cliquez sur ce que vous voulez retirer · Échap pour arrêter';
    if (app.tool === 'measure' || app.tool === 'dim' || app.tool === 'calibrate') return 'Appuyez et glissez entre deux points';
    if (app.tool === 'detect') return 'Cliquez dans un espace fermé par des murs';
    if (app.tool === 'note') return 'Cliquez où poser la note';
    return empty ? 'Pour commencer, faites glisser une pièce depuis le bas de l’écran' : '';
  });

  $effect(() => {
    document.documentElement.dataset.theme = app.theme;
    document.documentElement.lang = i18n.lang;
    document.title = `${app.dirty ? '• ' : ''}${app.project.name} — RenovaPlan`;
  });

  function setLayout(v: typeof app.layout) {
    app.layout = v;
    app.savePrefs();
  }
  function togglePanel(p: Panel) {
    app.panel = app.panel === p ? null : p;
  }
  function startSplit(e: PointerEvent) {
    const box = (e.currentTarget as HTMLElement).parentElement!.getBoundingClientRect();
    const mv = (m: PointerEvent) => (split = Math.max(0.25, Math.min(0.75, (m.clientX - box.left) / box.width)));
    const up = () => {
      window.removeEventListener('pointermove', mv);
      window.removeEventListener('pointerup', up);
    };
    window.addEventListener('pointermove', mv);
    window.addEventListener('pointerup', up);
  }
  function onKey(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    if (target.closest('input,textarea,select,dialog')) {
      if (e.key === 'Escape') target.blur?.();
      return;
    }
    if (editor.key(e)) {
      e.preventDefault();
      return;
    }
    const k = e.key.toLowerCase();
    if (e.ctrlKey || e.metaKey) {
      const run: Record<string, () => void> = { z: () => (e.shiftKey ? app.redo() : app.undo()), y: () => app.redo(), s: () => app.save(e.shiftKey), o: () => app.open(), d: () => app.duplicateSelection(), n: () => (app.dialog = 'new') };
      if (run[k]) {
        e.preventDefault();
        run[k]();
      }
      return;
    }
    if (e.key === 'Delete' || e.key === 'Backspace') editor.eraseSelection();
    else if (e.key === 'F1' || e.key === '?') {
      e.preventDefault();
      app.dialog = 'help';
    } else if (k === '1') setLayout('3d');
    else if (k === '2') setLayout('plan');
    else if (k === '3') setLayout('split');
  }
  function addLevel() {
    app.commit((p) => {
      const last = p.levels.at(-1)!;
      const id = `l${Date.now().toString(36)}`;
      p.levels.push({ id, name: `${t('Niveau')} ${p.levels.length}`, elevation: last.elevation + last.height + last.slab, height: last.height, slab: last.slab });
      queueMicrotask(() => (app.level = id));
    });
  }
  function closeAnd(fn: () => void) {
    menu = false;
    fn();
  }
</script>

<svelte:window onkeydown={onKey} onclick={() => (menu = false)} />

<div class="shell" class:drawer={!!app.panel}>
  <header class="top">
    <div class="brand"><img src={logo} alt="" width="26" height="26" /><input aria-label={t('Nom du projet')} value={app.project.name} onchange={(e) => { const v = e.currentTarget.value.trim(); if (v) app.commit((p) => (p.name = v)); }} /></div>
    <div class="group">
      <button class="tb" disabled={!app.past.length} onclick={() => app.undo()} title="Ctrl+Z"><Undo2 size={19} /><span>{t('Annuler')}</span></button>
      <button class="tb" disabled={!app.future.length} onclick={() => app.redo()} title="Ctrl+Y"><Redo2 size={19} /><span>{t('Rétablir')}</span></button>
    </div>
    <div class="switch" role="group" aria-label={t('Affichage')}>
      <button aria-pressed={app.layout === '3d'} onclick={() => setLayout('3d')}><Box size={17} />3D</button>
      <button aria-pressed={app.layout === 'plan'} onclick={() => setLayout('plan')}><MapIcon size={17} />{t('Plan')}</button>
      <button aria-pressed={app.layout === 'split'} onclick={() => setLayout('split')}><Columns2 size={17} />{t('Les deux')}</button>
    </div>
    <div class="switch" role="group" aria-label={t('Travaux')}>
      {#each PHASES as ph}<button class={ph.id} aria-pressed={app.phase === ph.id} onclick={() => app.setPhase(ph.id)}><i></i>{t(ph.label)}</button>{/each}
    </div>
    <div class="group right">
      <span class="sum">{fmtArea(est.carrezArea)} · {fmtEur(est.total)}</span>
      <button class="tb" aria-pressed={app.panel === 'rooms'} onclick={() => togglePanel('rooms')}><LayoutGrid size={19} /><span>{t('Pièces')}</span></button>
      <button class="tb" aria-pressed={app.panel === 'budget'} onclick={() => togglePanel('budget')}><Wallet size={19} /><span>{t('Budget')}</span></button>
      <button class="tb save" onclick={() => app.save()}><Save size={19} /><span>{t('Enregistrer')}</span>{#if app.dirty}<em></em>{/if}</button>
      <div class="menu-wrap">
        <button class="tb" aria-expanded={menu} aria-label={t('Menu')} onclick={(e) => { e.stopPropagation(); menu = !menu; }}><Menu size={21} /></button>
        {#if menu}
          <div class="menu" role="menu" onclick={(e) => e.stopPropagation()} onkeydown={() => {}} tabindex="-1">
            <button onclick={() => closeAnd(() => (app.dialog = 'new'))}><FilePlus2 size={17} />{t('Nouveau projet')}</button>
            <button onclick={() => closeAnd(() => app.open())}><FolderOpen size={17} />{t('Ouvrir un projet')}</button>
            <hr />
            <button onclick={() => closeAnd(() => app.exportSvg())}><Download size={17} />{t('Exporter le plan')}</button>
            <button onclick={() => closeAnd(() => app.exportPng())}><Download size={17} />{t('Exporter une image 3D')}</button>
            <button onclick={() => closeAnd(() => app.exportCsv())}><Download size={17} />{t('Exporter le budget')}</button>
            <button onclick={() => closeAnd(() => (app.dialog = 'report'))}><FileText size={17} />{t('Dossier pour artisans')}</button>
            <hr />
            <div class="themes"><Palette size={17} />{#each THEMES as th}<button aria-pressed={app.theme === th.id} onclick={() => { app.theme = th.id; app.savePrefs(); }}>{t(th.label)}</button>{/each}</div>
            <button onclick={() => i18n.toggle()}><Languages size={17} />{i18n.lang === 'fr' ? 'English' : 'Français'}</button>
            <button onclick={() => closeAnd(() => (app.dialog = 'help'))}><CircleHelp size={17} />{t('Aide')}</button>
          </div>
        {/if}
      </div>
    </div>
  </header>

  <main class="work">
    <div class="views" style={app.layout === 'split' ? `--split:${split}` : ''} data-layout={app.layout}>
      {#if app.layout !== '3d'}<section class="pane plan-pane"><PlanView /></section>{/if}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      {#if app.layout === 'split'}<div class="gutter" data-ui onpointerdown={startSplit}></div>{/if}
      {#if app.layout !== 'plan'}<section class="pane"><View3D /></section>{/if}
    </div>
    <div class="levels" data-ui role="group" aria-label={t('Niveaux')}>
      <button class="add" onclick={addLevel}><Plus size={15} />{t('Ajouter un étage')}</button>
      {#each levelsDesc as l}<button aria-pressed={app.level === l.id} onclick={() => { editor.cancel(); app.level = l.id; app.selection = []; }}>{l.name}</button>{/each}
    </div>
    <div class="shows" data-ui>
      <button aria-pressed={app.showFurniture} onclick={() => (app.showFurniture = !app.showFurniture)}><Armchair size={16} />{t('Meubles')}</button>
      {#if app.layout !== '3d'}<button aria-pressed={app.showDims} onclick={() => (app.showDims = !app.showDims)}><Ruler size={16} />{t('Cotes')}</button>{/if}
    </div>
    {#if hint}<div class="hint" role="status">{t(hint)}</div>{/if}
    {#if editor.calibration}
      <form class="calib" data-ui onsubmit={(e) => { e.preventDefault(); editor.applyCalibration(calibration); calibration = ''; }}>
        <label for="calib-v">{t('Longueur réelle (m)')}</label>
        <input id="calib-v" inputmode="decimal" bind:value={calibration} />
        <button class="btn primary">{t('Appliquer')}</button>
      </form>
    {/if}
  </main>

  {#if app.panel}
    <aside class="side" aria-label={t('Panneau')}>
      <div class="side-head">
        <h2>{app.panel === 'rooms' ? t('Pièces') : app.panel === 'budget' ? t('Budget') : app.selection.length ? t('Détails') : t('Réglages')}</h2>
        <button class="icon" aria-label={t('Fermer')} onclick={() => (app.panel = null)}><X size={19} /></button>
      </div>
      <div class="side-body">{#if app.panel === 'rooms'}<RoomsPanel />{:else if app.panel === 'props'}<Inspector />{:else}<BudgetPanel />{/if}</div>
    </aside>
  {/if}

  <BuildBar />
  {#if app.toast}<div class="toast" class:err={app.toast.kind === 'err'} role="status">{#if app.toast.kind === 'ok'}<Check size={16} />{/if}{app.toast.text}</div>{/if}
</div>
<Dialogs />

<style>
  .shell { position: relative; display: grid; height: 100%; grid-template: 56px minmax(0, 1fr) 156px / minmax(0, 1fr); grid-template-areas: 'top' 'work' 'build'; }
  .shell.drawer .toast { left: calc(50% - 170px); }
  .shell.drawer { grid-template-columns: minmax(0, 1fr) 340px; grid-template-areas: 'top top' 'work side' 'build side'; }
  .top { grid-area: top; display: flex; align-items: center; gap: 12px; padding: 0 12px; background: var(--shell); color: var(--shell-ink); min-width: 0; }
  .brand { display: flex; align-items: center; gap: 8px; min-width: 0; }
  .brand input { width: 170px; min-height: 34px; background: transparent; border-color: transparent; color: inherit; font-size: 15px; font-weight: 650; }
  .brand input:hover, .brand input:focus { background: var(--shell-2); }
  .group { display: flex; align-items: center; gap: 2px; }
  .right { margin-left: auto; }
  .tb { position: relative; display: inline-flex; align-items: center; gap: 6px; height: 38px; padding: 0 9px; border: 0; border-radius: 6px; background: transparent; color: inherit; font-size: 14.5px; }
  .tb:hover:not(:disabled), .tb[aria-pressed='true'] { background: var(--shell-2); }
  .save { font-weight: 650; }
  .tb em { position: absolute; top: 6px; left: 22px; width: 7px; height: 7px; border-radius: 50%; background: #ffb454; }
  .sum { margin-right: 6px; font-size: 14px; color: var(--shell-soft); white-space: nowrap; }
  .switch { display: inline-flex; gap: 2px; padding: 3px; border-radius: 7px; background: var(--shell-2); }
  .switch button { display: inline-flex; align-items: center; gap: 6px; height: 32px; padding: 0 11px; border: 0; border-radius: 5px; background: transparent; color: var(--shell-soft); font-size: 14.5px; white-space: nowrap; }
  .switch button:hover { color: var(--shell-ink); }
  .switch button[aria-pressed='true'] { background: var(--panel); color: var(--ink); font-weight: 650; }
  .switch i { width: 10px; height: 10px; border-radius: 2px; background: #8f9caa; }
  .switch .site i { background: linear-gradient(135deg, #e0564d 50%, #2fa578 50%); }
  .switch .after i { background: #2fa578; }
  .menu-wrap { position: relative; }
  .menu { position: absolute; top: 44px; right: 0; z-index: 60; display: grid; gap: 1px; width: 250px; padding: 6px; color: var(--ink); background: var(--panel); border: 1px solid var(--rule); border-radius: 8px; box-shadow: var(--shadow); }
  .menu > button { display: flex; align-items: center; gap: 9px; min-height: 36px; padding: 0 8px; border: 0; border-radius: 5px; background: transparent; text-align: left; }
  .menu > button:hover { background: var(--panel-2); }
  .menu hr { width: 100%; margin: 4px 0; border: 0; border-top: 1px solid var(--rule); }
  .themes { display: flex; align-items: center; gap: 3px; padding: 3px 8px; }
  .themes button { padding: 4px 7px; border: 0; border-radius: 4px; background: transparent; font-size: 13px; }
  .themes button[aria-pressed='true'] { background: var(--ink); color: var(--panel); }
  .work { grid-area: work; position: relative; min-width: 0; min-height: 0; }
  .views { position: absolute; inset: 0; display: flex; }
  .pane { position: relative; flex: 1 1 0; min-width: 0; }
  [data-layout='split'] .plan-pane { flex: 0 0 calc(var(--split) * 100%); }
  .gutter { flex: 0 0 6px; background: var(--rule); cursor: col-resize; }
  .levels { position: absolute; top: 10px; left: 10px; z-index: 15; display: grid; gap: 2px; padding: 4px; background: var(--panel); border: 1px solid var(--rule); border-radius: 8px; box-shadow: 0 1px 6px rgb(20 32 48 / 0.14); }
  .levels button { min-height: 34px; padding: 0 11px; border: 0; border-radius: 5px; background: transparent; text-align: left; }
  .levels button:hover { background: var(--panel-2); }
  .levels button[aria-pressed='true'] { background: var(--ink); color: var(--panel); font-weight: 650; }
  .levels .add { display: flex; align-items: center; gap: 5px; color: var(--soft); font-size: 13.5px; }
  .shows { position: absolute; top: 10px; right: 12px; z-index: 15; display: flex; gap: 4px; }
  .shows button { display: inline-flex; align-items: center; gap: 5px; height: 36px; padding: 0 9px; border: 1px solid var(--rule); border-radius: 6px; background: var(--panel); color: var(--soft); }
  .shows button[aria-pressed='true'] { color: var(--ink); font-weight: 600; }
  .hint { position: absolute; left: 50%; bottom: 14px; z-index: 12; transform: translateX(-50%); max-width: 80%; padding: 8px 15px; color: var(--panel); background: var(--ink); border-radius: 20px; font-size: 15px; font-weight: 550; pointer-events: none; white-space: nowrap; }
  .calib { position: absolute; top: 58px; left: 50%; z-index: 30; display: flex; align-items: center; gap: 8px; padding: 9px 11px; transform: translateX(-50%); background: var(--panel); border: 1px solid var(--rule); border-radius: 8px; box-shadow: var(--shadow); }
  .calib input { width: 90px; }
  .side { grid-area: side; display: flex; flex-direction: column; min-height: 0; background: var(--panel); border-left: 1px solid var(--rule); }
  .side-head { display: flex; align-items: center; justify-content: space-between; min-height: 48px; padding: 0 8px 0 14px; border-bottom: 1px solid var(--rule); }
  .side-head h2 { font-size: 17px; }
  .side-body { flex: 1; min-height: 0; overflow: auto; }
  .toast { position: absolute; left: 50%; bottom: 176px; z-index: 70; display: flex; align-items: center; gap: 7px; max-width: 70vw; padding: 9px 14px; color: var(--panel); background: var(--ink); border-radius: 6px; box-shadow: var(--shadow); transform: translateX(-50%); }
  .toast.err { background: var(--st-demolish); color: #fff; }
  @media (max-width: 1320px) { .sum { display: none; } .group .tb span { display: none; } .right .save span { display: inline; } }
  @media (max-width: 1040px) { .brand input { width: 110px; } .switch button { padding: 0 7px; } }
</style>
