# RenovaPlan — Design System (`DESIGN.md`)

## 1. Identité Visuelle : « Atelier d'Architecture & Chantier de Précision »
RenovaPlan est une surface de type **Operate** (outil de conception CAD/BIM domestique). L'interface s'efface au profit de la lisibilité géométrique : le plan 2D et la maquette 3D occupent plus de 75 % de la surface utile, encadrés par des panneaux d'inspection denses, alignés au pixel et structurés par la typographie tabulaire.

### Scène d'usage
Consulté sur un écran QHD/4K sous Pop!_OS Wayland aussi bien en journée (lumière naturelle, besoin d'un fond papier millimétré contrasté pour lire les cotes) qu'en soirée (fond sombre ardoise COSMIC reposant pour les yeux). Le logiciel offre donc des thèmes Clair et Sombre complets où le canevas 2D et la scène 3D adaptent automatiquement leurs couleurs de fond, de grille et de cotation.

## 2. Palettes Sémantiques & Codes Chantier
Au-delà des couleurs d'interface, la rénovation impose une sémantique de couleur universelle issue des plans d'exécution d'architecte :

| Rôle Métier | Couleur (Mode Sombre / Clair) | Usage |
|---|---|---|
| **Existant conservé** | `#64748B` / `#334155` | Murs porteurs et cloisons existantes conservées |
| **À démolir (Démolition)** | `#EF4444` / `#DC2626` | Cloisons et ouvertures abattues (pointillés + rouge chantier) |
| **Nouveau (Création)** | `#10B981` / `#059669` | Nouvelles cloisons BA13, nouvelles ouvertures (vert émeraude) |
| **Sélection & Cote active** | `#F59E0B` / `#D97706` | Élément sélectionné, poignées de redimensionnement, faisceau d'accrochage |
| **Cotation & Grille** | `#38BDF8` / `#0284C7` | Lignes de cotes architecturales, flèches de mesure, étiquettes de dimensions |

### Thèmes Inclus
1. **COSMIC Dark (Pop!_OS)** : Fond `#14161B`, panneaux `#1C1F26`, surface `#262A34`, accent `#F59E0B` (Ambre Pop) & `#38BDF8` (Cyan).
2. **Atelier Clair (Papier Architecte)** : Fond canevas `#F8FAFC`, panneaux `#F1F5F9`, surface `#E2E8F0`, texte `#0F172A`, accent `#0284C7` (Bleu tirage).
3. **Catppuccin Mocha** : Fond `#1E1E2E`, panneaux `#181825`, surface `#313244`, accent `#89B4FA`.
4. **Nord Blueprint** : Fond `#2E3440`, panneaux `#242933`, surface `#3B4252`, accent `#88C0D0`.

## 3. Typographie & Hiérarchie
- **Police embarquée** : `Inter Variable` (avec licence OFL incluse dans le Flatpak).
- **Échelle typographique accessible (Mode Normal / Mode Grand Texte)** :
  - Titre d'en-tête & sélection : `17px` (`20px` en mode Grand Texte, `font-weight: 800`)
  - Boutons, outils & valeurs cotées : `15px` (`17px` en mode Grand Texte, `font-weight: 600–700`)
  - Textes d'explication, sous-titres & barre d'aide : `13px` (`15px` en mode Grand Texte)
- **Cibles tactiles / souris** : `42px` à `54px` de hauteur minimum sur tous les boutons et cartes du catalogue.

## 4. Topologie de l'Espace de Travail (Mode Guidé & Mode Expert)
1. **Barre supérieure conviviale (`56px`)** :
   - Accès en un clic à l'écran **« Maisons »** (modèles illustrés : Maison familiale 110 m², Appartement 48 m², Feuille blanche) + Annuler / Rétablir.
   - Commutateur central **Mode de Vue** : `2D` | `2D + 3D` | `3D`.
   - Commutateur **Phase Travaux** : `Chantier` | `Avant` | `Après`.
   - Boutons de confort : **`Texte +` / `Grand ✓`**, **`Guidé` / `Expert`**, Langue (`FR ▸ EN`), Thème (**Clair Papier** par défaut, Sombre COSMIC, Mocha, Nord) et Aide (`?`).
2. **Bandeau de parcours guidé en 4 étapes (`54px`)** :
   - `1. Pièces & Murs` → `2. Portes & Fenêtres` → `3. Meubles & Déco` → `4. Travaux & Prix`.
3. **Panneau d'accompagnement à gauche (`310px`)** :
   - Encadré d'explication en langage courant (« Que faire à cette étape ? »).
   - Outils adaptés à l'étape active : **Créer une pièce complète (4 murs + sol)** en un glisser-déposer, **Marteau : marquer un mur à casser** en un clic, pose directe de 7 types de portes/fenêtres/baies vitrées, et catalogue de meubles avec gros boutons **`+ Poser`**.
   - En **Mode Expert**, affiche l'intégralité des outils et réglages de grille sur un seul panneau.
3. **Zone centrale (Canevas 2D & Viewport 3D interactifs)** :
   - Rendu haute fréquence par buffer pixel (`slint::Image` RGBA) à la résolution exacte du viewport.
   - En **2D** : Panoramique (clic droit / milieu ou barres), zoom molette, accrochage magnétique aux extrémités de murs, aperçu fantôme coté pendant le tracé, poignées de sélection et cotations automatiques.
   - En **3D** : Caméra orbitale fluide (clic gauche glisser = orbite azimut/élévation, clic droit = déplacement cible, molette = zoom), boutons de vues prédéfinies (Isométrique, Dessus, Façade Sud, Visite intérieure), curseur de hauteur de coupe des murs (pour voir l'intérieur facilement).
4. **Inspecteur contextuel & Métré à droite (`280px`)** :
   - Onglet **Propriétés** : édition au centimètre près de l'élément sélectionné (longueur, épaisseur, hauteur, élévation, statut Existant/Démolition/Création, matériau/couleur, sens d'ouverture, prix unitaire).
   - Onglet **Métré & Devis** : récapitulatif Loi Carrez, surface au sol par pièce, linéaire à démolir, linéaire à créer, surface de peinture, et détail chiffré par poste de rénovation.
   - Onglet **Étages & Calques** : gestion Rez-de-chaussée / Étage 1 / Combles et visibilité des cotes, grilles et meubles.
5. **Barre d'état inférieure (`28px`)** :
   - Coordonnées exactes du curseur `(X, Y)` en mètres, longueur/angle du tracé en cours, résumé instantané (`Surface Carrez`, `Murs à démolir`, `Coût estimé`), et astuce contextuelle de l'outil actif en FR/EN.
