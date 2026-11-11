---
name: RenovaPlan
description: Atelier d'architecture et de conception 2D/3D pour la rénovation de maison
colors:
  primary: "#0369a1"
  primary-dark: "#38bdf8"
  neutral-bg: "#faf8f5"
  neutral-panel: "#f1ede4"
  neutral-surface: "#ffffff"
  neutral-text: "#18181b"
  neutral-muted: "#52525b"
  neutral-border: "#cfc8b8"
  status-existing: "#64748b"
  status-demolish: "#dc2626"
  status-new: "#059669"
  status-selection: "#d97706"
typography:
  display:
    fontFamily: "Inter Variable, system-ui, sans-serif"
    fontSize: "24px"
    fontWeight: 800
    lineHeight: 1.2
  title:
    fontFamily: "Inter Variable, system-ui, sans-serif"
    fontSize: "16px"
    fontWeight: 800
    lineHeight: 1.3
  body:
    fontFamily: "Inter Variable, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 500
    lineHeight: 1.4
  label:
    fontFamily: "Inter Variable, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 600
    lineHeight: 1.3
rounded:
  sm: "6px"
  md: "8px"
  lg: "10px"
  xl: "14px"
spacing:
  xs: "4px"
  sm: "6px"
  md: "8px"
  lg: "12px"
  xl: "16px"
  xxl: "24px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "#ffffff"
    rounded: "{rounded.md}"
    padding: "8px 12px"
  button-surface:
    backgroundColor: "{colors.neutral-surface}"
    textColor: "{colors.neutral-text}"
    rounded: "{rounded.md}"
    padding: "8px 12px"
  card-panel:
    backgroundColor: "{colors.neutral-panel}"
    textColor: "{colors.neutral-text}"
    rounded: "{rounded.xl}"
    padding: "16px 20px"
---

# RenovaPlan — Design System

## Overview

RenovaPlan est une surface de type **Operate** dédiée à la conception CAD/BIM domestique et à la planification de chantier. L'interface s'efface pour laisser plus de 75 % de l'espace visuel au plan 2D coté et à la maquette 3D temps réel. Les panneaux latéraux et barres d'outils appliquent un rythme géométrique rigoureux, des contrastes élevés et une typographie tabulaire adaptée à la lecture de cotes au millimètre.

## Colors

Le système repose sur quatre palettes sémantiques adaptatives et un code couleur universel issu des plans d'exécution de rénovation :

### Palettes d'Interface

1. **Atelier Clair (Défaut)** : Fond canevas `#faf8f5`, panneaux `#f1ede4`, surfaces `#ffffff`, texte `#18181b`, accent `#0369a1` (Bleu tirage technique).
2. **COSMIC Sombre (Pop!_OS)** : Fond `#14161b`, panneaux `#1c1f26`, surfaces `#262a34`, texte `#f1f5f9`, accent `#38bdf8` (Cyan).
3. **Catppuccin Mocha** : Fond `#1e1e2e`, panneaux `#181825`, surfaces `#313244`, texte `#cdd6f4`, accent `#89b4fa`.
4. **Nord Architect** : Fond `#2e3440`, panneaux `#242933`, surfaces `#3b4252`, texte `#eceff4`, accent `#88c0d0`.

### Sémantique Métier Chantier

| Rôle Métier | Symbole | Couleur Clair / Sombre | Signification |
|---|---|---|---|
| **Existant conservé** | `✓` | `#64748B` / `#94A3B8` | Murs porteurs et éléments conservés en l'état |
| **Démolition** | `✕` | `#DC2626` / `#F87171` | Cloisons et ouvertures abattues (pointillés + rouge chantier) |
| **Création** | `+` | `#059669` / `#34D399` | Nouvelles cloisons BA13 et ouvertures neuves (vert émeraude) |
| **Sélection & Cote** | `◆` | `#D97706` / `#FBBF24` | Poignées de redimensionnement et faisceau magnétique |

## Typography

- **Famille principale** : `Inter Variable` embarquée directement dans le binaire pour un rendu vectoriel identique sur Windows et Linux.
- **Échelle typographique** :
  - **Titre dialogue & modale** : `24px`, poids 800.
  - **Titre de panneau & sélection** : `16px` (`18px` en mode Grand Texte), poids 800.
  - **Boutons & outils** : `14px` (`16px` en mode Grand Texte), poids 600–700.
  - **Cotes & mesures** : Chiffres tabulaires alignés à droite pour éviter tout sautillement de mise en page.
  - **Légendes & barre d'état** : `12px` (`14px` en mode Grand Texte), poids 500–600.

