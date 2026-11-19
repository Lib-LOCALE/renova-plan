/** Fichiers : dialogues natifs dans Tauri, équivalents navigateur ailleurs. */
export const isTauri = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export interface Picked {
  name: string;
  path: string | null;
  text: string;
}

const PROJECT_FILTER = [{ name: 'Projet RenovaPlan', extensions: ['renova', 'json'] }];
const base = (p: string) => p.split(/[\\/]/).pop() ?? p;

function pickBrowserFile(accept: string): Promise<File | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = accept;
    input.onchange = () => resolve(input.files?.[0] ?? null);
    input.oncancel = () => resolve(null);
    input.click();
  });
}

export async function openProjectFile(): Promise<Picked | null> {
  if (isTauri()) {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const { readTextFile } = await import('@tauri-apps/plugin-fs');
    const path = await open({ multiple: false, filters: PROJECT_FILTER });
    if (typeof path !== 'string') return null;
    return { name: base(path), path, text: await readTextFile(path) };
  }
  const f = await pickBrowserFile('.renova,.json');
  return f ? { name: f.name, path: null, text: await f.text() } : null;
}

export async function saveProjectFile(name: string, text: string, path: string | null): Promise<string | null> {
  if (isTauri()) {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const { writeTextFile } = await import('@tauri-apps/plugin-fs');
    const target = path ?? (await save({ defaultPath: `${name}.renova`, filters: PROJECT_FILTER }));
    if (!target) return null;
    await writeTextFile(target, text);
    return target;
  }
  download(`${name}.renova`, new Blob([text], { type: 'application/json' }));
  return `${name}.renova`;
}

export async function exportFile(name: string, data: string | Uint8Array, mime: string): Promise<string | null> {
  if (isTauri()) {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const { writeFile, writeTextFile } = await import('@tauri-apps/plugin-fs');
    const ext = name.split('.').pop()!;
    const target = await save({ defaultPath: name, filters: [{ name: ext.toUpperCase(), extensions: [ext] }] });
    if (!target) return null;
    if (typeof data === 'string') await writeTextFile(target, data);
    else await writeFile(target, data);
    return target;
  }
  download(name, new Blob([data as BlobPart], { type: mime }));
  return name;
}

function download(name: string, blob: Blob) {
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = name;
  a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 2000);
}

export async function pickImage(): Promise<{ src: string; aspect: number } | null> {
  const f = await pickBrowserFile('image/*');
  if (!f) return null;
  const src = await new Promise<string>((resolve, reject) => {
    const r = new FileReader();
    r.onload = () => resolve(String(r.result));
    r.onerror = () => reject(new Error("Impossible de lire l'image."));
    r.readAsDataURL(f);
  });
  const aspect = await new Promise<number>((resolve, reject) => {
    const i = new Image();
    i.onload = () => resolve(i.naturalWidth / i.naturalHeight);
    i.onerror = () => reject(new Error("Ce format d'image n'est pas pris en charge."));
    i.src = src;
  });
  return { src, aspect };
}

export function dataUrlBytes(url: string): Uint8Array {
  const bin = atob(url.split(',')[1]);
  return Uint8Array.from(bin, (c) => c.charCodeAt(0));
}
