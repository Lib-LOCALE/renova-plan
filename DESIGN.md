---
name: RenovaPlan
description: Nuancier de chantier pour dessiner, visualiser et chiffrer une rénovation
colors:
  shell: "#1e3556"
  shell-2: "#294569"
  ground: "#e6e9ec"
  panel: "#fbfcfc"
  ink: "#1a2128"
  soft: "#58636d"
  rule: "#d3d9de"
  accent: "#1d5fd8"
  mark: "#c2410c"
  status-new: "#157a55"
  status-demolish: "#c4322b"
  glass: "#1a7a98"
typography:
  family: "Hanken Grotesk Variable"
  body: "15px/1.4, 400"
  panel-title: "14px, 700"
  label: "12px, 400"
rounded:
  control: "3px"
  group: "4px"
  chip: "2px"
spacing:
  tight: "4px"
  control: "8px"
  panel: "12px"
  section: "16px"
---

# RenovaPlan : système visuel

## Idée

L'interface emprunte au nuancier de peinture. La coque reste en bleu de travail, les surfaces restent neutres, et la couleur vive appartient aux pièces du logement. Chaque pièce devient une carte de teinte avec un nom, une surface et un code.

## Structure

- Barre haute de 56 px : annuler, affichage, phase, pièces, budget et enregistrer.
- Zone de travail pleine largeur : 3D, plan ou les deux.
- Catalogue bas de 156 px : Modifier, Supprimer, catégories et vignettes à glisser.
- Barre d'actions flottante à côté de l'objet sélectionné.
- Panneau droit de 340 px, fermé par défaut, pour les détails et le budget.

## Couleur

- Bleu de travail pour la coque et accent cobalt pour la sélection et l'action principale.
- Vert pour ce qui est créé, rouge hachuré pour ce qui est démoli, orange pour une cote à vérifier.
- Les trois thèmes Clair, Sombre et Dracula redéfinissent les mêmes variables dans `src/app.css`.

## Plan et 3D

- Le plan est du SVG en coordonnées réelles. Les textes gardent une taille constante grâce à `--k`.
- Les états sont doublés par un motif ou un trait, jamais par la seule couleur.
- En 3D, les murs sont coupés bas par défaut et leur section reste foncée, comme sur une maquette.

## Règles

- Faire : réutiliser les variables, garder les chiffres alignés, nommer les actions par leur effet.
- Ne pas faire : colorer les contrôles inactifs, ajouter des cartes hors nuancier, coder une couleur d'interface en dur.
