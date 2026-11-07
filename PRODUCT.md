# RenovaPlan — Product Definition

## Vision
**RenovaPlan** est un logiciel natif Linux (Slint + Rust, optimisé Wayland / Pop!_OS et distribué en Flatpak autonome) dédié à la conception de plans 2D cotés et à la visualisation 3D temps réel pour la rénovation de maison.

Il combine la précision structurelle et l'autonomie hors-ligne de **Sweet Home 3D**, la clarté de rendu et de cotation de **RoomSketcher**, l'ergonomie immédiate de **Kozikaza**, et un système inédit pensé pour les chantiers de rénovation : la gestion des états de travaux (**Existant**, **Démolition**, **Création**), le **comparatif Avant / Projet / Chantier**, le **calque de fond mis à l'échelle**, et le **chiffrage / métré automatique** (surfaces Loi Carrez / Habitable, linéaires de murs, ouvertures, devis estimatif).

## Utilisateurs Cibles
- Particuliers rénovant une maison ou un appartement (abattage de cloisons, création de suite parentale, ouverture cuisine/séjour, aménagement d'étage).
- Artisans, maîtres d'œuvre et architectes d'intérieur cherchant un outil rapide sur Linux (Wayland) pour relever l'existant, tester des variantes d'aménagement et chiffrer immédiatement les quantités de matériaux.

## Fonctionnalités Clés
1. **Plan 2D Technique Coté & Interactif** :
   - Tracé de murs porteurs et cloisons avec magnétisme orthogonal et sur-grille (snapping).
   - Cotations automatiques en centimètres / mètres et surfaces par pièce ($m^2$, Loi Carrez $h \ge 1{,}80\text{ m}$ vs surface au sol).
   - Insertion intelligente de portes (battantes avec arc d'ouverture, coulissantes, doubles, ouvertures libres) et fenêtres (simples, doubles, baies vitrées, oscillo-battantes) qui s'encastrent sur les murs.
   - Catalogue de mobilier et d'équipements (Cuisine, Salon, Chambre, Salle de bain, Technique/Électricité) avec rotation, redimensionnement au centimètre et glisser-déposer.
   - Calque de fond (import d'image PNG ou grille d'architecte étalonnable) avec réglage d'opacité et d'échelle pour décalquer un plan existant.

2. **Mode Rénovation (Existant / Démolition / Création)** :
   - Chaque mur, ouverture ou équipement possède un statut : **Existant conservé** (gris ardoise technique), **À démolir** (rouge chantier hachuré/pointillé), **Nouveau / À créer** (vert émeraude ou ambre).
   - Filtre de phase instantané :
     - **Vue Chantier (Tout)** : affiche l'existant, les démolitions en rouge et les créations en vert.
     - **État Avant Rénovation** : affiche l'existant + les éléments qui seront démolis (masque les créations).
     - **État Après Rénovation (Projet final)** : affiche l'existant conservé + les créations (masque les murs démolis).

3. **Vue 3D Temps Réel Synchronisée** :
   - Vue 2D seule, Vue 3D seule, ou **Vue Scindée 2D + 3D côte-à-côte** mise à jour à chaque milliseconde.
   - Caméra 3D Orbitale (rotation azimut, élévation, zoom, panoramique) + mode Coupe / Hauteur des murs et éclairage directionnel solaire avec ombres portées au sol.
   - Rendu 3D des pièces (parquets, carrelages, bétons), des murs avec percements réels des portes et fenêtres (linteaux, allèges, vitrages translucides), et volumes 3D détaillés des meubles.

4. **Métré & Devis Estimatif Automatique** :
   - Calcul temps réel : Surface habitable totale (Carrez), surface au sol, périmètre, surface nette de murs à peindre/enduire (déduction faite des ouvertures), linéaire de cloisons à démolir (et volume de gravats en $m^3$), linéaire de cloisons BA13 à créer, nombre d'ouvertures et budget total estimé (en `€` ou `$`).
   - Export complet : Projet JSON natif, Plan vectoriel SVG coté, Capture PNG haute résolution (2D ou 3D), et Devis / Métré CSV/Markdown.

5. **Bilingue Français / Anglais & Palettes Linux** :
   - Interface 100 % traduite à chaud en **Français** et **Anglais** (détection automatique de la locale système + sélecteur immédiat).
   - Thèmes adaptés à Pop!_OS COSMIC et Linux : **Atelier Clair (Blueprint Paper)**, **COSMIC Sombre (Pop!_OS Dark)**, **Catppuccin Mocha**, **Ardoise Architecte (Nord)**.
