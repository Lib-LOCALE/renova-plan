use crate::model::{
    EstimateLine, EstimateSummary, FloorMaterial, FurnitureItem, FurnitureKind, Lang, Opening,
    OpeningKind, RenoStatus, RoomZone, SelectedEntity, ViewPhase, Wall,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub title_fr: String,
    pub title_en: String,
    pub levels: Vec<String>,
    pub active_level: usize,
    pub walls: Vec<Wall>,
    pub openings: Vec<Opening>,
    pub rooms: Vec<RoomZone>,
    pub furniture: Vec<FurnitureItem>,
    pub show_grid: bool,
    pub snap_to_grid: bool,
    pub grid_step_m: f32,
    pub show_dimensions: bool,
    pub show_underlay: bool,
    pub underlay_opacity: f32,
    pub underlay_scale_m: f32,
    next_id: u32,
}

impl Default for Project {
    fn default() -> Self {
        Self::preset_maison_renovation()
    }
}

impl Project {
    pub fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn title(&self, lang: Lang) -> &str {
        match lang {
            Lang::Fr => &self.title_fr,
            Lang::En => &self.title_en,
        }
    }

    /// Preset 1: Maison années 70 à rénover (Ouverture cuisine/séjour + création suite parentale)
    pub fn preset_maison_renovation() -> Self {
        let mut p = Self {
            title_fr: "Maison 110 m² — Ouverture Séjour & Suite Parentale".into(),
            title_en: "110 m² House — Open Living & Master Suite Renovation".into(),
            levels: vec!["Rez-de-chaussée".into(), "1er Étage / Combles".into()],
            active_level: 0,
            walls: Vec::new(),
            openings: Vec::new(),
            rooms: Vec::new(),
            furniture: Vec::new(),
            show_grid: true,
            snap_to_grid: true,
            grid_step_m: 0.25,
            show_dimensions: true,
            show_underlay: false,
            underlay_opacity: 0.35,
            underlay_scale_m: 12.0,
            next_id: 1,
        };

        // --- LEVEL 0 : REZ-DE-CHAUSSÉE (11.0m x 8.0m) ---
        // Rooms
        let r_sejour = p.alloc_id();
        p.rooms.push(RoomZone {
            id: r_sejour,
            level: 0,
            name_fr: "Séjour & Salon".into(),
            name_en: "Living & Dining Room".into(),
            x: 0.0,
            y: 0.0,
            width: 6.5,
            depth: 5.0,
            ceiling_height: 2.60,
            material: FloorMaterial::OakParquet,
            status: RenoStatus::Existing,
        });
        let r_cuisine = p.alloc_id();
        p.rooms.push(RoomZone {
            id: r_cuisine,
            level: 0,
            name_fr: "Cuisine Ouverte".into(),
            name_en: "Open Kitchen".into(),
            x: 6.5,
            y: 0.0,
            width: 4.5,
            depth: 5.0,
            ceiling_height: 2.60,
            material: FloorMaterial::LightTile,
            status: RenoStatus::New,
        });
        let r_suite = p.alloc_id();
        p.rooms.push(RoomZone {
            id: r_suite,
            level: 0,
            name_fr: "Suite Parentale".into(),
            name_en: "Master Bedroom".into(),
            x: 0.0,
            y: 5.0,
            width: 5.0,
            depth: 3.0,
            ceiling_height: 2.60,
            material: FloorMaterial::OakParquet,
            status: RenoStatus::New,
        });
        let r_sdb = p.alloc_id();
        p.rooms.push(RoomZone {
            id: r_sdb,
            level: 0,
            name_fr: "Salle d'Eau & Douche".into(),
            name_en: "En-suite Bathroom".into(),
            x: 5.0,
            y: 5.0,
            width: 3.2,
            depth: 3.0,
            ceiling_height: 2.60,
            material: FloorMaterial::SlateTile,
            status: RenoStatus::New,
        });
        let r_bureau = p.alloc_id();
        p.rooms.push(RoomZone {
            id: r_bureau,
            level: 0,
            name_fr: "Entrée & Bureau".into(),
            name_en: "Entry & Home Office".into(),
            x: 8.2,
            y: 5.0,
            width: 2.8,
            depth: 3.0,
            ceiling_height: 2.60,
            material: FloorMaterial::PolishedConcrete,
            status: RenoStatus::Existing,
        });

        // Exterior load-bearing walls (Existing)
        let w_north = p.add_wall(
            0,
            0.0,
            0.0,
            11.0,
            0.0,
            0.28,
            2.60,
            true,
            RenoStatus::Existing,
            "Façade Nord",
        );
        let w_east = p.add_wall(
            0,
            11.0,
            0.0,
            11.0,
            8.0,
            0.28,
            2.60,
            true,
            RenoStatus::Existing,
            "Pignon Est",
        );
        let w_south = p.add_wall(
            0,
            11.0,
            8.0,
            0.0,
            8.0,
            0.28,
            2.60,
            true,
            RenoStatus::Existing,
            "Façade Sud Jardin",
        );
        let w_west = p.add_wall(
            0,
            0.0,
            8.0,
            0.0,
            0.0,
            0.28,
            2.60,
            true,
            RenoStatus::Existing,
            "Pignon Ouest",
        );

        // Central spine wall (Existing)
        let w_spine_left = p.add_wall(
            0,
            0.0,
            5.0,
            6.5,
            5.0,
            0.20,
            2.60,
            true,
            RenoStatus::Existing,
            "Refend Central Ouest",
        );
        let w_spine_right = p.add_wall(
            0,
            6.5,
            5.0,
            11.0,
            5.0,
            0.20,
            2.60,
            true,
            RenoStatus::Existing,
            "Refend Central Est",
        );

        // Old partition between Living Room and Kitchen -> TO DEMOLISH (Red!)
        let w_demolish_kitchen = p.add_wall(
            0,
            6.5,
            0.0,
            6.5,
            4.2,
            0.10,
            2.60,
            false,
            RenoStatus::Demolish,
            "Ancienne cloison cuisine à abattre",
        );
        // Old narrow corridor wall -> TO DEMOLISH
        let _w_demolish_corridor = p.add_wall(
            0,
            3.8,
            5.0,
            3.8,
            8.0,
            0.10,
            2.60,
            false,
            RenoStatus::Demolish,
            "Ancienne cloison couloir à démolir",
        );

        // New partitions -> TO CREATE (Green!)
        let w_new_suite = p.add_wall(
            0,
            5.0,
            5.0,
            5.0,
            8.0,
            0.10,
            2.60,
            false,
            RenoStatus::New,
            "Nouvelle cloison acoustique Suite / SdB",
        );
        let w_new_sdb = p.add_wall(
            0,
            8.2,
            5.0,
            8.2,
            8.0,
            0.10,
            2.60,
            false,
            RenoStatus::New,
            "Nouvelle cloison Salle d'eau / Entrée",
        );
        let _w_new_verriere = p.add_wall(
            0,
            6.5,
            0.0,
            6.5,
            1.6,
            0.12,
            2.60,
            false,
            RenoStatus::New,
            "Muret verrière atelier cuisine",
        );

        // Openings (Doors & Windows)
        p.add_opening(
            0,
            w_north,
            0.25,
            OpeningKind::BayWindow,
            RenoStatus::Existing,
            false,
        );
        p.add_opening(
            0,
            w_north,
            0.78,
            OpeningKind::DoubleWindow,
            RenoStatus::Existing,
            false,
        );
        p.add_opening(
            0,
            w_east,
            0.32,
            OpeningKind::SingleWindow,
            RenoStatus::Existing,
            false,
        );
        p.add_opening(
            0,
            w_south,
            0.15,
            OpeningKind::SingleDoor,
            RenoStatus::Existing,
            true,
        );
        p.add_opening(
            0,
            w_south,
            0.50,
            OpeningKind::SingleWindow,
            RenoStatus::New,
            false,
        );
        p.add_opening(
            0,
            w_south,
            0.80,
            OpeningKind::DoubleWindow,
            RenoStatus::Existing,
            false,
        );
        p.add_opening(
            0,
            w_west,
            0.65,
            OpeningKind::BayWindow,
            RenoStatus::New,
            false,
        );

        p.add_opening(
            0,
            w_spine_left,
            0.42,
            OpeningKind::SingleDoor,
            RenoStatus::New,
            false,
        );
        p.add_opening(
            0,
            w_spine_right,
            0.72,
            OpeningKind::OpenPassage,
            RenoStatus::Existing,
            false,
        );
        p.add_opening(
            0,
            w_new_suite,
            0.50,
            OpeningKind::SlidingDoor,
            RenoStatus::New,
            false,
        );
        p.add_opening(
            0,
            w_new_sdb,
            0.45,
            OpeningKind::SingleDoor,
            RenoStatus::New,
            true,
        );
        p.add_opening(
            0,
            w_demolish_kitchen,
            0.75,
            OpeningKind::SingleDoor,
            RenoStatus::Demolish,
            false,
        );

        // Furniture & Equipment
        p.add_furniture(0, FurnitureKind::Sofa, 2.2, 1.8, 0.0, RenoStatus::Existing);
        p.add_furniture(
            0,
            FurnitureKind::CoffeeTable,
            2.2,
            3.0,
            0.0,
            RenoStatus::Existing,
        );
        p.add_furniture(
            0,
            FurnitureKind::DiningTable,
            4.8,
            2.6,
            0.0,
            RenoStatus::Existing,
        );
        p.add_furniture(0, FurnitureKind::WoodStove, 0.6, 4.4, 0.0, RenoStatus::New);

        p.add_furniture(
            0,
            FurnitureKind::KitchenCounter,
            8.8,
            0.45,
            0.0,
            RenoStatus::New,
        );
        p.add_furniture(
            0,
            FurnitureKind::KitchenIsland,
            8.4,
            2.5,
            0.0,
            RenoStatus::New,
        );
        p.add_furniture(0, FurnitureKind::Fridge, 10.4, 0.55, 0.0, RenoStatus::New);

        p.add_furniture(0, FurnitureKind::DoubleBed, 1.8, 6.4, 0.0, RenoStatus::New);
        p.add_furniture(0, FurnitureKind::Wardrobe, 4.4, 6.5, 90.0, RenoStatus::New);

        p.add_furniture(
            0,
            FurnitureKind::WalkInShower,
            5.8,
            7.2,
            0.0,
            RenoStatus::New,
        );
        p.add_furniture(0, FurnitureKind::VanitySink, 7.1, 5.4, 0.0, RenoStatus::New);
        p.add_furniture(0, FurnitureKind::Toilet, 7.7, 7.3, 0.0, RenoStatus::New);

        p.add_furniture(0, FurnitureKind::Desk, 9.6, 6.2, 0.0, RenoStatus::Existing);
        p.add_furniture(
            0,
            FurnitureKind::ElectricalPanel,
            10.6,
            7.6,
            0.0,
            RenoStatus::New,
        );
        p.add_furniture(
            0,
            FurnitureKind::WaterHeater,
            8.6,
            7.4,
            0.0,
            RenoStatus::New,
        );

        // --- LEVEL 1 : COMBLES AMÉNAGÉS (11.0m x 6.0m) ---
        let r_ch2 = p.alloc_id();
        p.rooms.push(RoomZone {
            id: r_ch2,
            level: 1,
            name_fr: "Chambre Enfant 1".into(),
            name_en: "Kids Bedroom 1".into(),
            x: 0.0,
            y: 1.0,
            width: 5.5,
            depth: 6.0,
            ceiling_height: 2.30,
            material: FloorMaterial::OakParquet,
            status: RenoStatus::New,
        });
        let r_ch3 = p.alloc_id();
        p.rooms.push(RoomZone {
            id: r_ch3,
            level: 1,
            name_fr: "Chambre 2 & Mezzanine".into(),
            name_en: "Guest Room & Loft".into(),
            x: 5.5,
            y: 1.0,
            width: 5.5,
            depth: 6.0,
            ceiling_height: 2.30,
            material: FloorMaterial::WarmCarpet,
            status: RenoStatus::New,
        });
        let w1_n = p.add_wall(
            1,
            0.0,
            1.0,
            11.0,
            1.0,
            0.25,
            2.30,
            true,
            RenoStatus::Existing,
            "Mur Nord Combles",
        );
        let _w1_e = p.add_wall(
            1,
            11.0,
            1.0,
            11.0,
            7.0,
            0.25,
            2.30,
            true,
            RenoStatus::Existing,
            "Pignon Est Combles",
        );
        let w1_s = p.add_wall(
            1,
            11.0,
            7.0,
            0.0,
            7.0,
            0.25,
            2.30,
            true,
            RenoStatus::Existing,
            "Mur Sud Combles",
        );
        let _w1_w = p.add_wall(
            1,
            0.0,
            7.0,
            0.0,
            1.0,
            0.25,
            2.30,
            true,
            RenoStatus::Existing,
            "Pignon Ouest Combles",
        );
        let w1_mid = p.add_wall(
            1,
            5.5,
            1.0,
            5.5,
            7.0,
            0.10,
            2.30,
            false,
            RenoStatus::New,
            "Cloison Combles BA13",
        );
        p.add_opening(
            1,
            w1_mid,
            0.5,
            OpeningKind::SingleDoor,
            RenoStatus::New,
            false,
        );
        p.add_opening(
            1,
            w1_n,
            0.3,
            OpeningKind::SingleWindow,
            RenoStatus::New,
            false,
        );
        p.add_opening(
            1,
            w1_s,
            0.7,
            OpeningKind::SingleWindow,
            RenoStatus::New,
            false,
        );
        p.add_furniture(1, FurnitureKind::DoubleBed, 2.2, 3.8, 0.0, RenoStatus::New);
        p.add_furniture(1, FurnitureKind::Desk, 7.8, 3.2, 0.0, RenoStatus::New);

        p
    }

