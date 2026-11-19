import { describe, expect, it } from 'vitest';
import { computeEstimate, estimateCsv } from '../src/lib/model/estimate';
import { detectRoom, polygonArea, rectPoints, solidSpans } from '../src/lib/model/geometry';
import { parseProject } from '../src/lib/model/migrate';
import { addBoxedRoom, addOpening, addWall, deleteRefs, moveRoomEdge, moveWall, moveWallEnd } from '../src/lib/model/ops';
import { blankProject, sampleFlat } from '../src/lib/model/templates';
import { emptyProject, visibleIn } from '../src/lib/model/types';
import { eraseRef, placeItem, placeOpening, placeRoom, roomOverlaps, ROOM_PRESETS } from '../src/lib/model/place';
import { hitTest, snapPoint } from '../src/lib/plan/hit';
import { planMarkup, standaloneSvg } from '../src/lib/plan/markup';

describe('géométrie', () => {
  it('calcule les surfaces et les portions pleines', () => {
    expect(polygonArea(rectPoints(1, 2, 3.19, 6.06))).toBeCloseTo(19.3314, 4);
    expect(solidSpans(5, [{ offset: 2, width: 1 }])).toEqual([[0, 1.5], [2.5, 5]]);
    expect(solidSpans(2, [{ offset: 1, width: 2 }])).toEqual([]);
  });

  it('détecte une pièce fermée en déduisant les murs', () => {
    const p = emptyProject();
    const c = rectPoints(0, 0, 4, 3);
    c.forEach((a, i) => addWall(p, 'l0', a, c[(i + 1) % 4], 'exterior'));
    addWall(p, 'l0', { x: 2, y: 0 }, { x: 2, y: 3 }, 'partition');
    const room = detectRoom(p.walls, { x: 1, y: 1 })!;
    expect(room).toBeTruthy();
    expect(polygonArea(room)).toBeCloseTo((2 - 0.125 - 0.05) * (3 - 0.25), 3);
    expect(detectRoom(p.walls, { x: 9, y: 9 })).toBeNull();
  });
});

describe('édition', () => {
  it('garde les murs reliés et la pièce quand un mur bouge', () => {
    const p = emptyProject();
    const room = addBoxedRoom(p, 'l0', 'Test', 0, 0, 4, 3);
    expect(p.walls).toHaveLength(4);
    const east = p.walls.find((w) => w.a.x > 3 && w.b.x > 3)!;
    moveWall(p, east.id, { x: 0.5, y: 0 });
    expect(polygonArea(room.points)).toBeCloseTo(4.5 * 3, 3);
    const north = p.walls.find((w) => w.a.y < 0 && w.b.y < 0)!;
    expect(Math.max(north.a.x, north.b.x)).toBeCloseTo(4.55, 3);
  });

  it('réutilise un mur mitoyen entre deux pièces accolées', () => {
    const p = emptyProject();
    addBoxedRoom(p, 'l0', 'A', 0, 0, 3, 3);
    addBoxedRoom(p, 'l0', 'B', 3.1, 0, 3, 3);
    expect(p.walls).toHaveLength(7);
  });

  it('déplace un côté de pièce avec son mur et supprime les ouvertures du mur retiré', () => {
    const p = emptyProject();
    const room = addBoxedRoom(p, 'l0', 'Test', 0, 0, 4, 3);
    moveRoomEdge(p, room.id, 1, { x: 1, y: 0 });
    expect(polygonArea(p.rooms[0].points)).toBeCloseTo(15, 3);
    const wall = p.walls[0];
    addOpening(p, wall, 1, 'door');
    moveWallEnd(p, wall.id, 'a', { x: -1, y: -0.05 });
    deleteRefs(p, [{ kind: 'wall', id: wall.id }]);
    expect(p.openings).toHaveLength(0);
  });

  it('accroche aux extrémités et sélectionne ce qui est sous le pointeur', () => {
    const p = sampleFlat();
    const wall = p.walls[0];
    const s = snapPoint(p, 'l0', 'site', { x: wall.a.x + 0.03, y: wall.a.y - 0.03 }, { grid: 0.05, tolerance: 0.15 });
    expect(s.kind).toBe('end');
    expect(s.point).toEqual(wall.a);
    expect(hitTest(p, 'l0', 'site', p.items[0], 0.1)?.kind).toBe('item');
    expect(hitTest(p, 'l0', 'site', { x: 0.6, y: 1.4 }, 0.1)?.kind).toBe('room');
  });
});

