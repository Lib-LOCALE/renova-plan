use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RenoStatus {
    #[default]
    Existing,
    Demolish,
    New,
}

impl RenoStatus {
    pub fn is_visible_in(self, phase: ViewPhase) -> bool {
        match phase {
            ViewPhase::Chantier => true,
            ViewPhase::Before => matches!(self, Self::Existing | Self::Demolish),
            ViewPhase::After => matches!(self, Self::Existing | Self::New),
        }
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Self::Existing, Lang::Fr) => "Existant (conservé)",
            (Self::Existing, Lang::En) => "Existing (kept)",
            (Self::Demolish, Lang::Fr) => "À démolir (enlever)",
            (Self::Demolish, Lang::En) => "To demolish (remove)",
            (Self::New, Lang::Fr) => "Nouveau (à créer)",
            (Self::New, Lang::En) => "New (to build)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ViewPhase {
    #[default]
    Chantier,
    Before,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Lang {
    #[default]
    Fr,
    En,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OpeningKind {
    #[default]
    SingleDoor,
    DoubleDoor,
    SlidingDoor,
    OpenPassage,
    SingleWindow,
    DoubleWindow,
    BayWindow,
}

impl OpeningKind {
    pub fn is_door(self) -> bool {
        matches!(
            self,
            Self::SingleDoor | Self::DoubleDoor | Self::SlidingDoor | Self::OpenPassage
        )
    }

    pub fn default_width(self) -> f32 {
        match self {
            Self::SingleDoor => 0.83,
            Self::DoubleDoor => 1.40,
            Self::SlidingDoor => 0.90,
            Self::OpenPassage => 1.20,
            Self::SingleWindow => 0.90,
            Self::DoubleWindow => 1.20,
            Self::BayWindow => 2.15,
        }
    }

    pub fn default_height(self) -> f32 {
        match self {
            Self::SingleDoor | Self::DoubleDoor | Self::SlidingDoor | Self::OpenPassage => 2.04,
            Self::SingleWindow | Self::DoubleWindow => 1.25,
            Self::BayWindow => 2.15,
        }
    }

    pub fn default_sill(self) -> f32 {
        match self {
            Self::SingleDoor
            | Self::DoubleDoor
            | Self::SlidingDoor
            | Self::OpenPassage
            | Self::BayWindow => 0.0,
            Self::SingleWindow | Self::DoubleWindow => 0.90,
        }
    }

    pub fn default_cost(self) -> f32 {
        match self {
            Self::SingleDoor => 320.0,
            Self::DoubleDoor => 640.0,
            Self::SlidingDoor => 580.0,
            Self::OpenPassage => 180.0,
            Self::SingleWindow => 450.0,
            Self::DoubleWindow => 690.0,
            Self::BayWindow => 1450.0,
        }
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Self::SingleDoor, Lang::Fr) => "Porte simple (83 cm)",
            (Self::SingleDoor, Lang::En) => "Single Door (83 cm)",
            (Self::DoubleDoor, Lang::Fr) => "Double porte (140 cm)",
            (Self::DoubleDoor, Lang::En) => "Double Door (140 cm)",
            (Self::SlidingDoor, Lang::Fr) => "Porte coulissante (90 cm)",
            (Self::SlidingDoor, Lang::En) => "Sliding Door (90 cm)",
            (Self::OpenPassage, Lang::Fr) => "Grande ouverture / Arche",
            (Self::OpenPassage, Lang::En) => "Open Archway / Passage",
            (Self::SingleWindow, Lang::Fr) => "Petite fenêtre (90 cm)",
            (Self::SingleWindow, Lang::En) => "Small Window (90 cm)",
            (Self::DoubleWindow, Lang::Fr) => "Grande fenêtre (120 cm)",
            (Self::DoubleWindow, Lang::En) => "Standard Window (120 cm)",
            (Self::BayWindow, Lang::Fr) => "Baie vitrée (215 cm)",
            (Self::BayWindow, Lang::En) => "Sliding Glass Door (215 cm)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FloorMaterial {
    #[default]
    OakParquet,
    LightTile,
    SlateTile,
    PolishedConcrete,
    Terracotta,
    WarmCarpet,
}

impl FloorMaterial {
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            Self::OakParquet => (206, 168, 122),
            Self::LightTile => (230, 232, 236),
            Self::SlateTile => (108, 118, 132),
            Self::PolishedConcrete => (178, 184, 190),
            Self::Terracotta => (196, 120, 94),
            Self::WarmCarpet => (192, 182, 168),
        }
    }