    /// Preset 2: Studio / Appartement Haussmannien 48 m²
    pub fn preset_studio_appartement() -> Self {
        let mut p = Self {
            title_fr: "Appartement 48 m² — Transformation 2 Pièces".into(),
            title_en: "48 m² Apartment — 1-Bedroom Loft Conversion".into(),
            levels: vec!["Étage Principal".into()],
            active_level: 0,
            walls: Vec::new(),
            openings: Vec::new(),
            rooms: Vec::new(),
            furniture: Vec::new(),
            show_grid: true,
            snap_to_grid: true,
            grid_step_m: 0.25,
            show_dimensions: true,
            show_underlay: false,
            underlay_opacity: 0.30,
            underlay_scale_m: 10.0,
            next_id: 1,
        };

        let id1 = p.alloc_id();
        p.rooms.push(RoomZone {
            id: id1,
            level: 0,
            name_fr: "Séjour & Cuisine US".into(),
            name_en: "Living & Open Kitchen".into(),
            x: 0.0,
            y: 0.0,
            width: 5.2,
            depth: 6.0,
            ceiling_height: 2.85,
            material: FloorMaterial::OakParquet,
            status: RenoStatus::Existing,
        });
        let id2 = p.alloc_id();
        p.rooms.push(RoomZone {
            id: id2,
            level: 0,
            name_fr: "Chambre Verrière".into(),
            name_en: "Glass Partition Bedroom".into(),
            x: 5.2,
            y: 0.0,
            width: 2.8,
            depth: 3.8,
            ceiling_height: 2.85,
            material: FloorMaterial::OakParquet,
            status: RenoStatus::New,
        });
        let id3 = p.alloc_id();
        p.rooms.push(RoomZone {
            id: id3,
            level: 0,
            name_fr: "Salle de Bain".into(),
            name_en: "Bathroom".into(),
            x: 5.2,
            y: 3.8,
            width: 2.8,
            depth: 2.2,
            ceiling_height: 2.85,
            material: FloorMaterial::Terracotta,
            status: RenoStatus::New,
        });

        let wn = p.add_wall(
            0,
            0.0,
            0.0,
            8.0,
            0.0,
            0.30,
            2.85,
            true,
            RenoStatus::Existing,
            "Mur Façade Rue",
        );
        let _we = p.add_wall(
            0,
            8.0,
            0.0,
            8.0,
            6.0,
            0.25,
            2.85,
            true,
            RenoStatus::Existing,
            "Mitoyen Est",
        );
        let ws = p.add_wall(
            0,
            8.0,
            6.0,
            0.0,
            6.0,
            0.25,
            2.85,
            true,
            RenoStatus::Existing,
            "Mur Cour",
        );
        let _ww = p.add_wall(
            0,
            0.0,
            6.0,
            0.0,
            0.0,
            0.25,
            2.85,
            true,
            RenoStatus::Existing,
            "Mitoyen Ouest",
        );

        let _w_old = p.add_wall(
            0,
            3.2,
            0.0,
            3.2,
            6.0,
            0.10,
            2.85,
            false,
            RenoStatus::Demolish,
            "Ancienne cloison alcôve",
        );
        let w_new1 = p.add_wall(
            0,
            5.2,
            0.0,
            5.2,
            6.0,
            0.10,
            2.85,
            false,
            RenoStatus::New,
            "Cloison Chambre & SdB",
        );
        let _w_new2 = p.add_wall(
            0,
            5.2,
            3.8,
            8.0,
            3.8,
            0.10,
            2.85,
            false,
            RenoStatus::New,
            "Séparation SdB",
        );

        p.add_opening(
            0,
            wn,
            0.30,
            OpeningKind::DoubleWindow,
            RenoStatus::Existing,
            false,
        );
        p.add_opening(
            0,
            wn,
            0.78,
            OpeningKind::SingleWindow,
            RenoStatus::Existing,
            false,
        );
        p.add_opening(
            0,
            ws,
            0.82,
            OpeningKind::SingleDoor,
            RenoStatus::Existing,
            false,
        );
        p.add_opening(
            0,
            w_new1,
            0.35,
            OpeningKind::SlidingDoor,
            RenoStatus::New,
            false,
        );
        p.add_opening(
            0,
            w_new1,
            0.80,
            OpeningKind::SingleDoor,
            RenoStatus::New,
            true,
        );

        p.add_furniture(0, FurnitureKind::Sofa, 1.8, 2.2, 0.0, RenoStatus::Existing);
        p.add_furniture(
            0,
            FurnitureKind::KitchenCounter,
            2.4,
            5.5,
            0.0,
            RenoStatus::New,
        );
        p.add_furniture(0, FurnitureKind::DoubleBed, 6.6, 1.8, 0.0, RenoStatus::New);
        p.add_furniture(0, FurnitureKind::Bathtub, 6.6, 5.2, 0.0, RenoStatus::New);

        p
    }