describe('pose simple', () => {
  it('accole une pièce prête à la précédente en partageant le mur', () => {
    const p = emptyProject();
    const a = placeRoom(p, 'l0', 'after', ROOM_PRESETS[1], { x: 2, y: 2 }, 'new', 'exterior');
    expect(p.walls).toHaveLength(4);
    placeRoom(p, 'l0', 'after', ROOM_PRESETS[1], { x: 5.75, y: 2.1 }, 'new');
    const right = Math.max(...a.points.map((q) => q.x));
    const next = Math.min(...p.rooms[1].points.map((q) => q.x));
    expect(next - right).toBeCloseTo(0.25, 2);
    expect(p.walls).toHaveLength(7);
    expect(roomOverlaps(p, 'l0', 'after', a.points[0].x, a.points[0].y, 1, 1)).toBe(true);
  });

  it('pose une porte sur le mur voisin et adosse un meuble', () => {
    const p = emptyProject();
    addBoxedRoom(p, 'l0', 'A', 0, 0, 4, 3);
    expect(placeOpening(p, 'l0', 'after', 'door', { x: 2, y: 0.2 }, 'new')).toBeTruthy();
    expect(placeOpening(p, 'l0', 'after', 'door', { x: 20, y: 20 }, 'new')).toBeNull();
    const bed = placeItem(p, 'l0', 'after', 'bed-double', { x: 2, y: 1.3 }, 'new');
    expect(bed.y).toBeCloseTo(2.05 / 2 + 0.015, 2);
    expect(bed.rotation).toBe(0);
  });

  it('retire le neuf et marque l’existant à démolir', () => {
    const p = emptyProject();
    const room = addBoxedRoom(p, 'l0', 'A', 0, 0, 4, 3);
    const n = placeItem(p, 'l0', 'after', 'sofa', { x: 2, y: 1.5 }, 'new');
    eraseRef(p, { kind: 'item', id: n.id }, 'after');
    expect(p.items).toHaveLength(0);
    eraseRef(p, { kind: 'room', id: room.id }, 'after');
    expect(p.rooms[0].status).toBe('demolish');
  });
});

describe('chiffrage', () => {
  it('distingue le métré du plan et les postes saisis', () => {
    const p = sampleFlat();
    p.rooms[0].status = 'new';
    p.works = [{ id: 'test', lot: 'Test', label: 'Travail fictif', amount: 100, rge: true, artisan: '', funding: 'eco-ptz', state: 'quote' }];
    const e = computeEstimate(p);
    expect(e.floorArea).toBeGreaterThan(0);
    expect(e.metered).toBeGreaterThan(0);
    expect(e.total).toBe(100);
    expect(e.rge).toBe(100);
    expect(e.byFunding['eco-ptz']).toBe(100);
    expect(estimateCsv(p)).toContain('Travail fictif');
    p.works = [];
    expect(computeEstimate(p).total).toBeCloseTo(e.metered, 5);
  });

  it('applique les phases de chantier', () => {
    expect(visibleIn('new', 'before')).toBe(false);
    expect(visibleIn('demolish', 'after')).toBe(false);
    expect(visibleIn('existing', 'after')).toBe(true);
  });
});

describe('fichiers et rendu', () => {
  it('commence avec un plan vierge et un budget vide', () => {
    const p = blankProject();
    expect(p.levels).toHaveLength(1);
    expect([...p.rooms, ...p.walls, ...p.items, ...p.works]).toEqual([]);
    expect(computeEstimate(p).total).toBe(0);
  });

  it('relit un projet et refuse un fichier étranger', () => {
    const p = sampleFlat();
    expect(parseProject(JSON.stringify(p)).rooms).toHaveLength(3);
    expect(() => parseProject(JSON.stringify({ ...p, levels: [] }))).toThrow(/niveau/);
    expect(() => parseProject(JSON.stringify({ ...p, walls: null }))).toThrow(/Liste/);
    expect(() => parseProject('{')).toThrow(/JSON/);
    expect(() => parseProject('{"a":1}')).toThrow(/RenovaPlan/);
  });

  it("importe l'ancien format Slint", () => {
    const old = {
      title_fr: 'Ancien',
      levels: ['RDC'],
      walls: [{ id: 1, level: 0, x1: 0, y1: 0, x2: 4, y2: 0, thickness: 0.2, height: 2.5, load_bearing: true, status: 'existing', name: '' }],
      openings: [{ id: 2, level: 0, wall_id: 1, offset_t: 0.5, width: 0.8, height: 2, sill_height: 0, kind: 'single_door', status: 'new', flip_swing: false }],
      rooms: [{ id: 3, level: 0, name_fr: 'Salon', name_en: 'Living', x: 0, y: 0, width: 4, depth: 3, ceiling_height: 2.5, material: 'light_tile', status: 'existing' }],
      furniture: [{ id: 4, level: 0, kind: 'double_bed', label_custom: '', x: 1, y: 1, width: 1.6, depth: 2, height: 0.5, rotation_deg: 0, status: 'existing', cost: 100 }],
    };
    const p = parseProject(JSON.stringify(old));
    expect(p.walls[0].kind).toBe('bearing');
    expect(p.openings[0].offset).toBeCloseTo(2);
    expect(p.rooms[0].material).toBe('tile');
    expect(p.items[0].kind).toBe('bed-double');
  });

  it('produit un SVG exportable selon la phase', () => {
    const p = sampleFlat();
    p.rooms[2].status = 'new';
    p.rooms[2].name = 'PhaseTest';
    expect(planMarkup(p, 'l0', 'after', {})).toContain('PhaseTest');
    const svg = standaloneSvg(p, 'l0', 'before');
    expect(svg.startsWith('<svg')).toBe(true);
    expect(svg).not.toContain('PhaseTest');
  });
});
