import { afterEach, expect, it, vi } from 'vitest';
import { openProjectFile } from '../src/lib/state/platform';
import { app } from '../src/lib/state/app.svelte';
import { blankProject } from '../src/lib/model/templates';

vi.mock('../src/lib/state/platform', () => ({ openProjectFile: vi.fn() }));

afterEach(() => {
  vi.unstubAllGlobals();
  vi.clearAllMocks();
});

it('persists a completed edit immediately, including undo and redo', () => {
  const drafts = new Map<string, string>();
  vi.stubGlobal('localStorage', {
    setItem: (key: string, value: string) => drafts.set(key, value),
    getItem: (key: string) => drafts.get(key),
  });
  app.adopt(blankProject());
  const savedName = () => JSON.parse(drafts.get('renovaplan.autosave.v2')!).name;
  app.commit(p => { p.name = 'Test'; });
  expect(savedName()).toBe('Test');
  app.undo();
  expect(savedName()).toBe('Nouveau projet');
  app.redo();
  expect(savedName()).toBe('Test');
});

it('keeps unsaved edits when creating or opening another project is cancelled', async () => {
  const confirm = vi.fn().mockResolvedValue(false);
  vi.stubGlobal('window', { confirm });
  app.adopt(blankProject());
  app.commit(p => { p.name = 'En cours'; });
  await app.newFrom('flat');
  expect(app.project.name).toBe('En cours');
  vi.mocked(openProjectFile).mockResolvedValue({
    name: 'test.renova', path: '/test.renova', text: JSON.stringify(blankProject()),
  });
  await app.open();
  expect(app.project.name).toBe('En cours');
  expect(app.dirty).toBe(true);
  confirm.mockResolvedValue(true);
  await app.open();
  expect(app.filePath).toBe('/test.renova');
  expect(app.dirty).toBe(false);
});