    pub fn cost_per_m2(self) -> f32 {
        match self {
            Self::OakParquet => 65.0,
            Self::LightTile => 55.0,
            Self::SlateTile => 62.0,
            Self::PolishedConcrete => 78.0,
            Self::Terracotta => 58.0,
            Self::WarmCarpet => 38.0,
        }
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Self::OakParquet, Lang::Fr) => "Parquet en chêne clair",
            (Self::OakParquet, Lang::En) => "Light Oak Parquet",
            (Self::LightTile, Lang::Fr) => "Carrelage clair",
            (Self::LightTile, Lang::En) => "Light Floor Tile",
            (Self::SlateTile, Lang::Fr) => "Carrelage gris ardoise",
            (Self::SlateTile, Lang::En) => "Dark Slate Tile",
            (Self::PolishedConcrete, Lang::Fr) => "Sol béton lissé",
            (Self::PolishedConcrete, Lang::En) => "Smooth Concrete",
            (Self::Terracotta, Lang::Fr) => "Terre cuite / Tomettes",
            (Self::Terracotta, Lang::En) => "Terracotta Tile",
            (Self::WarmCarpet, Lang::Fr) => "Moquette confortable",
            (Self::WarmCarpet, Lang::En) => "Soft Warm Carpet",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FurnitureCategory {
    #[default]
    Living,
    Kitchen,
    Bedroom,
    Bathroom,
    Technical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FurnitureKind {
    #[default]
    Sofa,
    DiningTable,
    CoffeeTable,
    KitchenIsland,
    KitchenCounter,
    Fridge,
    DoubleBed,
    Wardrobe,
    Desk,
    Bathtub,
    WalkInShower,
    VanitySink,
    Toilet,
    ElectricalPanel,
    WaterHeater,
    WoodStove,
}

impl FurnitureKind {
    pub fn category(self) -> FurnitureCategory {
        match self {
            Self::Sofa | Self::DiningTable | Self::CoffeeTable => FurnitureCategory::Living,
            Self::KitchenIsland | Self::KitchenCounter | Self::Fridge => FurnitureCategory::Kitchen,
            Self::DoubleBed | Self::Wardrobe | Self::Desk => FurnitureCategory::Bedroom,
            Self::Bathtub | Self::WalkInShower | Self::VanitySink | Self::Toilet => {
                FurnitureCategory::Bathroom
            }
            Self::ElectricalPanel | Self::WaterHeater | Self::WoodStove => {
                FurnitureCategory::Technical
            }
        }
    }

    pub fn default_dims(self) -> (f32, f32, f32) {
        // (width, depth, height) in meters
        match self {
            Self::Sofa => (2.20, 0.95, 0.82),
            Self::DiningTable => (1.80, 0.95, 0.76),
            Self::CoffeeTable => (1.10, 0.65, 0.42),
            Self::KitchenIsland => (1.80, 0.90, 0.90),
            Self::KitchenCounter => (2.40, 0.62, 0.90),
            Self::Fridge => (0.70, 0.70, 1.90),
            Self::DoubleBed => (1.65, 2.05, 0.58),
            Self::Wardrobe => (1.80, 0.60, 2.20),
            Self::Desk => (1.40, 0.70, 0.75),
            Self::Bathtub => (1.70, 0.75, 0.58),
            Self::WalkInShower => (1.20, 0.90, 2.00),
            Self::VanitySink => (1.20, 0.52, 0.85),
            Self::Toilet => (0.42, 0.68, 0.78),
            Self::ElectricalPanel => (0.50, 0.22, 1.80),
            Self::WaterHeater => (0.58, 0.58, 1.65),
            Self::WoodStove => (0.55, 0.55, 1.15),
        }
    }

    pub fn default_cost(self) -> f32 {
        match self {
            Self::Sofa => 950.0,
            Self::DiningTable => 520.0,
            Self::CoffeeTable => 210.0,
            Self::KitchenIsland => 1450.0,
            Self::KitchenCounter => 2200.0,
            Self::Fridge => 680.0,
            Self::DoubleBed => 780.0,
            Self::Wardrobe => 640.0,
            Self::Desk => 290.0,
            Self::Bathtub => 650.0,
            Self::WalkInShower => 890.0,
            Self::VanitySink => 540.0,
            Self::Toilet => 380.0,
            Self::ElectricalPanel => 1150.0,
            Self::WaterHeater => 920.0,
            Self::WoodStove => 1850.0,
        }
    }

    pub fn color_rgb(self) -> (u8, u8, u8) {
        match self {
            Self::Sofa => (94, 129, 172),
            Self::DiningTable | Self::CoffeeTable | Self::Desk => (184, 138, 92),
            Self::KitchenIsland | Self::KitchenCounter => (76, 86, 106),
            Self::Fridge | Self::Bathtub | Self::VanitySink | Self::Toilet => (236, 239, 244),
            Self::WalkInShower => (168, 212, 232),
            Self::DoubleBed => (163, 190, 140),
            Self::Wardrobe => (190, 165, 135),
            Self::ElectricalPanel => (235, 203, 139),
            Self::WaterHeater => (216, 222, 233),
            Self::WoodStove => (59, 66, 82),
        }
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Self::Sofa, Lang::Fr) => "Grand canapé 3 places",
            (Self::Sofa, Lang::En) => "3-Seater Sofa",
            (Self::DiningTable, Lang::Fr) => "Table à manger (6 pers.)",
            (Self::DiningTable, Lang::En) => "Dining Table (6 seats)",
            (Self::CoffeeTable, Lang::Fr) => "Table basse salon",
            (Self::CoffeeTable, Lang::En) => "Coffee Table",
            (Self::KitchenIsland, Lang::Fr) => "Îlot central de cuisine",
            (Self::KitchenIsland, Lang::En) => "Central Kitchen Island",
            (Self::KitchenCounter, Lang::Fr) => "Meubles & évier de cuisine",
            (Self::KitchenCounter, Lang::En) => "Kitchen Counter & Sink",
            (Self::Fridge, Lang::Fr) => "Grand réfrigérateur",
            (Self::Fridge, Lang::En) => "Fridge-Freezer",
            (Self::DoubleBed, Lang::Fr) => "Lit double (2 places)",
            (Self::DoubleBed, Lang::En) => "Double Bed (Queen)",
            (Self::Wardrobe, Lang::Fr) => "Grande armoire / Placard",
            (Self::Wardrobe, Lang::En) => "Large Wardrobe Closet",
            (Self::Desk, Lang::Fr) => "Bureau & chaise",
            (Self::Desk, Lang::En) => "Desk & Chair",
            (Self::Bathtub, Lang::Fr) => "Baignoire confortable",
            (Self::Bathtub, Lang::En) => "Bathtub",
            (Self::WalkInShower, Lang::Fr) => "Grande douche plate",
            (Self::WalkInShower, Lang::En) => "Walk-in Shower",
            (Self::VanitySink, Lang::Fr) => "Meuble lavabo & miroir",
            (Self::VanitySink, Lang::En) => "Bathroom Sink & Mirror",
            (Self::Toilet, Lang::Fr) => "Toilettes (WC)",
            (Self::Toilet, Lang::En) => "Toilet (WC)",
            (Self::ElectricalPanel, Lang::Fr) => "Tableau électrique",
            (Self::ElectricalPanel, Lang::En) => "Electrical Box",
            (Self::WaterHeater, Lang::Fr) => "Chauffe-eau (Ballon)",
            (Self::WaterHeater, Lang::En) => "Hot Water Heater",
            (Self::WoodStove, Lang::Fr) => "Poêle à bois / cheminée",
            (Self::WoodStove, Lang::En) => "Wood Stove / Heater",
        }
    }

