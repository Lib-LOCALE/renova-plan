<script lang="ts">
  import { onMount } from 'svelte';
  import { Armchair, Bath, BedDouble, BrickWall, CookingPot, DoorOpen, Hammer, Lamp, Layers, MousePointer2, MoveHorizontal, PaintRoller, Pencil, Plug, Ruler, ScanSearch, Square, StickyNote, AppWindow } from '@lucide/svelte';
  import { CATALOG, MATERIALS, OPENING_DEFAULTS, ROOM_CHIPS, type CatalogEntry } from '../model/catalog';
  import { ROOM_PRESETS } from '../model/place';
  import type { OpeningKind, WallKind } from '../model/types';
  import { app, type Tool } from '../state/app.svelte';
  import { editor, type Placement } from '../state/editor.svelte';
  import { t } from '../state/i18n.svelte';
  import { surfaceTexture } from '../three/textures';
  import { renderThumbs } from '../three/thumbs';

  type Tile = { id: string; label: string; note?: string; thumb?: string; icon?: typeof Armchair; swatch?: string; place?: Placement; tool?: Tool; wall?: WallKind; run?: () => void };
  const items = (g: CatalogEntry['group']): Tile[] => CATALOG.filter((c) => c.group === g).map((c) => ({ id: c.kind, label: c.label, thumb: `item:${c.kind}`, place: { type: 'item', kind: c.kind } }));
  const openings = (kinds: OpeningKind[]): Tile[] => kinds.map((k) => ({ id: k, label: OPENING_DEFAULTS[k].label, note: `${Math.round(OPENING_DEFAULTS[k].width * 100)} cm`, thumb: `opening:${k}`, place: { type: 'opening', kind: k } }));
  const material = (id: string) => {
    try {
      return ((surfaceTexture(id).image as HTMLCanvasElement).toDataURL?.('image/jpeg', 0.7)) ?? '';
    } catch {
      return '';
    }
  };
  const TABS: { id: string; label: string; icon: typeof Armchair; tiles: () => Tile[] }[] = [
    {
      id: 'build', label: 'Pièces et murs', icon: BrickWall, tiles: () => [
        { id: 'draw-room', label: 'Pièce libre', note: 'Glisser', icon: Square, tool: 'room' },
        { id: 'wall', label: 'Cloison', note: 'Glisser', icon: Pencil, tool: 'wall', wall: 'partition' },
        { id: 'ext', label: 'Mur extérieur', note: 'Glisser', icon: BrickWall, tool: 'wall', wall: 'exterior' },
        ...ROOM_PRESETS.map((r): Tile => ({ id: r.id, label: r.name, note: `${r.w} × ${r.h} m`.replaceAll('.', ','), thumb: `room:${r.id}`, place: { type: 'room', preset: r } })),
        { id: 's1', label: 'Escalier droit', thumb: 'stair:straight', place: { type: 'stair', shape: 'straight' } },
        { id: 's2', label: 'Escalier tournant', thumb: 'stair:quarter', place: { type: 'stair', shape: 'quarter' } },
        { id: 's3', label: 'Escalier en U', thumb: 'stair:half', place: { type: 'stair', shape: 'half' } },
      ],
    },
    { id: 'doors', label: 'Portes', icon: DoorOpen, tiles: () => openings(['door', 'double-door', 'sliding-door', 'passage', 'entry-door', 'garage-door']) },
    { id: 'windows', label: 'Fenêtres', icon: AppWindow, tiles: () => openings(['window', 'french-window', 'bay-window']) },
    { id: 'living', label: 'Séjour', icon: Armchair, tiles: () => items('Séjour') },
    { id: 'kitchen', label: 'Cuisine', icon: CookingPot, tiles: () => items('Cuisine') },
    { id: 'bedroom', label: 'Chambre', icon: BedDouble, tiles: () => items('Chambre') },
    { id: 'bath', label: 'Bain', icon: Bath, tiles: () => items('Bain') },
    { id: 'office', label: 'Bureau', icon: Lamp, tiles: () => items('Bureau') },
    { id: 'tech', label: 'Technique', icon: Plug, tiles: () => items('Technique') },
    {
      id: 'paint', label: 'Sols et couleurs', icon: PaintRoller, tiles: () => [
        ...MATERIALS.map((m): Tile => ({ id: m.id, label: m.label, swatch: `url(${material(m.id)})`, place: { type: 'floor', material: m.id } })),
        ...ROOM_CHIPS.map((c): Tile => ({ id: c.code, label: c.name, swatch: c.color, place: { type: 'color', color: c.color } })),
      ],
    },
    {
      id: 'measure', label: 'Mesures', icon: Ruler, tiles: () => [
        { id: 'measure', label: 'Mesurer', icon: Ruler, tool: 'measure' },
        { id: 'dim', label: 'Ajouter une cote', icon: MoveHorizontal, tool: 'dim' },
        { id: 'note', label: 'Ajouter une note', icon: StickyNote, tool: 'note' },
        { id: 'detect', label: 'Détecter une pièce', icon: ScanSearch, tool: 'detect' },
        { id: 'layer', label: 'Photo de plan', icon: Layers, run: () => { app.selection = []; app.panel = 'props'; } },
      ],
    },
  ];
  let tab = $state('build');
  let thumbs = $state<Record<string, string>>({});
  const tiles = $derived(TABS.find((x) => x.id === tab)!.tiles());
  const samePlace = (a: Placement | null, b?: Placement) => !!a && !!b && JSON.stringify(a) === JSON.stringify(b);

  onMount(() => {
    let stop = false;
    void renderThumbs((k, v) => (thumbs[k] = v), () => stop);
    return () => {
      stop = true;
    };
  });
  function setTool(tool: Tool, wall?: WallKind) {
    editor.cancel();
    app.selection = [];
    if (wall) app.wallKind = wall;
    app.tool = tool;
  }
  function press(tile: Tile, e: PointerEvent) {
    if (tile.place) {
      e.preventDefault();
      editor.startPlacing(tile.place, e);
    }
  }
  function click(tile: Tile) {
    if (tile.tool) setTool(tile.tool, tile.wall);
    tile.run?.();
  }
  const on = (tile: Tile) => samePlace(editor.placing, tile.place) || (!!tile.tool && app.tool === tile.tool && (!tile.wall || app.wallKind === tile.wall));