    /// Preset 0 (Par défaut à l'ouverture) : Plan 2D vierge pour créer soi-même ses pièces en 1 clic ou à la souris
    pub fn preset_creation_libre() -> Self {
        Self {
            title_fr: "Mon Plan — Création Libre".into(),
            title_en: "My Floor Plan — Free Creation".into(),
            levels: vec!["Rez-de-chaussée".into(), "Étage 1".into()],
            active_level: 0,
            walls: Vec::new(),
            openings: Vec::new(),
            rooms: Vec::new(),
            furniture: Vec::new(),
            show_grid: true,
            snap_to_grid: true,
            grid_step_m: 0.25,
            show_dimensions: true,
            show_underlay: false,
            underlay_opacity: 0.35,
            underlay_scale_m: 12.0,
            next_id: 1,
        }
    }

    /// Preset 3: Plan vierge prêt à tracer (Plateau 10×8 m)
    pub fn preset_blank_shell() -> Self {
        let mut p = Self {
            title_fr: "Nouveau Projet — Plateau Brut 10×8 m".into(),
            title_en: "New Project — 10×8 m Open Shell".into(),
            levels: vec!["Rez-de-chaussée".into(), "Étage 1".into()],
            active_level: 0,
            walls: Vec::new(),
            openings: Vec::new(),
            rooms: Vec::new(),
            furniture: Vec::new(),
            show_grid: true,
            snap_to_grid: true,
            grid_step_m: 0.25,
            show_dimensions: true,
            show_underlay: false,
            underlay_opacity: 0.35,
            underlay_scale_m: 12.0,
            next_id: 1,
        };
        let r_id = p.alloc_id();
        p.rooms.push(RoomZone {
            id: r_id,
            level: 0,
            name_fr: "Plateau Brut".into(),
            name_en: "Main Floor".into(),
            x: 0.0,
            y: 0.0,
            width: 10.0,
            depth: 8.0,
            ceiling_height: 2.50,
            material: FloorMaterial::PolishedConcrete,
            status: RenoStatus::Existing,
        });
        let wn = p.add_wall(
            0,
            0.0,
            0.0,
            10.0,
            0.0,
            0.25,
            2.50,
            true,
            RenoStatus::Existing,
            "Mur Nord",
        );
        let _we = p.add_wall(
            0,
            10.0,
            0.0,
            10.0,
            8.0,
            0.25,
            2.50,
            true,
            RenoStatus::Existing,
            "Mur Est",
        );
        let ws = p.add_wall(
            0,
            10.0,
            8.0,
            0.0,
            8.0,
            0.25,
            2.50,
            true,
            RenoStatus::Existing,
            "Mur Sud",
        );
        let _ww = p.add_wall(
            0,
            0.0,
            8.0,
            0.0,
            0.0,
            0.25,
            2.50,
            true,
            RenoStatus::Existing,
            "Mur Ouest",
        );
        p.add_opening(
            0,
            ws,
            0.5,
            OpeningKind::SingleDoor,
            RenoStatus::Existing,
            false,
        );
        p.add_opening(
            0,
            wn,
            0.5,
            OpeningKind::BayWindow,
            RenoStatus::Existing,
            false,
        );
        p
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_wall(
        &mut self,
        level: usize,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        thickness: f32,
        height: f32,
        load_bearing: bool,
        status: RenoStatus,
        name: &str,
    ) -> u32 {
        let id = self.alloc_id();
        self.walls.push(Wall {
            id,
            level,
            x1,
            y1,
            x2,
            y2,
            thickness,
            height,
            load_bearing,
            status,
            name: name.to_owned(),
        });
        id
    }

    pub fn add_opening(
        &mut self,
        level: usize,
        wall_id: u32,
        offset_t: f32,
        kind: OpeningKind,
        status: RenoStatus,
        flip_swing: bool,
    ) -> u32 {
        let id = self.alloc_id();
        self.openings.push(Opening {
            id,
            level,
            wall_id,
            offset_t: offset_t.clamp(0.08, 0.92),
            width: kind.default_width(),
            height: kind.default_height(),
            sill_height: kind.default_sill(),
            kind,
            status,
            flip_swing,
        });
        id
    }

    pub fn add_furniture(
        &mut self,
        level: usize,
        kind: FurnitureKind,
        x: f32,
        y: f32,
        rotation_deg: f32,
        status: RenoStatus,
    ) -> u32 {
        let id = self.alloc_id();
        let (width, depth, height) = kind.default_dims();
        self.furniture.push(FurnitureItem {
            id,
            level,
            kind,
            label_custom: String::new(),
            x,
            y,
            width,
            depth,
            height,
            rotation_deg,
            status,
            cost: kind.default_cost(),
        });
        id
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_room(
        &mut self,
        level: usize,
        name_fr: &str,
        name_en: &str,
        x: f32,
        y: f32,
        width: f32,
        depth: f32,
        material: FloorMaterial,
        status: RenoStatus,
    ) -> u32 {
        let id = self.alloc_id();
        self.rooms.push(RoomZone {
            id,
            level,
            name_fr: name_fr.to_owned(),
            name_en: name_en.to_owned(),
            x: x.min(x + width),
            y: y.min(y + depth),
            width: width.abs().max(0.5),
            depth: depth.abs().max(0.5),
            ceiling_height: 2.60,
            material,
            status,
        });
        id
    }

    /// Crée en un seul geste une pièce complète (sol + 4 cloisons autour) — idéal pour débutants
    #[allow(clippy::too_many_arguments)]
    pub fn add_boxed_room(
        &mut self,
        level: usize,
        name_fr: &str,
        name_en: &str,
        x: f32,
        y: f32,
        width: f32,
        depth: f32,
        material: FloorMaterial,
        status: RenoStatus,
    ) -> u32 {
        let rx = x.min(x + width);
        let ry = y.min(y + depth);
        let rw = width.abs().max(1.0);
        let rd = depth.abs().max(1.0);
        let room_id = self.add_room(level, name_fr, name_en, rx, ry, rw, rd, material, status);
        let wall_name = format!("Cloison {name_fr}");
        self.add_wall(
            level,
            rx,
            ry,
            rx + rw,
            ry,
            0.10,
            2.60,
            false,
            status,
            &wall_name,
        );
        self.add_wall(
            level,
            rx + rw,
            ry,
            rx + rw,
            ry + rd,
            0.10,
            2.60,
            false,
            status,
            &wall_name,
        );
        self.add_wall(
            level,
            rx + rw,
            ry + rd,
            rx,
            ry + rd,
            0.10,
            2.60,
            false,
            status,
            &wall_name,
        );
        self.add_wall(
            level,
            rx,
            ry + rd,
            rx,
            ry,
            0.10,
            2.60,
            false,
            status,
            &wall_name,
        );
        room_id
    }

    /// Calcule la boîte englobante de toutes les pièces et murs du niveau actif
    pub fn floor_bounding_box(&self, level: usize) -> Option<(f32, f32, f32, f32)> {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut found = false;

        for r in &self.rooms {
            if r.level == level {
                min_x = min_x.min(r.x);
                min_y = min_y.min(r.y);
                max_x = max_x.max(r.x + r.width);
                max_y = max_y.max(r.y + r.depth);
                found = true;
            }
        }
        for w in &self.walls {
            if w.level == level {
                min_x = min_x.min(w.x1.min(w.x2));
                min_y = min_y.min(w.y1.min(w.y2));
                max_x = max_x.max(w.x1.max(w.x2));
                max_y = max_y.max(w.y1.max(w.y2));
                found = true;
            }
        }
        if found {
            Some((min_x, min_y, max_x, max_y))
        } else {
            None
        }
    }

    /// Trouve automatiquement un emplacement libre accolé aux pièces existantes ou centré sur la vue
    pub fn find_free_room_spot(
        &self,
        level: usize,
        width: f32,
        depth: f32,
        hint_x: f32,
        hint_y: f32,
    ) -> (f32, f32) {
        let level_rooms: Vec<&RoomZone> = self.rooms.iter().filter(|r| r.level == level).collect();
        if level_rooms.is_empty() {
            // Centre la première pièce pile sous le regard actuel de l'utilisateur (aligné sur grille 0.25m)
            let cx = ((hint_x - width * 0.5) / 0.25).round() * 0.25;
            let cy = ((hint_y - depth * 0.5) / 0.25).round() * 0.25;
            return (cx.max(0.5), cy.max(0.5));
        }

        let overlaps = |cx: f32, cy: f32| -> bool {
            let eps = 0.05;
            level_rooms.iter().any(|r| {
                // Skip giant outer shell rooms (> 70 m²) so users can place rooms inside a blank shell if desired
                if r.area_m2() > 70.0 && level_rooms.len() == 1 {
                    return false;
                }
                !(cx + width <= r.x + eps
                    || cx >= r.x + r.width - eps
                    || cy + depth <= r.y + eps
                    || cy >= r.y + r.depth - eps)
            })
        };

        // 1. Tente de coller la pièce à droite ou en bas de la dernière pièce posée
        for r in level_rooms.iter().rev() {
            if r.area_m2() > 70.0 && level_rooms.len() == 1 {
                return (r.x + 0.5, r.y + 0.5);
            }
            let candidates = [
                (r.x + r.width, r.y),
                (r.x, r.y + r.depth),
                (r.x - width, r.y),
                (r.x, r.y - depth),
            ];
            for (cx, cy) in candidates {
                if cx >= 0.0 && cy >= 0.0 && !overlaps(cx, cy) {
                    return (cx, cy);
                }
            }
        }

        // 2. Fallback : à droite de la boîte englobante
        if let Some((_min_x, _min_y, max_x, min_y)) = self.floor_bounding_box(level) {
            let cx = (max_x / 0.25).ceil() * 0.25;
            let cy = (min_y / 0.25).floor() * 0.25;
            if !overlaps(cx, cy) {
                return (cx, cy);
            }
        }

        (hint_x.max(1.0), hint_y.max(1.0))
    }

    /// Crée en un clic une pièce prête à l'emploi (ex: Chambre 3,5×3m, Salon 5×4m) avec ses 4 murs et sa porte, accolée proprement sans superposition
    #[allow(clippy::too_many_arguments)]
    pub fn add_ready_room(
        &mut self,
        level: usize,
        name_fr: &str,
        name_en: &str,
        hint_x: f32,
        hint_y: f32,
        width: f32,
        depth: f32,
        material: FloorMaterial,
        status: RenoStatus,
    ) -> u32 {
        let (rx, ry) = self.find_free_room_spot(level, width, depth, hint_x, hint_y);
        let room_id = self.add_room(
            level, name_fr, name_en, rx, ry, width, depth, material, status,
        );
        let wall_name = format!("Mur {name_fr}");
        self.add_wall(
            level,
            rx,
            ry,
            rx + width,
            ry,
            0.10,
            2.60,
            false,
            status,
            &wall_name,
        );
        self.add_wall(
            level,
            rx + width,
            ry,
            rx + width,
            ry + depth,
            0.10,
            2.60,
            false,
            status,
            &wall_name,
        );
        let south_wall = self.add_wall(
            level,
            rx + width,
            ry + depth,
            rx,
            ry + depth,
            0.10,
            2.60,
            false,
            status,
            &wall_name,
        );
        self.add_wall(
            level,
            rx,
            ry + depth,
            rx,
            ry,
            0.10,
            2.60,
            false,
            status,
            &wall_name,
        );
        self.add_opening(
            level,
            south_wall,
            0.5,
            OpeningKind::SingleDoor,
            status,
            false,
        );
        room_id
    }

    /// Déplace ou redimensionne une pièce ET déplace/étire automatiquement ses 4 murs associés (ainsi que les meubles contenus si déplacement)
    pub fn update_room_rect(
        &mut self,
        room_id: u32,
        new_x: f32,
        new_y: f32,
        new_w: f32,
        new_d: f32,
    ) {
        let Some(r_idx) = self.rooms.iter().position(|r| r.id == room_id) else {
            return;
        };
        let level = self.rooms[r_idx].level;
        let old_x = self.rooms[r_idx].x;
        let old_y = self.rooms[r_idx].y;
        let old_w = self.rooms[r_idx].width;
        let old_d = self.rooms[r_idx].depth;
        let nw = new_w.max(0.75);
        let nd = new_d.max(0.75);

        let is_pure_move = (nw - old_w).abs() < 0.01 && (nd - old_d).abs() < 0.01;
        let dx = new_x - old_x;
        let dy = new_y - old_y;

        let eps = 0.16;
        let on_old_perimeter = |px: f32, py: f32| -> bool {
            let in_x = px >= old_x - eps && px <= old_x + old_w + eps;
            let in_y = py >= old_y - eps && py <= old_y + old_d + eps;
            let near_x_edge = (px - old_x).abs() <= eps || (px - (old_x + old_w)).abs() <= eps;
            let near_y_edge = (py - old_y).abs() <= eps || (py - (old_y + old_d)).abs() <= eps;
            (in_x && near_y_edge) || (in_y && near_x_edge)
        };

        for w in &mut self.walls {
            if w.level != level {
                continue;
            }
            if on_old_perimeter(w.x1, w.y1) && on_old_perimeter(w.x2, w.y2) {
                if (w.x1 - old_x).abs() <= eps {
                    w.x1 = new_x;
                } else if (w.x1 - (old_x + old_w)).abs() <= eps {
                    w.x1 = new_x + nw;
                } else if old_w > 0.1 {
                    w.x1 = new_x + ((w.x1 - old_x) / old_w) * nw;
                }

                if (w.y1 - old_y).abs() <= eps {
                    w.y1 = new_y;
                } else if (w.y1 - (old_y + old_d)).abs() <= eps {
                    w.y1 = new_y + nd;
                } else if old_d > 0.1 {
                    w.y1 = new_y + ((w.y1 - old_y) / old_d) * nd;
                }

                if (w.x2 - old_x).abs() <= eps {
                    w.x2 = new_x;
                } else if (w.x2 - (old_x + old_w)).abs() <= eps {
                    w.x2 = new_x + nw;
                } else if old_w > 0.1 {
                    w.x2 = new_x + ((w.x2 - old_x) / old_w) * nw;
                }

                if (w.y2 - old_y).abs() <= eps {
                    w.y2 = new_y;
                } else if (w.y2 - (old_y + old_d)).abs() <= eps {
                    w.y2 = new_y + nd;
                } else if old_d > 0.1 {
                    w.y2 = new_y + ((w.y2 - old_y) / old_d) * nd;
                }
            }
        }

        if is_pure_move {
            for f in &mut self.furniture {
                if f.level == level
                    && f.x >= old_x
                    && f.x <= old_x + old_w
                    && f.y >= old_y
                    && f.y <= old_y + old_d
                {
                    f.x += dx;
                    f.y += dy;
                }
            }
        }

        self.rooms[r_idx].x = new_x;
        self.rooms[r_idx].y = new_y;
        self.rooms[r_idx].width = nw;
        self.rooms[r_idx].depth = nd;
    }

    /// Supprime une pièce ainsi que ses murs dédiés et leurs portes/fenêtres
    pub fn delete_room_with_walls(&mut self, room_id: u32) {
        let Some(r) = self.rooms.iter().find(|r| r.id == room_id).cloned() else {
            return;
        };
        let eps = 0.16;
        let on_room = |px: f32, py: f32| -> bool {
            let in_x = px >= r.x - eps && px <= r.x + r.width + eps;
            let in_y = py >= r.y - eps && py <= r.y + r.depth + eps;
            let near_x = (px - r.x).abs() <= eps || (px - (r.x + r.width)).abs() <= eps;
            let near_y = (py - r.y).abs() <= eps || (py - (r.y + r.depth)).abs() <= eps;
            (in_x && near_y) || (in_y && near_x)
        };
        let mut removed_walls = Vec::new();
        self.walls.retain(|w| {
            if w.level == r.level && on_room(w.x1, w.y1) && on_room(w.x2, w.y2) {
                removed_walls.push(w.id);
                false
            } else {
                true
            }
        });
        self.openings
            .retain(|o| !removed_walls.contains(&o.wall_id));
        self.rooms.retain(|rm| rm.id != room_id);
    }

    pub fn snap_point(&self, x: f32, y: f32, phase: ViewPhase) -> (f32, f32) {
        // First snap to existing wall endpoints on active level within 25 cm
        let snap_radius = 0.25;
        for w in &self.walls {
            if w.level != self.active_level || !w.status.is_visible_in(phase) {
                continue;
            }
            for &(wx, wy) in &[(w.x1, w.y1), (w.x2, w.y2)] {
                if ((x - wx).powi(2) + (y - wy).powi(2)).sqrt() <= snap_radius {
                    return (wx, wy);
                }
            }
        }
        if self.snap_to_grid && self.grid_step_m > 0.01 {
            let step = self.grid_step_m;
            ((x / step).round() * step, (y / step).round() * step)
        } else {
            ((x * 100.0).round() / 100.0, (y * 100.0).round() / 100.0)
        }
    }

    pub fn hit_test(&self, x: f32, y: f32, phase: ViewPhase) -> Option<SelectedEntity> {
        let level = self.active_level;

        // 1. Check openings first
        for op in self.openings.iter().rev() {
            if op.level != level || !op.status.is_visible_in(phase) {
                continue;
            }
            if let Some(w) = self.walls.iter().find(|w| w.id == op.wall_id) {
                let (ox, oy) = w.point_at(op.offset_t);
                let dist = ((x - ox).powi(2) + (y - oy).powi(2)).sqrt();
                if dist <= (op.width * 0.55).max(0.35) {
                    return Some(SelectedEntity::Opening(op.id));
                }
            }
        }

        // 2. Check furniture
        for item in self.furniture.iter().rev() {
            if item.level != level || !item.status.is_visible_in(phase) {
                continue;
            }
            let rad = -item.rotation_deg.to_radians();
            let dx = x - item.x;
            let dy = y - item.y;
            let lx = dx * rad.cos() - dy * rad.sin();
            let ly = dx * rad.sin() + dy * rad.cos();
            if lx.abs() <= item.width * 0.55 && ly.abs() <= item.depth * 0.55 {
                return Some(SelectedEntity::Furniture(item.id));
            }
        }

        // 3. Check walls
        for w in self.walls.iter().rev() {
            if w.level != level || !w.status.is_visible_in(phase) {
                continue;
            }
            let (_, dist) = w.project_point(x, y);
            if dist <= (w.thickness * 0.75).max(0.22) {
                return Some(SelectedEntity::Wall(w.id));
            }
        }

        // 4. Check room zones
        for r in self.rooms.iter().rev() {
            if r.level != level || !r.status.is_visible_in(phase) {
                continue;
            }
            if x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.depth {
                return Some(SelectedEntity::Room(r.id));
            }
        }

        None
    }

    pub fn closest_wall(&self, x: f32, y: f32, phase: ViewPhase) -> Option<(u32, f32)> {
        let mut best: Option<(u32, f32, f32)> = None;
        for w in &self.walls {
            if w.level != self.active_level || !w.status.is_visible_in(phase) {
                continue;
            }
            let (t, dist) = w.project_point(x, y);
            if dist <= 0.65 && (best.is_none() || dist < best.unwrap().2) {
                best = Some((w.id, t.clamp(0.10, 0.90), dist));
            }
        }
        best.map(|(id, t, _)| (id, t))
    }

    pub fn delete_entity(&mut self, entity: SelectedEntity) -> bool {
        match entity {
            SelectedEntity::Wall(id) => {
                let before = self.walls.len();
                self.walls.retain(|w| w.id != id);
                self.openings.retain(|o| o.wall_id != id);
                self.walls.len() < before
            }
            SelectedEntity::Opening(id) => {
                let before = self.openings.len();
                self.openings.retain(|o| o.id != id);
                self.openings.len() < before
            }
            SelectedEntity::Room(id) => {
                let before = self.rooms.len();
                self.rooms.retain(|r| r.id != id);
                self.rooms.len() < before
            }
            SelectedEntity::Furniture(id) => {
                let before = self.furniture.len();
                self.furniture.retain(|f| f.id != id);
                self.furniture.len() < before
            }
        }
    }

    /// Computes full renovation Bill of Quantities (Métré & Devis)
    pub fn compute_estimate(&self) -> EstimateSummary {
        let mut s = EstimateSummary::default();

        // 1. Rooms (Floor area, Carrez area, Flooring renovation costs)
        for r in &self.rooms {
            if r.status == RenoStatus::Demolish {
                continue;
            }
            let area = r.area_m2();
            s.floor_area_m2 += area;
            s.carrez_area_m2 += r.carrez_m2();

            if r.status == RenoStatus::New {
                let rate = r.material.cost_per_m2();
                let cost = area * rate;
                s.flooring_cost += cost;
                s.lines.push(EstimateLine {
                    category_fr: "Sols & Revêtements".into(),
                    category_en: "Flooring & Finishes".into(),
                    label_fr: format!("{} — {}", r.name_fr, r.material.label(Lang::Fr)),
                    label_en: format!("{} — {}", r.name_en, r.material.label(Lang::En)),
                    quantity: format!("{area:.1} m²"),
                    unit_price: format!("{rate:.0} €/m²"),
                    total_cost: cost,
                    status: r.status,
                });
            }
        }

        // 2. Walls (Demolition, New Partitions, Paintable surfaces)
        for w in &self.walls {
            let len = w.length();
            let gross_area = len * w.height;
            let openings_area: f32 = self
                .openings
                .iter()
                .filter(|o| o.wall_id == w.id && o.status != RenoStatus::Demolish)
                .map(|o| o.width * o.height)
                .sum();
            let net_area = (gross_area - openings_area).max(0.0);

            match w.status {
                RenoStatus::Demolish => {
                    s.demolish_wall_linear_m += len;
                    let vol = len * w.thickness * w.height;
                    s.demolish_volume_m3 += vol;
                    let rate = if w.load_bearing { 185.0 } else { 52.0 };
                    let cost = gross_area * rate;
                    s.demolish_cost += cost;
                    s.lines.push(EstimateLine {
                        category_fr: "Démolition & Évacuation".into(),
                        category_en: "Demolition & Disposal".into(),
                        label_fr: format!("Dépose : {}", w.name),
                        label_en: format!("Demolish: {}", w.name),
                        quantity: format!("{len:.2} ml ({vol:.2} m³)"),
                        unit_price: format!("{rate:.0} €/m²"),
                        total_cost: cost,
                        status: RenoStatus::Demolish,
                    });
                }
                RenoStatus::New => {
                    s.new_wall_linear_m += len;
                    s.paint_wall_area_m2 += net_area * 2.0;
                    let rate = if w.load_bearing { 145.0 } else { 68.0 };
                    let cost = net_area * rate;
                    s.build_wall_cost += cost;
                    s.lines.push(EstimateLine {
                        category_fr: "Cloisons & Maçonnerie".into(),
                        category_en: "Partitions & Masonry".into(),
                        label_fr: format!("Création BA13/ Isolation : {}", w.name),
                        label_en: format!("New Partition & Insulation: {}", w.name),
                        quantity: format!("{net_area:.1} m² ({len:.2} ml)"),
                        unit_price: format!("{rate:.0} €/m²"),
                        total_cost: cost,
                        status: RenoStatus::New,
                    });
                }
                RenoStatus::Existing => {
                    s.paint_wall_area_m2 += net_area;
                }
            }
        }

        // 3. Openings (Doors & Windows)
        for op in &self.openings {
            if op.status != RenoStatus::Demolish {
                s.openings_count += 1;
            }
            if op.status == RenoStatus::New {
                let cost = op.kind.default_cost();
                s.openings_cost += cost;
                s.lines.push(EstimateLine {
                    category_fr: "Menuiseries & Ouvertures".into(),
                    category_en: "Doors & Windows".into(),
                    label_fr: op.kind.label(Lang::Fr).into(),
                    label_en: op.kind.label(Lang::En).into(),
                    quantity: format!("{:.0}×{:.0} cm", op.width * 100.0, op.height * 100.0),
                    unit_price: format!("{cost:.0} €/u"),
                    total_cost: cost,
                    status: RenoStatus::New,
                });
            }
        }

        // 4. Furniture & Technical Equipment
        for item in &self.furniture {
            if item.status != RenoStatus::Demolish {
                s.furniture_count += 1;
            }
            if item.status == RenoStatus::New {
                s.furniture_cost += item.cost;
                s.lines.push(EstimateLine {
                    category_fr: "Équipements & Mobilier".into(),
                    category_en: "Fixtures & Furniture".into(),
                    label_fr: item.display_name(Lang::Fr),
                    label_en: item.display_name(Lang::En),
                    quantity: "1 u".into(),
                    unit_price: format!("{:.0} €", item.cost),
                    total_cost: item.cost,
                    status: RenoStatus::New,
                });
            }
        }

        s.total_cost = s.demolish_cost
            + s.build_wall_cost
            + s.flooring_cost
            + s.openings_cost
            + s.furniture_cost;
        s
    }

    /// Exports clean architectural SVG blueprint
    pub fn export_svg(&self, phase: ViewPhase, lang: Lang) -> String {
        let scale = 70.0_f32; // 70 px per meter
        let pad = 80.0_f32;
        let width = 1000.0_f32;
        let height = 760.0_f32;
        let est = self.compute_estimate();

        let mut svg = String::new();
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}" font-family="Inter, sans-serif">"#
        ));
        svg.push_str(&format!(
            r##"<rect width="{width}" height="{height}" fill="#F8FAFC"/>"##
        ));

