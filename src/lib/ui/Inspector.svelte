<script lang="ts">
  import { Check, Copy, ImagePlus, Plus, Trash2, X } from '@lucide/svelte';
  import { CATALOG, catalogEntry, chipFor, MATERIALS, OPENING_DEFAULTS, ROOM_CHIPS, WALL_DEFAULTS } from '../model/catalog';
  import { fmtArea, fmtLen } from '../model/estimate';
  import { add, bounds, dist, mul, polygonArea, polygonPerimeter, wallDir, wallLength } from '../model/geometry';
  import { clampOffset, levelBounds, moveWallEnd } from '../model/ops';
  import type { Dimension, Item, Note, Opening, OpeningKind, Project, Room, Stair, Status, Wall, WallKind } from '../model/types';
  import { uid } from '../model/types';
  import { app } from '../state/app.svelte';
  import { t } from '../state/i18n.svelte';
  import { pickImage } from '../state/platform';
  import Num from './Num.svelte';

  const STATES: { id: Status; label: string; icon: typeof Check }[] = [
    { id: 'existing', label: 'Existant', icon: Check },
    { id: 'demolish', label: 'À démolir', icon: X },
    { id: 'new', label: 'À créer', icon: Plus },
  ];
  const ref = $derived(app.selection[0]);
  const one = $derived(app.selection.length === 1 ? ref : null);
  const wall = $derived(one?.kind === 'wall' ? app.project.walls.find((e) => e.id === one.id) : undefined);
  const opening = $derived(one?.kind === 'opening' ? app.project.openings.find((e) => e.id === one.id) : undefined);
  const room = $derived(one?.kind === 'room' ? app.project.rooms.find((e) => e.id === one.id) : undefined);
  const item = $derived(one?.kind === 'item' ? app.project.items.find((e) => e.id === one.id) : undefined);
  const stair = $derived(one?.kind === 'stair' ? app.project.stairs.find((e) => e.id === one.id) : undefined);
  const dim = $derived(one?.kind === 'dim' ? app.project.dims.find((e) => e.id === one.id) : undefined);
  const note = $derived(one?.kind === 'note' ? app.project.notes.find((e) => e.id === one.id) : undefined);
  const currentStatus = $derived((wall ?? opening ?? room ?? item ?? stair)?.status);
  const underlay = $derived(app.project.underlays.find((u) => u.level === app.level));
  const level = $derived(app.levelInfo);

  type Lists = { w: Wall; o: Opening; r: Room; i: Item; s: Stair; d: Dimension; n: Note };
  const setW = (f: (e: Wall, p: Project) => void) => app.commit((p) => f(p.walls.find((e) => e.id === wall!.id)!, p));
  const setO = (f: (e: Opening, p: Project) => void) => app.commit((p) => f(p.openings.find((e) => e.id === opening!.id)!, p));
  const setR = (f: (e: Room) => void) => app.commit((p) => f(p.rooms.find((e) => e.id === room!.id)!));
  const setI = (f: (e: Item) => void) => app.commit((p) => f(p.items.find((e) => e.id === item!.id)!));
  const setS = (f: (e: Stair) => void) => app.commit((p) => f(p.stairs.find((e) => e.id === stair!.id)!));
  const setD = (f: (e: Dimension) => void) => app.commit((p) => f(p.dims.find((e) => e.id === dim!.id)!));
  const setN = (f: (e: Note) => void) => app.commit((p) => f(p.notes.find((e) => e.id === note!.id)!));
  const setL = (f: (e: Lists extends never ? never : Project['levels'][number]) => void) => app.commit((p) => f(p.levels.find((e) => e.id === app.level)!));

  function setWallLength(v: number) {
    if (v < 0.05) return;
    setW((w, p) => moveWallEnd(p, w.id, 'b', add(w.a, mul(wallDir(w), v))));
  }

  async function addUnderlay() {
    try {
      const img = await pickImage();
      if (!img) return;
      const pts = levelBounds(app.project, app.level);
      const b = pts.length ? bounds(pts) : { minX: 0, minY: 0, w: 8 };
      app.commit((p) => {
        p.underlays = p.underlays.filter((u) => u.level !== app.level);
        p.underlays.push({ level: app.level, src: img.src, x: b.minX, y: b.minY, width: Math.max(b.w, 8), aspect: img.aspect, opacity: 0.5, visible: true });
      });
      app.setTool('calibrate');
      app.notify(t('Glissez sur une longueur connue pour régler l’échelle.'));
    } catch (e) {
      app.notify(e instanceof Error ? e.message : String(e), 'err');
    }
  }

  function addRoof() {
    const pts = levelBounds(app.project, app.level);
    if (!pts.length) return app.notify(t('Dessinez d’abord des murs à ce niveau.'), 'err');
    const b = bounds(pts);
    app.commit((p) => void p.roofs.push({ id: uid('t'), level: app.level, x: b.minX, y: b.minY, w: b.w, d: b.h, kind: 'gable', ridge: b.w >= b.h ? 'x' : 'y', rise: 1.8, overhang: 0.3, color: '#a8583c' }));
  }
  const roofs = $derived(app.project.roofs.filter((r) => r.level === app.level));
  const areaGap = $derived(room?.referenceArea ? polygonArea(room.points) - room.referenceArea : 0);
