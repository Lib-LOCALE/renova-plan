pub mod canvas;
pub mod model;
pub mod project;
pub mod render2d;
pub mod render3d;

pub use canvas::{CanvasBuffer, Color};
pub use model::{
    EstimateLine, EstimateSummary, FloorMaterial, FurnitureCategory, FurnitureItem, FurnitureKind,
    Lang, Opening, OpeningKind, RenoStatus, RoomZone, SelectedEntity, ViewPhase, Wall,
};
pub use project::Project;
pub use render2d::{Camera2D, Render2DOptions, render_plan_2d};
pub use render3d::{Camera3D, render_scene_3d};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_maison_computes_accurate_carrez_and_renovation_estimate() {
        let p = Project::preset_maison_renovation();
        let est = p.compute_estimate();
        // Level 0 + Level 1 non-demolished floor area:
        // Level 0: 6.5*5 + 4.5*5 + 5*3 + 3.2*3 + 2.8*3 = 32.5 + 22.5 + 15.0 + 9.6 + 8.4 = 88.0 m²
        // Level 1: 5.5*6 + 5.5*6 = 66.0 m² -> total 154.0 m²
        assert!((est.floor_area_m2 - 154.0).abs() < 0.05);
        assert_eq!(est.floor_area_m2, est.carrez_area_m2);
        assert!(est.demolish_wall_linear_m > 7.0);
        assert!(est.new_wall_linear_m > 12.0);
        assert!(est.total_cost > 15_000.0);
        assert!(!est.lines.is_empty());
    }

    #[test]
    fn phase_visibility_filters_before_and_after_correctly() {
        assert!(RenoStatus::Existing.is_visible_in(ViewPhase::Chantier));
        assert!(RenoStatus::Existing.is_visible_in(ViewPhase::Before));
        assert!(RenoStatus::Existing.is_visible_in(ViewPhase::After));

        assert!(RenoStatus::Demolish.is_visible_in(ViewPhase::Chantier));
        assert!(RenoStatus::Demolish.is_visible_in(ViewPhase::Before));
        assert!(!RenoStatus::Demolish.is_visible_in(ViewPhase::After));

        assert!(RenoStatus::New.is_visible_in(ViewPhase::Chantier));
        assert!(!RenoStatus::New.is_visible_in(ViewPhase::Before));
        assert!(RenoStatus::New.is_visible_in(ViewPhase::After));
    }

    #[test]
    fn wall_projection_and_hit_testing_work() {
        let mut p = Project::preset_blank_shell();
        let w_id = p.add_wall(
            0,
            2.0,
            2.0,
            6.0,
            2.0,
            0.15,
            2.50,
            false,
            RenoStatus::New,
            "Cloison test",
        );
        let hit = p.hit_test(4.0, 2.02, ViewPhase::Chantier);
        assert_eq!(hit, Some(SelectedEntity::Wall(w_id)));

        // In Before phase, a New wall is hidden and not hit
        let hit_before = p.hit_test(4.0, 2.02, ViewPhase::Before);
        assert_ne!(hit_before, Some(SelectedEntity::Wall(w_id)));

        assert!(p.delete_entity(SelectedEntity::Wall(w_id)));
    }

    #[test]
    fn render_2d_and_3d_produce_non_empty_frames() {
        let p = Project::preset_maison_renovation();
        let cam2d = Camera2D::default();
        let opts = Render2DOptions {
            dark_mode: true,
            phase: ViewPhase::Chantier,
            lang: Lang::Fr,
            selected: None,
            draft_line: Some((1.0, 1.0, 4.0, 1.0, RenoStatus::New)),
            measure_line: Some((1.0, 2.0, 3.5, 2.0)),
            hover_world: Some((4.0, 1.0)),
        };
        let buf2d = render_plan_2d(&p, &cam2d, 640, 480, &opts);
        assert_eq!(buf2d.pixels.len(), 640 * 480 * 4);

        let cam3d = Camera3D::default();
        let buf3d = render_scene_3d(
            &p,
            &cam3d,
            640,
            480,
            true,
            ViewPhase::Chantier,
            Lang::Fr,
            None,
        );
        assert_eq!(buf3d.pixels.len(), 640 * 480 * 4);
    }

    #[test]
    fn export_svg_and_csv_are_valid_in_french_and_english() {
        let p = Project::preset_maison_renovation();
        let svg_fr = p.export_svg(ViewPhase::Chantier, Lang::Fr);
        assert!(svg_fr.contains("<svg"));
        assert!(svg_fr.contains("Séjour & Salon"));

        let csv_en = p.export_csv(Lang::En);
        assert!(csv_en.contains("Category;Item;Status;Quantity;Unit Price;Total EUR"));
        assert!(csv_en.contains("New Partition"));
    }

    #[test]
    fn add_boxed_room_creates_floor_zone_and_four_walls() {
        let mut p = Project::preset_blank_shell();
        let initial_walls = p.walls.len();
        let initial_rooms = p.rooms.len();
        let room_id = p.add_boxed_room(
            0,
            "Chambre Amis",
            "Guest Bedroom",
            1.0,
            1.0,
            3.5,
            3.0,
            FloorMaterial::OakParquet,
            RenoStatus::New,
        );
        assert_eq!(p.rooms.len(), initial_rooms + 1);
        assert_eq!(p.walls.len(), initial_walls + 4);
        assert_eq!(
            p.hit_test(2.5, 2.5, ViewPhase::Chantier),
            Some(SelectedEntity::Room(room_id))
        );
    }
}
