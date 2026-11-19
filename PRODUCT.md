# RenovaPlan — Product Definition

<!-- impeccable:product-schema 1 -->

## Platform

desktop

## Users

- **Particuliers en rénovation** : propriétaires qui relèvent leur logement, préparent des travaux et discutent avec leurs artisans.
- **Artisans et maîtres d'œuvre** : professionnels qui veulent dessiner vite un existant, tester une variante et sortir des quantités.

## Product Purpose

RenovaPlan transforme un relevé en plans cotés, en maquette 3D et en liste de travaux chiffrée. Chaque élément porte un état de chantier : existant, à démolir ou à créer.

## Positioning

Un outil de bureau hors ligne, sans compte ni abonnement, centré sur la rénovation plutôt que sur la décoration ou la construction neuve.

## Operating Context

- Windows 11 en priorité, Linux via Tauri.
- Usage à la maison ou sur chantier, souvent sans connexion.
- Écrans de 1280 px de large et plus, souris ou pavé tactile.

## Capabilities and Constraints

- Murs, pièces, détection de pièce, portes, fenêtres, escaliers, mobilier, cotes et notes.
- Calque de fond pour redessiner une photo de croquis à l'échelle.
- Phases Avant, Chantier et Après dans le plan comme dans la 3D.
- Maquette, étages séparés, niveau seul, extérieur, vue de dessus et visite.
- Métré, postes de travaux, artisans, exigence RGE et répartition du financement.
- Exports SVG, PNG, CSV et dossier imprimable.
- Stack : Tauri 2, Svelte 5, TypeScript et Three.js. Aucun service réseau à l'exécution.

## Brand Commitments

- **Nom** : RenovaPlan.
- **Identité** : nuancier de chantier, bleu de travail, couleurs réservées aux pièces et aux états.
- **Promesse** : les projets restent sur le disque de l'utilisateur, sans télémétrie.

## Evidence on Hand

- Tests du modèle : géométrie, édition, chiffrage, import et export.
- Parcours d'édition vérifiés dans l'interface.
- Exécutable Windows vérifié : démarrage, ouverture, enregistrement et export SVG.

## Product Principles

1. **Simple d'abord** : on construit en glissant des objets, sans régler de valeurs ni apprendre un logiciel de plan.
2. **Le plan fait foi** : une cote relevée reste visible, même quand elle contredit une autre.
3. **Vérité de chantier** : tout élément dit s'il reste, part ou arrive.
4. **Deux regards, un modèle** : 2D, 3D et chiffrage lisent les mêmes données.
5. **Local d'abord** : aucune fonction essentielle ne dépend d'Internet.

## Accessibility & Inclusion

- Contraste WCAG AA visé sur les thèmes Clair, Sombre et Dracula.
- Les états de chantier combinent couleur, motif et libellé.
- Raccourcis clavier pour les outils et les commandes principales.
- Interface en français et en anglais.
