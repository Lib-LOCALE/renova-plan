# RenovaPlan — Product Definition

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Users

- **Particuliers en rénovation** : Propriétaires d'appartements ou de maisons planifiant des travaux d'aménagement intérieur (abattage de cloisons, création de suites parentales, réagencement cuisine/séjour, combles).
- **Artisans, maîtres d'œuvre et architectes d'intérieur** : Professionnels recherchant un outil de conception rapide, fonctionnant à 100 % hors ligne sur PC Windows 11 et Linux, pour relever un existant, tester des variantes et chiffrer immédiatement les quantitatifs de chantier.

## Product Purpose

RenovaPlan permet de concevoir des plans 2D cotés et de visualiser en 3D temps réel un aménagement intérieur, tout en automatisant le chiffrage et le métré des travaux grâce à la séparation rigoureuse des états de rénovation (**Existant**, **Démolition**, **Création**). Le succès se mesure à la vitesse de prise en main, à la précision des surfaces calculées (Loi Carrez) et à l'autonomie hors ligne totale.

## Positioning

RenovaPlan combine la précision structurelle de Sweet Home 3D, l'ergonomie moderne de Kozikaza et la clarté technique d'un plan d'architecte, sans aucun abonnement cloud, sans télémétrie, et avec un moteur dédié au chantier de rénovation : marquage des cloisons à abattre en rouge, nouvelles cloisons en vert, calcul immédiat du volume de gravats et devis estimatif instantané.

## Operating Context

- **Environnements d'exécution** : Ordinateurs de bureau et PC portables sous Windows 11 x64 (exécutable natif PE sans dépendances externes) et Linux (Wayland / Pop!_OS COSMIC / Flatpak).
- **Conditions d'usage** : En atelier, sur table à dessin ou directement sur chantier sans connexion Internet.
- **Affichage** : Optimisé pour écrans Full HD, QHD et 4K avec mode Grand Texte accessible.

## Capabilities and Constraints

- **Capacités clés** :
  - Tracé 2D coté avec magnétisme sur grille et accrochage orthogonal.
  - Calcul en temps réel de la surface Loi Carrez ($h \ge 1{,}80\text{ m}$), surface au sol, linéaires de cloisons à démolir/créer et surface de murs à peindre.
  - Insertion intelligente de portes (battantes, doubles, coulissantes, passages libres) et fenêtres s'encastrant sur les cloisons.
  - Rendu 3D orbital temps réel avec textures, percements réels de murs et réglage de hauteur de coupe.
  - Devis estimatif chiffré par poste avec devises personnalisables.
  - Export vectoriel SVG coté, devis CSV/Markdown et projet natif JSON.
  - Bilingue temps réel Français / Anglais avec bascule à chaud.
- **Contraintes techniques** :
  - Moteur 100 % Rust (Slint + FemtoVG / Winit).
  - Aucune dépendance réseau requise à l'exécution.
  - Sauvegarde et préférences locales sécurisées dans `%APPDATA%\RenovaPlan\` (Windows) et `~/.config/RenovaPlan/` (Linux).

## Brand Commitments

- **Nom** : RenovaPlan.
- **Ton & Identité** : Atelier d'architecture, chantier de précision, lisibilité géométrique absolue.
- **Règles d'or** : Aucun artifice décoratif gratuit, aucun écran de chargement bloquant, aucune collecte de données personnelles.

## Evidence on Hand

- **Modèles de démarrage intégrés** : Maison familiale (110 m²), Appartement 2 pièces (48 m²), Canevas vierge structuré.
- **Tests unitaires et d'intégration** : 9 tests automatisés couvrant les calculs géométriques, l'export SVG/CSV bilingue et la conformité des thèmes.
- **Rendus d'interface vérifiés** : Snapshots réels du cycle complet (2D, 3D scindée, sélecteur de modèles, devis estimatif).

## Product Principles

1. **Autonomie locale absolue** : Les projets, devis et plans restent sous le contrôle exclusif de l'utilisateur sur son disque local.
2. **Vérité de chantier** : Un plan n'est pas qu'un dessin, c'est une liste d'actions concrètes (conserver, démolir, construire) avec un impact financier direct.
3. **Double regard instantané** : Synchronisation permanente milliseconde par milliseconde entre la rigueur du plan 2D coté et la projection 3D spatiale.
4. **Accessibilité universelle** : Respect des contrastes WCAG AA, lisibilité sur tous les thèmes, raccourcis clavier natifs et interface bilingue complète.

## Accessibility & Inclusion

- Conforme WCAG 2.2 niveau AA (ratios de contraste de texte et boutons $\ge 4{,}5:1$ sur les 4 thèmes).
- Double encodage des statuts chantier (couleur + symboles `✓`, `✕`, `+`) pour les utilisateurs daltoniens.
- Mode « Grand Texte » agrandissant l'échelle typographique à 16–18 px.
- Prise en charge native de la navigation clavier et des raccourcis universels (`Ctrl+S`, `Ctrl+O`, `Ctrl+Z`, `Échap`, `Suppr`).