    pub const ALL: [FurnitureKind; 16] = [
        Self::Sofa,
        Self::DiningTable,
        Self::CoffeeTable,
        Self::KitchenIsland,
        Self::KitchenCounter,
        Self::Fridge,
        Self::DoubleBed,
        Self::Wardrobe,
        Self::Desk,
        Self::Bathtub,
        Self::WalkInShower,
        Self::VanitySink,
        Self::Toilet,
        Self::ElectricalPanel,
        Self::WaterHeater,
        Self::WoodStove,
    ];
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wall {
    pub id: u32,
    pub level: usize,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub thickness: f32,
    pub height: f32,
    pub load_bearing: bool,
    pub status: RenoStatus,
    pub name: String,
}

impl Wall {
    pub fn length(&self) -> f32 {
        ((self.x2 - self.x1).powi(2) + (self.y2 - self.y1).powi(2)).sqrt()
    }

    pub fn angle_deg(&self) -> f32 {
        (self.y2 - self.y1).atan2(self.x2 - self.x1).to_degrees()
    }

    pub fn midpoint(&self) -> (f32, f32) {
        ((self.x1 + self.x2) * 0.5, (self.y1 + self.y2) * 0.5)
    }

    pub fn point_at(&self, t: f32) -> (f32, f32) {
        let t = t.clamp(0.0, 1.0);
        (
            self.x1 + (self.x2 - self.x1) * t,
            self.y1 + (self.y2 - self.y1) * t,
        )
    }