        // Grid
        for i in 0..=14 {
            let x = pad + i as f32 * scale;
            svg.push_str(&format!(
                r##"<line x1="{x}" y1="{pad}" x2="{x}" y2="{}" stroke="#E2E8F0" stroke-width="1"/>"##,
                height - pad
            ));
        }
        for j in 0..=9 {
            let y = pad + j as f32 * scale;
            svg.push_str(&format!(
                r##"<line x1="{pad}" y1="{y}" x2="{}" y2="{y}" stroke="#E2E8F0" stroke-width="1"/>"##,
                width - pad
            ));
        }

        // Rooms
        for r in &self.rooms {
            if r.level != self.active_level || !r.status.is_visible_in(phase) {
                continue;
            }
            let rx = pad + r.x * scale;
            let ry = pad + r.y * scale;
            let rw = r.width * scale;
            let rd = r.depth * scale;
            let (cr, cg, cb) = r.material.rgb();
            svg.push_str(&format!(
                r##"<rect x="{rx:.1}" y="{ry:.1}" width="{rw:.1}" height="{rd:.1}" fill="rgb({cr},{cg},{cb})" fill-opacity="0.28" stroke="#94A3B8" stroke-dasharray="4 2"/>"##
            ));
            svg.push_str(&format!(
                r##"<text x="{:.1}" y="{:.1}" text-anchor="middle" font-size="13" font-weight="700" fill="#1E293B">{}</text>"##,
                rx + rw * 0.5,
                ry + rd * 0.5 - 4.0,
                r.name(lang)
            ));
            svg.push_str(&format!(
                r##"<text x="{:.1}" y="{:.1}" text-anchor="middle" font-size="11" fill="#475569">{:.1} m²</text>"##,
                rx + rw * 0.5,
                ry + rd * 0.5 + 12.0,
                r.area_m2()
            ));
        }