</script>

<section class="build" data-ui aria-label={t('Catalogue')}>
  <div class="main">
    <button class:on={app.tool === 'select' && !editor.placing} onclick={() => setTool('select')}><MousePointer2 size={22} />{t('Modifier')}</button>
    <button class="hammer" class:on={app.tool === 'erase'} onclick={() => setTool('erase')}><Hammer size={22} />{t('Supprimer')}</button>
  </div>
  <div class="cat">
    <div class="tabs" role="tablist">
      {#each TABS as x}
        {@const Icon = x.icon}
        <button role="tab" aria-selected={tab === x.id} onclick={() => (tab = x.id)}><Icon size={16} />{t(x.label)}</button>
      {/each}
    </div>
    <div class="tiles">
      {#each tiles as tile (tile.id)}
        {@const Icon = tile.icon}
        <button class="tile" class:on={on(tile)} title={t(tile.label)} onpointerdown={(e) => press(tile, e)} onclick={() => click(tile)}>
          <span class="pic" style={tile.swatch ? `background:${tile.swatch} center/cover` : ''}>
            {#if tile.thumb && thumbs[tile.thumb]}<img src={thumbs[tile.thumb]} alt="" draggable="false" />{:else if Icon}<Icon size={32} />{/if}
          </span>
          <b>{t(tile.label)}</b>
          {#if tile.note}<small>{t(tile.note)}</small>{/if}
        </button>
      {/each}
    </div>
  </div>
</section>

<style>
  .build { grid-area: build; display: flex; min-width: 0; background: var(--panel); border-top: 1px solid var(--rule); }
  .main { display: grid; gap: 6px; flex: 0 0 auto; width: 104px; padding: 8px; border-right: 1px solid var(--rule); }
  .main button { display: grid; justify-items: center; align-content: center; gap: 2px; border: 1px solid var(--rule); border-radius: 7px; background: var(--panel); font-size: 13.5px; font-weight: 600; }
  .main button:hover { background: var(--panel-2); }
  .main button.on { background: var(--accent); border-color: var(--accent); color: var(--accent-ink); }
  .main .hammer.on { background: var(--st-demolish); border-color: var(--st-demolish); color: #fff; }
  .cat { flex: 1; min-width: 0; display: grid; grid-template-rows: auto 1fr; }
  .tabs { display: flex; gap: 2px; padding: 6px 8px 2px; overflow-x: auto; scrollbar-width: none; }
  .tabs button { display: inline-flex; align-items: center; gap: 5px; height: 32px; padding: 0 10px; border: 0; border-radius: 6px; background: transparent; color: var(--soft); font-size: 14px; white-space: nowrap; }
  .tabs button:hover { background: var(--panel-2); color: var(--ink); }
  .tabs button[aria-selected='true'] { background: var(--ink); color: var(--panel); font-weight: 650; }
  .tiles { display: flex; gap: 6px; min-height: 0; padding: 5px 8px 8px; overflow-x: auto; overflow-y: hidden; scrollbar-width: none; }
  .tile { position: relative; flex: 0 0 108px; display: grid; grid-template-rows: 1fr auto; justify-items: center; gap: 2px; min-width: 0; padding: 3px 4px 4px; border: 1.5px solid var(--rule); border-radius: 8px; background: var(--plan-sheet); cursor: grab; touch-action: none; }
  .tile:hover { border-color: var(--accent); transform: translateY(-1px); }
  .tile.on { border-color: var(--accent); box-shadow: inset 0 0 0 1.5px var(--accent); }
  .pic { display: grid; place-items: center; width: 100%; height: 66px; overflow: hidden; border-radius: 5px; color: var(--soft); }
  .pic img { width: 100%; height: 100%; object-fit: contain; pointer-events: none; }
  b { max-width: 100%; overflow: hidden; font-size: 13px; font-weight: 600; line-height: 1.15; white-space: nowrap; text-overflow: ellipsis; }
  small { position: absolute; top: 5px; right: 6px; padding: 0 4px; font-size: 11px; color: var(--soft); background: color-mix(in srgb, var(--plan-sheet) 85%, transparent); border-radius: 3px; }
</style>
