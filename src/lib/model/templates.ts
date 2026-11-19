import { addBoxedRoom, addItem, addOpening, addWall } from './ops';
import { rectPoints } from './geometry';
import type { Project, Pt, Wall } from './types';
import { emptyProject } from './types';

function shell(p: Project, level: string, x: number, y: number, w: number, h: number): Wall[] {
  const t = 0.125;
  const [a, b, c, d] = rectPoints(x - t, y - t, w + 2 * t, h + 2 * t);
  return ([[a, b], [b, c], [c, d], [d, a]] as [Pt, Pt][]).map(([m, n]) => addWall(p, level, m, n, 'exterior'));
}

export function sampleFlat(): Project {
  const p = emptyProject('Appartement T2');
  const [n, , s] = shell(p, 'l0', 0, 0, 8, 6);
  const sejour = addBoxedRoom(p, 'l0', 'Pièce de vie', 0, 0, 5, 6, 'oak');
  sejour.color = '#f3c969';
  const ch = addBoxedRoom(p, 'l0', 'Chambre', 5.1, 0, 2.9, 3.4, 'oak');
  ch.color = '#a98ccf';
  const sdb = addBoxedRoom(p, 'l0', "Salle d'eau", 5.1, 3.5, 2.9, 2.5, 'tile');
  sdb.color = '#62b7d3';
  addOpening(p, n, 2.5, 'bay-window', 'existing', { tag: '1' });
  addOpening(p, n, 6.6, 'window', 'existing', { tag: '2' });
  addOpening(p, s, 1.2, 'entry-door');
  addItem(p, 'l0', 'sofa', 2.4, 2.6);
  addItem(p, 'l0', 'counter', 2.8, 5.65);
  addItem(p, 'l0', 'bed-double', 6.6, 1.15);
  addItem(p, 'l0', 'shower', 7.3, 5.4);
  return p;
}

export const blankProject = () => emptyProject('Nouveau projet');