</script>

<div class="insp stack">
  {#if !app.selection.length}
    <section class="stack">
      <h2>{t('Projet')}</h2>
      <div class="grid2">
        <Num label={t('Nord')} mode="raw" unit="°" min={-360} value={app.project.north} onchange={(v) => app.commit((p) => (p.north = v))} />
        <label class="field"><span>{t('Cotes en')}</span>
          <select value={app.project.dimUnit} onchange={(e) => { const v = e.currentTarget.value as 'm' | 'cm'; app.commit((p) => (p.dimUnit = v)); }}><option value="m">{t('mètres')}</option><option value="cm">{t('centimètres')}</option></select>
        </label>
      </div>
    </section>
    <section class="stack">
      <h2>{t('Niveau')} : {level.name}</h2>
      <label class="field"><span>{t('Nom')}</span><input type="text" value={level.name} onchange={(e) => { const v = e.currentTarget.value; setL((l) => (l.name = v)); }} /></label>
      <div class="grid3">
        <Num label={t('Hauteur')} value={level.height} onchange={(v) => setL((l) => (l.height = v))} />
        <Num label={t('Altitude')} value={level.elevation} min={-20} onchange={(v) => setL((l) => (l.elevation = v))} />
        <Num label={t('Dalle')} value={level.slab} onchange={(v) => setL((l) => (l.slab = v))} />
      </div>
    </section>
    <section class="stack">
      <div class="rowb"><h2>{t('Calque de fond')}</h2><button class="btn" onclick={addUnderlay}><ImagePlus size={14} />{underlay ? t('Remplacer') : t('Importer')}</button></div>
      {#if underlay}
        <label class="field"><span>{t('Opacité')}</span><input type="range" min="0.1" max="1" step="0.05" value={underlay.opacity} onchange={(e) => { const v = +e.currentTarget.value; app.commit((p) => (p.underlays.find((u) => u.level === app.level)!.opacity = v)); }} /></label>
        <div class="grid2">
          <button class="btn" onclick={() => app.setTool('calibrate')}>{t('Régler l’échelle')}</button>
          <button class="btn danger" onclick={() => app.commit((p) => (p.underlays = p.underlays.filter((u) => u.level !== app.level)))}>{t('Retirer')}</button>
        </div>
      {:else}<p class="muted">{t('Importez une photo de croquis ou un plan, puis redessinez par-dessus.')}</p>{/if}
    </section>
    <section class="stack">
      <div class="rowb"><h2>{t('Toiture')}</h2><button class="btn" onclick={addRoof}><Plus size={14} />{t('Ajouter')}</button></div>
      {#each roofs as r}
        {@const set = (f: (e: Project['roofs'][number]) => void) => app.commit((p) => f(p.roofs.find((x) => x.id === r.id)!))}
        <div class="grid3">
          <label class="field"><span>{t('Forme')}</span><select value={r.kind} onchange={(e) => { const v = e.currentTarget.value as 'gable' | 'flat'; set((x) => (x.kind = v)); }}><option value="gable">{t('2 pans')}</option><option value="flat">{t('Plate')}</option></select></label>
          <Num label={t('Faîtage')} value={r.rise} onchange={(v) => set((x) => (x.rise = v))} />
          <Num label={t('Débord')} value={r.overhang} onchange={(v) => set((x) => (x.overhang = v))} />
        </div>
        <button class="btn danger" onclick={() => app.commit((p) => (p.roofs = p.roofs.filter((x) => x.id !== r.id)))}><Trash2 size={14} />{t('Retirer la toiture')}</button>
      {/each}
    </section>
  {:else}
    <div class="rowb head">
      <h2>{#if wall}{t(WALL_DEFAULTS[wall.kind].label)}{:else if opening}{t(OPENING_DEFAULTS[opening.kind].label)}{:else if room}{room.name}{:else if item}{item.label || t(catalogEntry(item.kind).label)}{:else if stair}{t('Escalier')}{:else if dim}{t('Cote')}{:else if note}{t('Note')}{:else}{app.selection.length} {t('éléments')}{/if}</h2>
      <div>
        <button class="icon" title={`${t('Dupliquer')} (Ctrl+D)`} aria-label={t('Dupliquer')} onclick={() => app.duplicateSelection()}><Copy size={15} /></button>
        <button class="icon" title={`${t('Supprimer')} (Suppr)`} aria-label={t('Supprimer')} onclick={() => app.deleteSelection()}><Trash2 size={15} /></button>
      </div>
    </div>

    {#if !dim && !note}
      <div class="seg full" role="group" aria-label={t('État chantier')}>
        {#each STATES as s}
          {@const Icon = s.icon}
          <button class={s.id} aria-pressed={currentStatus === s.id} onclick={() => app.statusSelection(s.id)}><Icon size={13} />{t(s.label)}</button>
        {/each}
      </div>
    {/if}

    {#if wall}
      <label class="field"><span>{t('Type')}</span><select value={wall.kind} onchange={(e) => { const v = e.currentTarget.value as WallKind; setW((w) => { w.kind = v; w.thickness = WALL_DEFAULTS[v].thickness; }); }}>{#each Object.entries(WALL_DEFAULTS) as [id, d]}<option value={id}>{t(d.label)}</option>{/each}</select></label>
      <div class="grid3">
        <Num label={t('Longueur')} value={wallLength(wall)} onchange={setWallLength} />
        <Num label={t('Épaisseur')} value={wall.thickness} onchange={(v) => setW((w) => (w.thickness = v))} />
        <Num label={t('Hauteur')} value={wall.height ?? level.height} onchange={(v) => setW((w) => (w.height = v))} />
      </div>
    {:else if opening}
      <label class="field"><span>{t('Type')}</span><select value={opening.kind} onchange={(e) => { const v = e.currentTarget.value as OpeningKind; const d = OPENING_DEFAULTS[v]; setO((o) => { o.kind = v; o.width = d.width; o.height = d.height; o.sill = d.sill; }); }}>{#each Object.entries(OPENING_DEFAULTS) as [id, d]}<option value={id}>{t(d.label)}</option>{/each}</select></label>
      <div class="grid2">
        <Num label={t('Largeur')} value={opening.width} onchange={(v) => setO((o, p) => { o.width = v; o.offset = clampOffset(p.walls.find((w) => w.id === o.wall)!, o.offset, v); })} />
        <Num label={t('Hauteur')} value={opening.height} onchange={(v) => setO((o) => (o.height = v))} />
        <Num label={t('Allège')} value={opening.sill} onchange={(v) => setO((o) => (o.sill = v))} />
        <Num label={t('Position')} value={opening.offset} onchange={(v) => setO((o, p) => (o.offset = clampOffset(p.walls.find((w) => w.id === o.wall)!, v, o.width)))} />
      </div>
      <div class="grid2">
        <button class="btn" onclick={() => setO((o) => (o.flip = !o.flip))}>{t('Inverser le côté')}</button>
        <button class="btn" onclick={() => setO((o) => (o.hinge = !o.hinge))}>{t('Inverser les gonds')}</button>
      </div>
      <label class="field"><span>{t('Repère sur le plan')}</span><input type="text" maxlength="3" value={opening.tag ?? ''} onchange={(e) => { const v = e.currentTarget.value.trim(); setO((o) => (o.tag = v || undefined)); }} /></label>
      <label class="field"><span>{t('Note pour le devis')}</span><textarea value={opening.note ?? ''} onchange={(e) => { const v = e.currentTarget.value; setO((o) => (o.note = v || undefined)); }}></textarea></label>
    {:else if room}
      <label class="field"><span>{t('Nom')}</span><input type="text" value={room.name} onchange={(e) => { const v = e.currentTarget.value; setR((r) => (r.name = v)); }} /></label>
      <dl class="facts"><div><dt>{t('Surface')}</dt><dd>{fmtArea(polygonArea(room.points))}</dd></div><div><dt>{t('Périmètre')}</dt><dd>{fmtLen(polygonPerimeter(room.points))}</dd></div></dl>
      <div class="grid2">
        <Num label={t('Plafond')} value={room.ceiling ?? level.height} onchange={(v) => setR((r) => (r.ceiling = v))} />
        <Num label={t('Surface de référence')} mode="raw" unit="m²" step={0.01} value={room.referenceArea ?? 0} onchange={(v) => setR((r) => (r.referenceArea = v || undefined))} />
      </div>
      {#if room.referenceArea}<p class:warn={Math.abs(areaGap) > 0.5} class="muted">{t('Écart avec la référence')} : {areaGap >= 0 ? '+' : ''}{areaGap.toFixed(2).replace('.', ',')} m²</p>{/if}
      <div class="field"><span>{t('Teinte')} <small>{chipFor(room.color)?.code ?? ''} {chipFor(room.color)?.name ?? ''}</small></span>
        <div class="chips">{#each ROOM_CHIPS as c}<button style={`background:${c.color}`} title={`${c.code} ${c.name}`} aria-label={c.name} aria-pressed={room.color === c.color} onclick={() => setR((r) => (r.color = c.color))}></button>{/each}</div>
      </div>
      <div class="field"><span>{t('Sol')}</span>
        <div class="deck">{#each MATERIALS as m}<button aria-pressed={room.material === m.id} onclick={() => setR((r) => (r.material = m.id))}><i style={`background:${m.color}`}></i><b>{t(m.label)}</b><small>{m.code}</small></button>{/each}</div>
      </div>
    {:else if item}
      <label class="field"><span>{t('Modèle')}</span><select value={item.kind} onchange={(e) => { const c = catalogEntry(e.currentTarget.value); setI((i) => { i.kind = c.kind; i.w = c.w; i.d = c.d; i.h = c.h; i.color = c.color; i.cost = c.cost; }); }}>{#each CATALOG as c}<option value={c.kind}>{t(c.label)}</option>{/each}</select></label>
      <label class="field"><span>{t('Libellé')}</span><input type="text" value={item.label ?? ''} onchange={(e) => { const v = e.currentTarget.value; setI((i) => (i.label = v || undefined)); }} /></label>
      <div class="grid3">
        <Num label={t('Largeur')} value={item.w} onchange={(v) => setI((i) => (i.w = v))} />
        <Num label={t('Profondeur')} value={item.d} onchange={(v) => setI((i) => (i.d = v))} />
        <Num label={t('Hauteur')} value={item.h} onchange={(v) => setI((i) => (i.h = v))} />
      </div>
      <div class="grid3">
        <Num label={t('Rotation')} mode="raw" unit="°" min={-360} step={15} value={item.rotation} onchange={(v) => setI((i) => (i.rotation = v))} />
        <Num label={t('Prix')} mode="raw" unit="€" value={item.cost ?? 0} onchange={(v) => setI((i) => (i.cost = v))} />
        <label class="field"><span>{t('Couleur')}</span><input class="clr" type="color" value={item.color ?? '#bbbbbb'} onchange={(e) => { const v = e.currentTarget.value; setI((i) => (i.color = v)); }} /></label>
      </div>
    {:else if stair}
      <label class="field"><span>{t('Forme')}</span><select value={stair.shape} onchange={(e) => { const v = e.currentTarget.value as Stair['shape']; setS((s) => (s.shape = v)); }}><option value="straight">{t('Droit')}</option><option value="quarter">{t('Quart tournant')}</option><option value="half">{t('Demi-tournant')}</option></select></label>
      <div class="grid2">
        <Num label={t('Longueur')} value={stair.w} onchange={(v) => setS((s) => (s.w = v))} />
        <Num label={t('Largeur')} value={stair.d} onchange={(v) => setS((s) => (s.d = v))} />
        <Num label={t('Marches')} mode="raw" min={3} value={stair.steps} onchange={(v) => setS((s) => (s.steps = Math.round(v)))} />
        <Num label={t('Rotation')} mode="raw" unit="°" min={-360} step={15} value={stair.rotation} onchange={(v) => setS((s) => (s.rotation = v))} />
      </div>
      <button class="btn" onclick={() => setS((s) => (s.flip = !s.flip))}>{t('Inverser le sens')}</button>
    {:else if dim}
      <dl class="facts"><div><dt>{t('Mesure réelle')}</dt><dd>{fmtLen(dist(dim.a, dim.b))}</dd></div></dl>
      <label class="field"><span>{t('Texte affiché')}</span><input type="text" placeholder={t('Automatique')} value={dim.label ?? ''} onchange={(e) => { const v = e.currentTarget.value.trim(); setD((d) => (d.label = v || undefined)); }} /></label>
      <label class="check"><input type="checkbox" checked={!!dim.flag} onchange={(e) => { const v = e.currentTarget.checked; setD((d) => (d.flag = v)); }} />{t('Cote à vérifier')}</label>
    {:else if note}
      <label class="field"><span>{t('Texte')}</span><textarea value={note.text} onchange={(e) => { const v = e.currentTarget.value; setN((n) => (n.text = v)); }}></textarea></label>
      <label class="check"><input type="checkbox" checked={!!note.flag} onchange={(e) => { const v = e.currentTarget.checked; setN((n) => (n.flag = v)); }} />{t('À vérifier')}</label>
    {/if}
  {/if}
</div>

<style>
  .insp { padding: 12px; gap: 14px; }
  section + section { padding-top: 12px; border-top: 1px solid var(--rule); }
  .head { min-height: 30px; }
  .seg .demolish[aria-pressed='true'] { color: var(--st-demolish); }
  .seg .new[aria-pressed='true'] { color: var(--st-new); }
  .facts { display: flex; gap: 22px; margin: 0; }
  .facts dt { font-size: 12px; color: var(--soft); }
  .facts dd { margin: 0; font-size: 15px; font-weight: 650; font-variant-numeric: tabular-nums; }
  .warn { color: var(--mark); font-weight: 600; }
  small { font-size: 11px; color: var(--soft); }
  .chips { display: grid; grid-template-columns: repeat(6, 1fr); gap: 3px; }
  .chips button { height: 26px; border: 0; border-radius: 2px; }
  .chips button[aria-pressed='true'] { outline: 2px solid var(--ink); outline-offset: 1px; }
  .deck { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 4px; }
  .deck button { display: grid; grid-template-columns: 22px 1fr; column-gap: 7px; align-items: center; padding: 4px; border: 1px solid var(--rule); border-radius: 3px; background: var(--panel); text-align: left; }
  .deck button:hover { border-color: var(--soft); }
  .deck button[aria-pressed='true'] { border-color: var(--ink); box-shadow: inset 0 0 0 1px var(--ink); }
  .deck i { grid-row: span 2; width: 22px; height: 28px; border-radius: 1px; }
  .deck b { font-size: 12px; font-weight: 550; line-height: 1.1; }
  .deck small { font-size: 10.5px; }
  .clr { padding: 2px !important; }
</style>