    pub fn direction(&self) -> (f32, f32) {
        let len = self.length().max(1e-5);
        ((self.x2 - self.x1) / len, (self.y2 - self.y1) / len)
    }

    pub fn normal(&self) -> (f32, f32) {
        let (dx, dy) = self.direction();
        (-dy, dx)
    }

    pub fn project_point(&self, px: f32, py: f32) -> (f32, f32) {
        // Returns (t along segment in [0,1], distance in meters)
        let dx = self.x2 - self.x1;
        let dy = self.y2 - self.y1;
        let len_sq = dx * dx + dy * dy;
        if len_sq < 1e-6 {
            let d = ((px - self.x1).powi(2) + (py - self.y1).powi(2)).sqrt();
            return (0.0, d);
        }
        let t = (((px - self.x1) * dx + (py - self.y1) * dy) / len_sq).clamp(0.0, 1.0);
        let proj_x = self.x1 + t * dx;
        let proj_y = self.y1 + t * dy;
        let dist = ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt();
        (t, dist)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Opening {
    pub id: u32,
    pub level: usize,
    pub wall_id: u32,
    pub offset_t: f32, // 0.0 to 1.0 along wall
    pub width: f32,
    pub height: f32,
    pub sill_height: f32,
    pub kind: OpeningKind,
    pub status: RenoStatus,
    pub flip_swing: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomZone {
    pub id: u32,
    pub level: usize,
    pub name_fr: String,
    pub name_en: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub depth: f32,
    pub ceiling_height: f32,
    pub material: FloorMaterial,
    pub status: RenoStatus,
}

impl RoomZone {
    pub fn area_m2(&self) -> f32 {
        (self.width.abs() * self.depth.abs()).max(0.0)
    }

    pub fn carrez_m2(&self) -> f32 {
        if self.ceiling_height >= 1.80 {
            self.area_m2()
        } else {
            0.0
        }
    }

    pub fn perimeter_m(&self) -> f32 {
        2.0 * (self.width.abs() + self.depth.abs())
    }

    pub fn name(&self, lang: Lang) -> &str {
        match lang {
            Lang::Fr => &self.name_fr,
            Lang::En => &self.name_en,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FurnitureItem {
    pub id: u32,
    pub level: usize,
    pub kind: FurnitureKind,
    pub label_custom: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub rotation_deg: f32,
    pub status: RenoStatus,
    pub cost: f32,
}

impl FurnitureItem {
    pub fn display_name(&self, lang: Lang) -> String {
        if !self.label_custom.trim().is_empty() {
            self.label_custom.clone()
        } else {
            self.kind.label(lang).to_owned()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectedEntity {
    Wall(u32),
    Opening(u32),
    Room(u32),
    Furniture(u32),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EstimateLine {
    pub category_fr: String,
    pub category_en: String,
    pub label_fr: String,
    pub label_en: String,
    pub quantity: String,
    pub unit_price: String,
    pub total_cost: f32,
    pub status: RenoStatus,
}

#[derive(Clone, Debug, Default)]
pub struct EstimateSummary {
    pub floor_area_m2: f32,
    pub carrez_area_m2: f32,
    pub demolish_wall_linear_m: f32,
    pub demolish_volume_m3: f32,
    pub new_wall_linear_m: f32,
    pub paint_wall_area_m2: f32,
    pub openings_count: usize,
    pub furniture_count: usize,
    pub demolish_cost: f32,
    pub build_wall_cost: f32,
    pub flooring_cost: f32,
    pub openings_cost: f32,
    pub furniture_cost: f32,
    pub total_cost: f32,
    pub lines: Vec<EstimateLine>,
}