        // Walls
        for w in &self.walls {
            if w.level != self.active_level || !w.status.is_visible_in(phase) {
                continue;
            }
            let x1 = pad + w.x1 * scale;
            let y1 = pad + w.y1 * scale;
            let x2 = pad + w.x2 * scale;
            let y2 = pad + w.y2 * scale;
            let sw = (w.thickness * scale).max(4.0);
            let (stroke, dash) = match w.status {
                RenoStatus::Existing => ("#334155", ""),
                RenoStatus::Demolish => ("#DC2626", r#"stroke-dasharray="8 5""#),
                RenoStatus::New => ("#059669", ""),
            };
            svg.push_str(&format!(
                r##"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{stroke}" stroke-width="{sw:.1}" stroke-linecap="square" {dash}/>"##
            ));
        }

        // Title block (Cartouche)
        svg.push_str(r##"<rect x="20" y="16" width="960" height="44" rx="6" fill="#0F172A"/>"##);
        svg.push_str(&format!(
            r##"<text x="36" y="43" font-size="15" font-weight="700" fill="#F8FAFC">RenovaPlan — {}</text>"##,
            self.title(lang)
        ));
        svg.push_str(&format!(
            r##"<text x="960" y="43" text-anchor="end" font-size="13" font-weight="600" fill="#38BDF8">Carrez: {:.1} m² | Budget: {:.0} €</text>"##,
            est.carrez_area_m2, est.total_cost
        ));

        svg.push_str("</svg>");
        svg
    }

    /// Exports Bill of Quantities as CSV
    pub fn export_csv(&self, lang: Lang) -> String {
        let est = self.compute_estimate();
        let mut out = String::new();
        match lang {
            Lang::Fr => out.push_str("Catégorie;Poste;Statut;Quantité;Prix Unitaire;Total EUR\n"),
            Lang::En => out.push_str("Category;Item;Status;Quantity;Unit Price;Total EUR\n"),
        }
        for line in &est.lines {
            let (cat, lbl) = match lang {
                Lang::Fr => (&line.category_fr, &line.label_fr),
                Lang::En => (&line.category_en, &line.label_en),
            };
            out.push_str(&format!(
                "{};{};{};{};{};{:.2}\n",
                cat,
                lbl,
                line.status.label(lang),
                line.quantity,
                line.unit_price,
                line.total_cost
            ));
        }
        out
    }
}