## Layout

- **Barre supérieure (`54px`)** : Titre, modèles illustrés, annuler/rétablir, sélecteur de vue (2D / 2D+3D / 3D), filtre de phase (Avant / Après / Chantier), badge de coût direct, sélecteur de langue (`FR ▾ / EN ▾`), thème et aide.
- **Rail d'outils docké à gauche (`96px`)** : Onglets à larges cibles tactiles (`1. Pièces`, `2. Ouvertures`, `3. Meubles`, `4. Devis`, `? Aide`).
- **Tiroir contextuel d'étape (`276px`)** : Accompagnement pas à pas avec pièces prêtes en 1 clic, outils de tracé coté, catalogue de mobilier et tableau d'estimation budgétaire.
- **Zone centrale de travail (75 % de l'écran)** : Canevas 2D interactif et viewport 3D orbital haute fréquence.
- **Barre contextuelle flottante (`60px`)** : Apparaît uniquement lors de la sélection d'un élément pour ajuster ses dimensions, son statut et ses actions rapides.
- **Barre d'état inférieure (`34px`)** : Coordonnées réelles en mètres, légende double encodée et astuce contextuelle.

## Elevation & Depth

- **Plan de travail** : Élévation zéro, netteté vectorielle pure sans ombre portée sur les murs 2D.
- **Panneaux & Tiroirs** : Délimitation par bordure fine de 1 px (`border-color: Theme.border`) avec fond teinté doux (`panel`).
- **Modales & Dialogues** : Voile semi-transparent (`#000000aa`), bordure d'accentuation de 2 px (`border-color: Theme.accent`) et rayon de courbure de 14 px.
- **Scène 3D** : Éclairage solaire directionnel avec ombrage projeté au sol pour asseoir les volumes.

## Shapes

- **Boutons d'action standards** : Rayon de courbure de 8 px (`border-radius: 8px`).
- **Onglets du rail principal** : Rayon de courbure de 10 px.
- **Puces d'icônes & badges** : Rayon de courbure de 6 px.
- **Cartes de modèles de démarrage** : Rayon de courbure de 10 à 14 px avec marges généreuses de 16 à 24 px.

## Components

- **`ActionButton`** : Bouton polyvalent avec gestion d'état (`active`, `danger`, `emerald`, `amber-highlight`, `enabled`), retour tactile au clic, transition fluide de bordure (140 ms) et étiquette accessible explicite.
- **`ToolItem`** : Outil de tracé avec puce d'icône contrastée, libellé et retour de sélection persistant.
- **`DockTab`** : Onglet d'étape architecturale avec pictogramme géométrique et texte enroulable.
- **`MetricRow`** : Ligne de métrique financière ou de surface avec libellé muet à gauche et valeur tabulaire en gras à droite.

## Do's and Don'ts

### À faire (Do's)
- **Toujours** associer une forme (`✓`, `✕`, `+`) à une couleur pour les états de rénovation.
- **Toujours** préserver des ratios de contraste supérieurs à 4,5:1 pour tout texte ou chiffre clé sur fond teinté.
- **Toujours** utiliser les chiffres tabulaires pour les métrés, surfaces Carrez et montants en euros.
- **Toujours** offrir un raccourci clavier standard (`Ctrl+S`, `Ctrl+O`, `Ctrl+Z`, `Échap`) en complément du clic souris.

### À proscrire (Don'ts)
- **Ne jamais** afficher de texte blanc sur fond jaune ou ambre clair en mode sombre (utiliser `#0f172a`).
- **Ne jamais** bloquer l'utilisateur dans une boîte de dialogue modale sans possibilité de fermer par la touche `Échap`.
- **Ne jamais** coder des chaînes en dur sans clé correspondante dans [`ui/data/i18n.json`](file:///c:/Users/TonyBoySUPER/Documents/Projets/renova-plan/ui/data/i18n.json).
- **Ne jamais** afficher d'ombres lourdes ou d'effets néomorphiques flous sur les éléments de dessin géométrique.
