<script lang="ts">
  import { app } from '../state/app.svelte';

  interface Props {
    label: string;
    value: number;
    onchange: (v: number) => void;
    /** length : mètres affichés selon l'unité du projet ; raw : valeur telle quelle. */
    mode?: 'length' | 'raw';
    unit?: string;
    min?: number;
    step?: number;
  }
  let { label, value, onchange, mode = 'length', unit, min = 0, step }: Props = $props();
  const cm = $derived(mode === 'length' && app.project.dimUnit === 'cm');
  const factor = $derived(cm ? 100 : 1);
  const shown = $derived(mode === 'length' ? +(value * factor).toFixed(cm ? 1 : 3) : +value.toFixed(2));
  const suffix = $derived(unit ?? (mode === 'length' ? (cm ? 'cm' : 'm') : ''));

  function commit(e: Event & { currentTarget: HTMLInputElement }) {
    const v = parseFloat(e.currentTarget.value.replace(',', '.'));
    if (Number.isFinite(v) && v >= min) onchange(v / factor);
    else e.currentTarget.value = String(shown);
  }
</script>

<label class="field">
  <span>{label}{#if suffix} <small>({suffix})</small>{/if}</span>
  <input type="number" value={shown} step={step ?? (mode === 'length' ? (cm ? 1 : 0.01) : 1)} onchange={commit} />
</label>
