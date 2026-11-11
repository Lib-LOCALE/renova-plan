#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod native;
#[cfg(debug_assertions)]
mod smoke;

use moteur::{
    Camera2D, Camera3D, FloorMaterial, FurnitureCategory, FurnitureKind, Lang, OpeningKind,
    Project, Render2DOptions, RenoStatus, SelectedEntity, ViewPhase, render_plan_2d,
    render_scene_3d,
};
use native::{I18nData, Preferences};
use slint::winit_030::{EventResult, WinitWindowAccessor, winit};
use slint::{ComponentHandle, Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel};
use std::{cell::RefCell, rc::Rc};
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{Key as NativeKey, ModifiersState, NamedKey},
};

slint::include_modules!();

#[derive(Clone, Copy)]
enum Drag2DState {
    None,
    Panning {
        start_sx: f32,
        start_sy: f32,
        orig_pan_x: f32,
        orig_pan_y: f32,
    },
    MovingEntity {
        entity: SelectedEntity,
        last_wx: f32,
        last_wy: f32,
    },
    ResizingWallEndpoint {
        wall_id: u32,
        is_end2: bool,
    },
    ResizingRoom {
        room_id: u32,
        anchor_x: Option<f32>,
        anchor_y: Option<f32>,
    },
    DrawingWall {
        start_wx: f32,
        start_wy: f32,
        cur_wx: f32,
        cur_wy: f32,
        load_bearing: bool,
    },
    DrawingRoom {
        start_wx: f32,
        start_wy: f32,
        cur_wx: f32,
        cur_wy: f32,
        boxed: bool,
    },
    Measuring {
        start_wx: f32,
        start_wy: f32,
        cur_wx: f32,
        cur_wy: f32,
    },
}

struct AppState {
    project: Project,
    undo_stack: Vec<Project>,
    redo_stack: Vec<Project>,
    cam2d: Camera2D,
    cam3d: Camera3D,
    selected: Option<SelectedEntity>,
    drag2d: Drag2DState,
    drag3d_last: Option<(f32, f32, i32)>,
    hover_world: Option<(f32, f32)>,
    measure_line: Option<(f32, f32, f32, f32)>,
    i18n: I18nData,
    prefs: Preferences,
    is_dirty: bool,
    current_file_path: Option<std::path::PathBuf>,
}

impl AppState {
    fn lang(&self, ui: &RenovaWindow) -> Lang {
        if ui.get_language_index() == 1 {
            Lang::En
        } else {
            Lang::Fr
        }
    }

    fn phase(&self, ui: &RenovaWindow) -> ViewPhase {
        match ui.get_phase_index() {
            1 => ViewPhase::Before,
            2 => ViewPhase::After,
            _ => ViewPhase::Chantier,
        }
    }

    fn active_reno_status(&self, ui: &RenovaWindow) -> RenoStatus {
        match ui.get_active_status() {
            0 => RenoStatus::Existing,
            1 => RenoStatus::Demolish,
            _ => RenoStatus::New,
        }
    }

    fn push_undo(&mut self) {
        if self.undo_stack.len() >= 30 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(self.project.clone());
        self.redo_stack.clear();
        self.is_dirty = true;
    }

    fn apply_theme(&mut self, ui: &RenovaWindow) {
        let idx = ui.get_theme_index().max(0) as usize;
        self.prefs.theme = idx % 4;
        self.prefs.language = ui.get_language_index().max(0) as usize % 2;
        self.prefs.large_text = ui.get_large_text();
        self.prefs.expert_mode = ui.get_expert_mode();
        let _ = self.prefs.save();

        let pal = native::theme_palette(self.prefs.theme);
        let theme = ui.global::<Theme>();
        theme.set_bg(native::hex_color(pal.bg));
        theme.set_panel(native::hex_color(pal.panel));
        theme.set_surface(native::hex_color(pal.surface));
        theme.set_hover(native::hex_color(pal.hover));
        theme.set_border(native::hex_color(pal.border));
        theme.set_text(native::hex_color(pal.text));
        theme.set_muted(native::hex_color(pal.muted));
        theme.set_accent(native::hex_color(pal.accent));
        theme.set_on_accent(native::hex_color(pal.on_accent));
        theme.set_green(native::hex_color(pal.green));
        theme.set_amber(native::hex_color(pal.amber));
        theme.set_red(native::hex_color(pal.red));
    }

    fn refresh_catalog(&self, ui: &RenovaWindow) {
        let lang = self.lang(ui);
        let cat = match ui.get_catalog_cat() {
            1 => FurnitureCategory::Kitchen,
            2 => FurnitureCategory::Bedroom,
            3 => FurnitureCategory::Bathroom,
            4 => FurnitureCategory::Technical,
            _ => FurnitureCategory::Living,
        };
        let items: Vec<CatalogItem> = FurnitureKind::ALL
            .iter()
            .copied()
            .filter(|k| k.category() == cat)
            .map(|k| {
                let (w, d, _) = k.default_dims();
                CatalogItem {
                    id: format!("{k:?}").into(),
                    label: k.label(lang).into(),
                    dims: format!("{:.0}×{:.0} cm", w * 100.0, d * 100.0).into(),
                    cost: format!("{:.0} €", k.default_cost()).into(),
                }
            })
            .collect();
        ui.set_catalog_items(ModelRc::new(VecModel::from(items)));
    }

    fn refresh_inspector_and_estimate(&self, ui: &RenovaWindow) {
        let lang = self.lang(ui);
        let est = self.project.compute_estimate();

        ui.set_kpi_carrez(format!("{:.1} m²", est.carrez_area_m2).into());
        ui.set_kpi_floor(format!("{:.1} m²", est.floor_area_m2).into());
        ui.set_kpi_demolish(
            format!(
                "{:.2} ml ({:.1} m³)",
                est.demolish_wall_linear_m, est.demolish_volume_m3
            )
            .into(),
        );
        ui.set_kpi_new_walls(format!("{:.2} ml", est.new_wall_linear_m).into());
        ui.set_kpi_paint(format!("{:.1} m²", est.paint_wall_area_m2).into());
        ui.set_kpi_total_cost(format!("{:.0} €", est.total_cost).into());

        let rows: Vec<EstimateRow> = est
            .lines
            .iter()
            .map(|line| EstimateRow {
                category: (if lang == Lang::En {
                    &line.category_en
                } else {
                    &line.category_fr
                })
                .clone()
                .into(),
                label: (if lang == Lang::En {
                    &line.label_en
                } else {
                    &line.label_fr
                })
                .clone()
                .into(),
                status_tag: line.status.label(lang).into(),
                status_kind: match line.status {
                    RenoStatus::Existing => 0,
                    RenoStatus::Demolish => 1,
                    RenoStatus::New => 2,
                },
                quantity: line.quantity.clone().into(),
                total: format!("{:.0} €", line.total_cost).into(),
            })
            .collect();
        ui.set_estimate_rows(ModelRc::new(VecModel::from(rows)));

        // Inspector
        let Some(sel) = self.selected else {
            ui.set_has_selection(false);
            return;
        };

        match sel {
            SelectedEntity::Wall(id) => {
                if let Some(w) = self.project.walls.iter().find(|w| w.id == id) {
                    ui.set_has_selection(true);
                    let display_name = if lang == Lang::En {
                        w.name
                            .replace("Mur Extérieur", "Exterior Wall")
                            .replace("Mur Porteur", "Structural Wall")
                            .replace("Cloison", "Partition")
                            .replace("Ancienne", "Old")
                            .replace("Nouvelle", "New")
                    } else {
                        w.name.clone()
                    };
                    ui.set_sel_name(display_name.into());
                    ui.set_sel_kind(
                        (if w.load_bearing {
                            if lang == Lang::En {
                                "Thick Structural Wall (25 cm)"
                            } else {
                                "Gros mur porteur (25 cm)"
                            }
                        } else if lang == Lang::En {
                            "Indoor Partition Wall (10 cm)"
                        } else {
                            "Cloison intérieure (10 cm)"
                        })
                        .into(),
                    );
                    ui.set_sel_status(status_to_int(w.status));
                    ui.set_sel_length(format!("{:.2} m", w.length()).into());
                    ui.set_sel_depth(format!("{:.0} cm", w.thickness * 100.0).into());
                    ui.set_sel_height(format!("{:.2} m", w.height).into());
                    ui.set_sel_rotation(format!("{:.0}°", w.angle_deg()).into());
                    let rate = if w.status == RenoStatus::Demolish {
                        52.0
                    } else {
                        68.0
                    };
                    ui.set_sel_cost(format!("{:.0} €", w.length() * w.height * rate).into());
                } else {
                    ui.set_has_selection(false);
                }
            }
            SelectedEntity::Opening(id) => {
                if let Some(op) = self.project.openings.iter().find(|o| o.id == id) {
                    ui.set_has_selection(true);
                    ui.set_sel_name(op.kind.label(lang).into());
                    ui.set_sel_kind(
                        (if op.kind.is_door() {
                            if lang == Lang::En { "Door" } else { "Porte" }
                        } else if lang == Lang::En {
                            "Window"
                        } else {
                            "Fenêtre"
                        })
                        .into(),
                    );
                    ui.set_sel_status(status_to_int(op.status));
                    ui.set_sel_length(format!("{:.0} cm", op.width * 100.0).into());
                    ui.set_sel_depth(
                        (if lang == Lang::En {
                            format!("Height from floor: {:.0} cm", op.sill_height * 100.0)
                        } else {
                            format!("Hauteur du bas : {:.0} cm", op.sill_height * 100.0)
                        })
                        .into(),
                    );
                    ui.set_sel_height(format!("{:.0} cm", op.height * 100.0).into());
                    ui.set_sel_rotation(
                        (if op.flip_swing {
                            if lang == Lang::En {
                                "Flipped side"
                            } else {
                                "Sens inversé"
                            }
                        } else if lang == Lang::En {
                            "Normal side"
                        } else {
                            "Sens normal"
                        })
                        .into(),
                    );
                    ui.set_sel_cost(format!("{:.0} €", op.kind.default_cost()).into());
                } else {
                    ui.set_has_selection(false);
                }
            }
            SelectedEntity::Room(id) => {
                if let Some(r) = self.project.rooms.iter().find(|r| r.id == id) {
                    ui.set_has_selection(true);
                    ui.set_sel_name(r.name(lang).into());
                    ui.set_sel_kind(r.material.label(lang).into());
                    ui.set_sel_status(status_to_int(r.status));
                    ui.set_sel_length(format!("{:.2} m ({:.1} m²)", r.width, r.area_m2()).into());
                    ui.set_sel_depth(format!("{:.2} m", r.depth).into());
                    ui.set_sel_height(format!("{:.2} m", r.ceiling_height).into());
                    ui.set_sel_rotation(
                        (if lang == Lang::En {
                            format!("Living area: {:.1} m²", r.carrez_m2())
                        } else {
                            format!("Surface : {:.1} m²", r.carrez_m2())
                        })
                        .into(),
                    );
                    ui.set_sel_cost(
                        format!("{:.0} €", r.area_m2() * r.material.cost_per_m2()).into(),
                    );
                } else {
                    ui.set_has_selection(false);
                }
            }
            SelectedEntity::Furniture(id) => {
                if let Some(f) = self.project.furniture.iter().find(|f| f.id == id) {
                    ui.set_has_selection(true);
                    ui.set_sel_name(f.display_name(lang).into());
                    ui.set_sel_kind(
                        (if lang == Lang::En {
                            "Furniture / Fixture"
                        } else {
                            "Mobilier & Équipement"
                        })
                        .into(),
                    );
                    ui.set_sel_status(status_to_int(f.status));
                    ui.set_sel_length(format!("{:.0} cm", f.width * 100.0).into());
                    ui.set_sel_depth(format!("{:.0} cm", f.depth * 100.0).into());
                    ui.set_sel_height(format!("{:.0} cm", f.height * 100.0).into());
                    ui.set_sel_rotation(format!("{:.0}°", f.rotation_deg).into());
                    ui.set_sel_cost(format!("{:.0} €", f.cost).into());
                } else {
                    ui.set_has_selection(false);
                }
            }
        }
    }

    fn refresh_all(&mut self, ui: &RenovaWindow) {
        let lang = self.lang(ui);
        let phase = self.phase(ui);
        let pal = native::theme_palette(ui.get_theme_index().max(0) as usize);

        let title = if self.is_dirty {
            format!("{} *", self.project.title(lang))
        } else {
            self.project.title(lang).to_string()
        };
        ui.set_project_title(title.into());
        ui.set_has_unsaved_changes(self.is_dirty);
        ui.set_active_level(self.project.active_level as i32);
        ui.set_show_grid(self.project.show_grid);
        ui.set_show_dims(self.project.show_dimensions);
        ui.set_show_underlay(self.project.show_underlay);

        // Contextual hint in bottom bar
        let tool = ui.get_active_tool();
        let hint_key = match tool.as_str() {
            "wall" | "partition" | "room" | "boxed_room" => "hint_wall",
            "door" | "double_door" | "sliding_door" | "passage" | "window" | "small_window"
            | "bay_window" => "hint_opening",
            "measure" => "hint_measure",
            _ => "hint_select",
        };
        ui.set_status_hint(self.i18n.get(hint_key, ui.get_language_index()).into());

        self.refresh_catalog(ui);
        self.refresh_inspector_and_estimate(ui);

        let view_mode = ui.get_view_mode();
        let draft_line = match self.drag2d {
            Drag2DState::DrawingWall {
                start_wx,
                start_wy,
                cur_wx,
                cur_wy,
                ..
            }
            | Drag2DState::DrawingRoom {
                start_wx,
                start_wy,
                cur_wx,
                cur_wy,
                ..
            } => Some((
                start_wx,
                start_wy,
                cur_wx,
                cur_wy,
                self.active_reno_status(ui),
            )),
            _ => None,
        };
        let measure_line = match self.drag2d {
            Drag2DState::Measuring {
                start_wx,
                start_wy,
                cur_wx,
                cur_wy,
            } => Some((start_wx, start_wy, cur_wx, cur_wy)),
            _ => self.measure_line,
        };

        let render_w = if view_mode == 1 { 560 } else { 1020 };
        let render_h = 700;

        if view_mode == 0 || view_mode == 1 {
            let opts = Render2DOptions {
                dark_mode: pal.dark_canvas,
                phase,
                lang,
                selected: self.selected,
                draft_line,
                measure_line,
                hover_world: self.hover_world,
            };
            let buf2d = render_plan_2d(&self.project, &self.cam2d, render_w, render_h, &opts);
            let slint_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                &buf2d.pixels,
                buf2d.width,
                buf2d.height,
            );
            ui.set_canvas_2d(Image::from_rgba8(slint_buf));
        }

        if view_mode == 1 || view_mode == 2 {
            let buf3d = render_scene_3d(
                &self.project,
                &self.cam3d,
                render_w,
                render_h,
                pal.dark_canvas,
                phase,
                lang,
                self.selected,
            );
            let slint_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                &buf3d.pixels,
                buf3d.width,
                buf3d.height,
            );
            ui.set_canvas_3d(Image::from_rgba8(slint_buf));
        }
    }

    fn auto_frame_floor(&mut self) {
        if let Some((min_x, min_y, max_x, max_y)) =
            self.project.floor_bounding_box(self.project.active_level)
        {
            let cx = (min_x + max_x) * 0.5;
            let cy = (min_y + max_y) * 0.5;
            self.cam2d.pan_x = cx;
            self.cam2d.pan_y = cy;
            let span_w = (max_x - min_x).max(6.0);
            let span_h = (max_y - min_y).max(4.5);
            let fit_zoom = (720.0 / (span_w * 1.30)).min(520.0 / (span_h * 1.30));
            self.cam2d.zoom = fit_zoom.clamp(28.0, 75.0);
        }
    }

    fn quick_add_room(
        &mut self,
        ui: &RenovaWindow,
        name_fr: &str,
        name_en: &str,
        w: f32,
        d: f32,
        mat: FloorMaterial,
    ) {
        self.push_undo();
        // Si l'utilisateur est en phase "Avant", la pièce créée est "Existante" pour rester toujours visible
        let status = if self.phase(ui) == ViewPhase::Before {
            RenoStatus::Existing
        } else {
            self.active_reno_status(ui)
        };
        let id = self.project.add_ready_room(
            self.project.active_level,
            name_fr,
            name_en,
            self.cam2d.pan_x,
            self.cam2d.pan_y,
            w,
            d,
            mat,
            status,
        );
        self.selected = Some(SelectedEntity::Room(id));
        self.auto_frame_floor();
        ui.set_active_tool("select".into());
        let label = if self.lang(ui) == Lang::En { name_en } else { name_fr };
        let msg = if self.lang(ui) == Lang::En {
            format!("✓ {label} ({w:.1} × {d:.1} m) added to plan!")
        } else {
            format!("✓ {label} ({w:.1} × {d:.1} m) ajouté sur le plan !")
        };
        show_toast(ui, msg);
    }

    fn handle_action(&mut self, ui: &RenovaWindow, cmd: &str) {
        match cmd {
            "refresh" => {}
            "lang:toggle" => {
                let cur = ui.get_language_index();
                self.prefs.language = (cur.max(0) as usize) % 2;
                let _ = self.prefs.save();
                self.refresh_catalog(ui);
            }
            "theme:cycle" | "theme:apply" | "prefs:save" => {
                self.apply_theme(ui);
            }
            "preset:house" => {
                self.push_undo();
                self.project = Project::preset_maison_renovation();
                self.selected = None;
                self.cam2d = Camera2D::default();
                self.is_dirty = false;
                self.current_file_path = None;
            }
            "preset:studio" => {
                self.push_undo();
                self.project = Project::preset_studio_appartement();
                self.selected = None;
                self.cam2d = Camera2D {
                    pan_x: 4.0,
                    pan_y: 3.0,
                    zoom: 68.0,
                };
                self.is_dirty = false;
                self.current_file_path = None;
            }
            "preset:blank" => {
                self.push_undo();
                self.project = Project::preset_creation_libre();
                self.selected = None;
                self.cam2d = Camera2D::default();
                self.is_dirty = false;
                self.current_file_path = None;
            }
            "preset:shell" => {
                self.push_undo();
                self.project = Project::preset_blank_shell();
                self.selected = None;
                self.cam2d = Camera2D::default();
                self.is_dirty = false;
                self.current_file_path = None;
            }
            "undo" => {
                if let Some(prev) = self.undo_stack.pop() {
                    self.redo_stack.push(self.project.clone());
                    self.project = prev;
                    self.selected = None;
                }
            }
            "redo" => {
                if let Some(next) = self.redo_stack.pop() {
                    self.undo_stack.push(self.project.clone());
                    self.project = next;
                    self.selected = None;
                }
            }
            "toggle:grid" => {
                self.project.show_grid = ui.get_show_grid();
                self.project.snap_to_grid = ui.get_show_grid();
            }
            "toggle:dims" => {
                self.project.show_dimensions = ui.get_show_dims();
            }
            "toggle:underlay" => {
                self.project.show_underlay = ui.get_show_underlay();
            }
            "level:0" => {
                self.project.active_level = 0;
                self.selected = None;
            }
            "level:1" => {
                self.project.active_level = 1;
                self.selected = None;
            }
            "level:toggle" => {
                self.project.active_level = ui.get_active_level().max(0) as usize % 2;
                self.selected = None;
            }
            "2d:zoom_in" => {
                self.cam2d.zoom = (self.cam2d.zoom * 1.2).clamp(24.0, 180.0);
            }
            "2d:zoom_out" => {
                self.cam2d.zoom = (self.cam2d.zoom / 1.2).clamp(24.0, 180.0);
            }
            "2d:reset" => {
                self.cam2d = Camera2D::default();
            }
            "3d:iso" => self.cam3d.preset_iso(),
            "3d:top" => self.cam3d.preset_top(),
            "3d:south" => self.cam3d.preset_south(),
            "3d:interior" => self.cam3d.preset_interior(),
            "3d:cut_cycle" => {
                self.cam3d.wall_cut_ratio = if self.cam3d.wall_cut_ratio > 0.90 {
                    0.38
                } else if self.cam3d.wall_cut_ratio < 0.50 {
                    0.82
                } else {
                    1.0
                };
            }
            "3d:cut:35" => self.cam3d.wall_cut_ratio = 0.35,
            "3d:cut:80" => self.cam3d.wall_cut_ratio = 0.80,
            "3d:cut:100" => self.cam3d.wall_cut_ratio = 1.00,
            "sel:status:0" | "sel:status:1" | "sel:status:2" => {
                let st = match cmd {
                    "sel:status:0" => RenoStatus::Existing,
                    "sel:status:1" => RenoStatus::Demolish,
                    _ => RenoStatus::New,
                };
                self.push_undo();
                self.set_selected_status(st);
            }
            "sel:len:+" => {
                self.push_undo();
                self.adjust_selected_length(0.10);
            }
            "sel:len:-" => {
                self.push_undo();
                self.adjust_selected_length(-0.10);
            }
            "sel:thick:+" => {
                self.push_undo();
                self.adjust_selected_thickness(0.05);
            }
            "sel:thick:-" => {
                self.push_undo();
                self.adjust_selected_thickness(-0.05);
            }
            "sel:rotate" => {
                self.push_undo();
                self.rotate_selected();
            }
            "sel:duplicate" => {
                self.push_undo();
                self.duplicate_selected();
            }
            "sel:delete" => {
                if let Some(sel) = self.selected {
                    self.push_undo();
                    match sel {
                        SelectedEntity::Room(id) => self.project.delete_room_with_walls(id),
                        other => {
                            let _ = self.project.delete_entity(other);
                        }
                    }
                    self.selected = None;
                }
            }
            "quick:bedroom" => {
                self.quick_add_room(
                    ui,
                    "Chambre",
                    "Bedroom",
                    3.5,
                    3.0,
                    FloorMaterial::OakParquet,
                );
            }
            "quick:living" => {
                self.quick_add_room(
                    ui,
                    "Salon / Séjour",
                    "Living Room",
                    5.0,
                    4.0,
                    FloorMaterial::OakParquet,
                );
            }
            "quick:kitchen" => {
                self.quick_add_room(ui, "Cuisine", "Kitchen", 3.5, 3.0, FloorMaterial::LightTile);
            }
            "quick:bathroom" => {
                self.quick_add_room(
                    ui,
                    "Salle de bain",
                    "Bathroom",
                    2.5,
                    2.0,
                    FloorMaterial::SlateTile,
                );
            }
            "quick:hallway" => {
                self.quick_add_room(
                    ui,
                    "Entrée / Couloir",
                    "Hallway",
                    3.0,
                    1.5,
                    FloorMaterial::LightTile,
                );
            }
            "quick:office" => {
                self.quick_add_room(ui, "Bureau", "Office", 3.0, 2.5, FloorMaterial::OakParquet);
            }
            "quick:wc" => {
                self.quick_add_room(ui, "WC", "WC", 1.5, 1.25, FloorMaterial::LightTile);
            }
            "quick:garage" => {
                self.quick_add_room(
                    ui,
                    "Garage / Cellier",
                    "Garage",
                    5.5,
                    3.0,
                    FloorMaterial::PolishedConcrete,
                );
            }
            "quick:door" | "quick:double_door" | "quick:sliding_door" | "quick:passage"
            | "quick:window" | "quick:small_window" | "quick:bay_window" => {
                let (kind, label) = match cmd {
                    "quick:door" => (OpeningKind::SingleDoor, "Porte"),
                    "quick:double_door" => (OpeningKind::DoubleDoor, "Porte double"),
                    "quick:sliding_door" => (OpeningKind::SlidingDoor, "Porte coulissante"),
                    "quick:passage" => (OpeningKind::OpenPassage, "Passage"),
                    "quick:small_window" => (OpeningKind::SingleWindow, "Fenêtre 1 vantail"),
                    "quick:bay_window" => (OpeningKind::BayWindow, "Baie vitrée"),
                    _ => (OpeningKind::DoubleWindow, "Fenêtre double"),
                };
                let target_wall = if let Some(SelectedEntity::Wall(w_id)) = self.selected {
                    Some((w_id, 0.5))
                } else {
                    self.project
                        .closest_wall(self.cam2d.pan_x, self.cam2d.pan_y, self.phase(ui))
                };
                let status = if self.phase(ui) == ViewPhase::Before {
                    RenoStatus::Existing
                } else {
                    self.active_reno_status(ui)
                };
                if let Some((wall_id, t)) = target_wall {
                    self.push_undo();
                    let op_id = self.project.add_opening(
                        self.project.active_level,
                        wall_id,
                        t,
                        kind,
                        status,
                        false,
                    );
                    self.selected = Some(SelectedEntity::Opening(op_id));
                    ui.set_active_tool("select".into());
                    let msg = if self.lang(ui) == Lang::En {
                        format!("✓ {label} placed on wall!")
                    } else {
                        format!("✓ {label} posée sur la cloison !")
                    };
                    show_toast(ui, msg);
                } else {
                    let tool_name = cmd.trim_start_matches("quick:");
                    ui.set_active_tool(tool_name.into());
                    let msg = if self.lang(ui) == Lang::En {
                        "Click on a wall on the plan to place the opening"
                    } else {
                        "Cliquez sur une cloison sur le plan pour poser l'ouverture"
                    };
                    show_toast(ui, msg.to_owned());
                }
            }
            "project:save" => {
                let target_path = if let Some(existing) = &self.current_file_path {
                    Some(existing.clone())
                } else if is_headless() {
                    Some(export_file_path("mon-plan-renova.json"))
                } else {
                    let title = if self.lang(ui) == Lang::En {
                        "Save Renova Plan"
                    } else {
                        "Sauvegarder le plan Renova"
                    };
                    let filter_label = if self.lang(ui) == Lang::En {
                        "Renova Plan (*.json)"
                    } else {
                        "Plan Renova (*.json)"
                    };
                    let default_dir = export_dir();
                    rfd::FileDialog::new()
                        .set_title(title)
                        .set_directory(&default_dir)
                        .set_file_name("mon-plan-renova.json")
                        .add_filter(filter_label, &["json"])
                        .save_file()
                };

                if let Some(path) = target_path {
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    match serde_json::to_vec_pretty(&self.project)
                        .map_err(|e| e.to_string())
                        .and_then(|bytes| std::fs::write(&path, bytes).map_err(|e| e.to_string()))
                    {
                        Ok(()) => {
                            self.current_file_path = Some(path.clone());
                            self.is_dirty = false;
                            let msg = if self.lang(ui) == Lang::En {
                                format!("Project saved: {}", path.display())
                            } else {
                                format!("Plan sauvegardé : {}", path.display())
                            };
                            show_toast(ui, msg);
                        }
                        Err(err) => {
                            let msg = if self.lang(ui) == Lang::En {
                                format!("Error saving project: {err}")
                            } else {
                                format!("Erreur lors de la sauvegarde : {err}")
                            };
                            show_toast(ui, msg);
                        }
                    }
                }
            }
            "project:save_as" => {
                let target_path = if is_headless() {
                    Some(export_file_path("mon-plan-renova.json"))
                } else {
                    let title = if self.lang(ui) == Lang::En {
                        "Save Renova Plan As..."
                    } else {
                        "Enregistrer le plan sous..."
                    };
                    let filter_label = if self.lang(ui) == Lang::En {
                        "Renova Plan (*.json)"
                    } else {
                        "Plan Renova (*.json)"
                    };
                    let default_dir = export_dir();
                    rfd::FileDialog::new()
                        .set_title(title)
                        .set_directory(&default_dir)
                        .set_file_name("mon-plan-renova.json")
                        .add_filter(filter_label, &["json"])
                        .save_file()
                };

                if let Some(path) = target_path {
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    match serde_json::to_vec_pretty(&self.project)
                        .map_err(|e| e.to_string())
                        .and_then(|bytes| std::fs::write(&path, bytes).map_err(|e| e.to_string()))
                    {
                        Ok(()) => {
                            self.current_file_path = Some(path.clone());
                            self.is_dirty = false;
                            let msg = if self.lang(ui) == Lang::En {
                                format!("Project saved: {}", path.display())
                            } else {
                                format!("Plan sauvegardé : {}", path.display())
                            };
                            show_toast(ui, msg);
                        }
                        Err(err) => {
                            let msg = if self.lang(ui) == Lang::En {
                                format!("Error saving project: {err}")
                            } else {
                                format!("Erreur lors de la sauvegarde : {err}")
                            };
                            show_toast(ui, msg);
                        }
                    }
                }
            }
            "project:load" => {
                let chosen_path = if is_headless() {
                    let candidate_paths = [
                        export_file_path("mon-plan-renova.json"),
                        std::path::PathBuf::from("mon-plan-renova.json"),
                        std::env::var_os("USERPROFILE")
                            .map(|p| std::path::PathBuf::from(p).join("Documents").join("mon-plan-renova.json"))
                            .unwrap_or_default(),
                        std::env::var_os("HOME")
                            .map(|p| std::path::PathBuf::from(p).join("mon-plan-renova.json"))
                            .unwrap_or_default(),
                    ];
                    candidate_paths.into_iter().filter(|p| !p.as_os_str().is_empty()).find(|p| p.exists())
                } else {
                    let title = if self.lang(ui) == Lang::En {
                        "Open Renova Plan"
                    } else {
                        "Ouvrir un plan Renova"
                    };
                    let filter_label = if self.lang(ui) == Lang::En {
                        "Renova Plan (*.json)"
                    } else {
                        "Plan Renova (*.json)"
                    };
                    let default_dir = export_dir();
                    rfd::FileDialog::new()
                        .set_title(title)
                        .set_directory(&default_dir)
                        .add_filter(filter_label, &["json"])
                        .pick_file()
                };

                if let Some(path) = chosen_path {
                    match std::fs::read(&path)
                        .map_err(|e| e.to_string())
                        .and_then(|bytes| serde_json::from_slice::<Project>(&bytes).map_err(|e| e.to_string()))
                    {
                        Ok(loaded) => {
                            self.push_undo();
                            self.project = loaded;
                            self.selected = None;
                            self.is_dirty = false;
                            self.current_file_path = Some(path.clone());
                            let msg = if self.lang(ui) == Lang::En {
                                format!("Project loaded: {}", path.display())
                            } else {
                                format!("Plan chargé : {}", path.display())
                            };
                            show_toast(ui, msg);
                        }
                        Err(err) => {
                            let msg = if self.lang(ui) == Lang::En {
                                format!("Error opening file: {err}")
                            } else {
                                format!("Erreur lors de l'ouverture du fichier : {err}")
                            };
                            show_toast(ui, msg);
                        }
                    }
                }
            }
            "export:svg" => {
                let chosen_path = if is_headless() {
                    Some(export_file_path("renova-plan-export.svg"))
                } else {
                    let title = if self.lang(ui) == Lang::En {
                        "Export SVG Floor Plan"
                    } else {
                        "Exporter le plan en SVG"
                    };
                    let filter_label = if self.lang(ui) == Lang::En {
                        "Scalable Vector Graphics (*.svg)"
                    } else {
                        "Image vectorielle SVG (*.svg)"
                    };
                    let default_dir = export_dir();
                    rfd::FileDialog::new()
                        .set_title(title)
                        .set_directory(&default_dir)
                        .set_file_name("renova-plan-export.svg")
                        .add_filter(filter_label, &["svg"])
                        .save_file()
                };

                if let Some(path) = chosen_path {
                    let svg = self.project.export_svg(self.phase(ui), self.lang(ui));
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    match std::fs::write(&path, svg) {
                        Ok(()) => {
                            let msg = if self.lang(ui) == Lang::En {
                                format!("SVG exported: {}", path.display())
                            } else {
                                format!("SVG exporté : {}", path.display())
                            };
                            show_toast(ui, msg);
                        }
                        Err(err) => {
                            let msg = if self.lang(ui) == Lang::En {
                                format!("Error exporting SVG: {err}")
                            } else {
                                format!("Erreur export SVG : {err}")
                            };
                            show_toast(ui, msg);
                        }
                    }
                }
            }
            "export:csv" => {
                let chosen_path = if is_headless() {
                    Some(export_file_path("renova-devis-export.csv"))
                } else {
                    let title = if self.lang(ui) == Lang::En {
                        "Export Work Estimate CSV"
                    } else {
                        "Exporter le devis en CSV"
                    };
                    let filter_label = if self.lang(ui) == Lang::En {
                        "CSV Spreadsheet (*.csv)"
                    } else {
                        "Tableur CSV (*.csv)"
                    };
                    let default_dir = export_dir();
                    rfd::FileDialog::new()
                        .set_title(title)
                        .set_directory(&default_dir)
                        .set_file_name("renova-devis-export.csv")
                        .add_filter(filter_label, &["csv"])
                        .save_file()
                };

                if let Some(path) = chosen_path {
                    let csv = self.project.export_csv(self.lang(ui));
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    match std::fs::write(&path, csv) {
                        Ok(()) => {
                            let msg = if self.lang(ui) == Lang::En {
                                format!("CSV devis exported: {}", path.display())
                            } else {
                                format!("Devis CSV exporté : {}", path.display())
                            };
                            show_toast(ui, msg);
                        }
                        Err(err) => {
                            let msg = if self.lang(ui) == Lang::En {
                                format!("Error exporting CSV: {err}")
                            } else {
                                format!("Erreur export CSV : {err}")
                            };
                            show_toast(ui, msg);
                        }
                    }
                }
            }
            _ => {}
        }
        self.refresh_all(ui);
    }

    fn set_selected_status(&mut self, status: RenoStatus) {
        let Some(sel) = self.selected else { return };
        match sel {
            SelectedEntity::Wall(id) => {
                if let Some(w) = self.project.walls.iter_mut().find(|w| w.id == id) {
                    w.status = status;
                }
            }
            SelectedEntity::Opening(id) => {
                if let Some(o) = self.project.openings.iter_mut().find(|o| o.id == id) {
                    o.status = status;
                }
            }
            SelectedEntity::Room(id) => {
                if let Some(r) = self.project.rooms.iter_mut().find(|r| r.id == id) {
                    r.status = status;
                }
            }
            SelectedEntity::Furniture(id) => {
                if let Some(f) = self.project.furniture.iter_mut().find(|f| f.id == id) {
                    f.status = status;
                }
            }
        }
    }

    fn adjust_selected_length(&mut self, delta: f32) {
        let Some(sel) = self.selected else { return };
        match sel {
            SelectedEntity::Wall(id) => {
                if let Some(w) = self.project.walls.iter_mut().find(|w| w.id == id) {
                    let (ux, uy) = w.direction();
                    let new_len = (w.length() + delta).max(0.30);
                    w.x2 = w.x1 + ux * new_len;
                    w.y2 = w.y1 + uy * new_len;
                }
            }
            SelectedEntity::Opening(id) => {
                if let Some(o) = self.project.openings.iter_mut().find(|o| o.id == id) {
                    o.width = (o.width + delta).clamp(0.45, 3.60);
                }
            }
            SelectedEntity::Room(id) => {
                if let Some(r) = self.project.rooms.iter().find(|r| r.id == id).cloned() {
                    self.project.update_room_rect(
                        id,
                        r.x,
                        r.y,
                        (r.width + delta).max(1.0),
                        r.depth,
                    );
                }
            }
            SelectedEntity::Furniture(id) => {
                if let Some(f) = self.project.furniture.iter_mut().find(|f| f.id == id) {
                    f.width = (f.width + delta).clamp(0.30, 5.00);
                }
            }
        }
    }

    fn adjust_selected_thickness(&mut self, delta: f32) {
        let Some(sel) = self.selected else { return };
        match sel {
            SelectedEntity::Wall(id) => {
                if let Some(w) = self.project.walls.iter_mut().find(|w| w.id == id) {
                    w.thickness = (w.thickness + delta).clamp(0.07, 0.60);
                }
            }
            SelectedEntity::Opening(id) => {
                if let Some(o) = self.project.openings.iter_mut().find(|o| o.id == id) {
                    o.sill_height = (o.sill_height + delta).clamp(0.0, 1.50);
                }
            }
            SelectedEntity::Room(id) => {
                if let Some(r) = self.project.rooms.iter().find(|r| r.id == id).cloned() {
                    self.project.update_room_rect(
                        id,
                        r.x,
                        r.y,
                        r.width,
                        (r.depth + delta).max(1.0),
                    );
                }
            }
            SelectedEntity::Furniture(id) => {
                if let Some(f) = self.project.furniture.iter_mut().find(|f| f.id == id) {
                    f.depth = (f.depth + delta).clamp(0.20, 4.00);
                }
            }
        }
    }

    fn rotate_selected(&mut self) {
        let Some(sel) = self.selected else { return };
        match sel {
            SelectedEntity::Furniture(id) => {
                if let Some(f) = self.project.furniture.iter_mut().find(|f| f.id == id) {
                    f.rotation_deg = (f.rotation_deg + 45.0) % 360.0;
                }
            }
            SelectedEntity::Opening(id) => {
                if let Some(o) = self.project.openings.iter_mut().find(|o| o.id == id) {
                    o.flip_swing = !o.flip_swing;
                }
            }
            SelectedEntity::Room(id) => {
                if let Some(r) = self.project.rooms.iter_mut().find(|r| r.id == id) {
                    r.material = match r.material {
                        FloorMaterial::OakParquet => FloorMaterial::LightTile,
                        FloorMaterial::LightTile => FloorMaterial::SlateTile,
                        FloorMaterial::SlateTile => FloorMaterial::PolishedConcrete,
                        FloorMaterial::PolishedConcrete => FloorMaterial::Terracotta,
                        FloorMaterial::Terracotta => FloorMaterial::WarmCarpet,
                        FloorMaterial::WarmCarpet => FloorMaterial::OakParquet,
                    };
                }
            }
            SelectedEntity::Wall(id) => {
                if let Some(w) = self.project.walls.iter_mut().find(|w| w.id == id) {
                    w.load_bearing = !w.load_bearing;
                }
            }
        }
    }

    fn duplicate_selected(&mut self) {
        let Some(sel) = self.selected else { return };
        match sel {
            SelectedEntity::Furniture(id) => {
                if let Some(f) = self.project.furniture.iter().find(|f| f.id == id).cloned() {
                    let new_id = self.project.add_furniture(
                        f.level,
                        f.kind,
                        f.x + 0.50,
                        f.y + 0.50,
                        f.rotation_deg,
                        f.status,
                    );
                    self.selected = Some(SelectedEntity::Furniture(new_id));
                }
            }
            SelectedEntity::Room(id) => {
                if let Some(r) = self.project.rooms.iter().find(|r| r.id == id).cloned() {
                    let new_id = self.project.add_ready_room(
                        r.level,
                        &r.name_fr,
                        &r.name_en,
                        r.x + r.width,
                        r.y,
                        r.width,
                        r.depth,
                        r.material,
                        r.status,
                    );
                    self.selected = Some(SelectedEntity::Room(new_id));
                }
            }
            SelectedEntity::Wall(id) => {
                if let Some(w) = self.project.walls.iter().find(|w| w.id == id).cloned() {
                    let new_id = self.project.add_wall(
                        w.level,
                        w.x1 + 0.50,
                        w.y1 + 0.50,
                        w.x2 + 0.50,
                        w.y2 + 0.50,
                        w.thickness,
                        w.height,
                        w.load_bearing,
                        w.status,
                        &w.name,
                    );
                    self.selected = Some(SelectedEntity::Wall(new_id));
                }
            }
            SelectedEntity::Opening(id) => {
                if let Some(o) = self.project.openings.iter().find(|o| o.id == id).cloned() {
                    let new_id = self.project.add_opening(
                        o.level,
                        o.wall_id,
                        (o.offset_t + 0.15).clamp(0.10, 0.90),
                        o.kind,
                        o.status,
                        o.flip_swing,
                    );
                    self.selected = Some(SelectedEntity::Opening(new_id));
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_pointer_2d(
        &mut self,
        ui: &RenovaWindow,
        kind: &str,
        sx: f32,
        sy: f32,
        vw: f32,
        vh: f32,
        button: i32,
    ) {
        let view_mode = ui.get_view_mode();
        let buf_w = if view_mode == 1 { 560.0 } else { 1020.0 };
        let buf_h = 700.0;
        let bx = sx * (buf_w / vw.max(1.0));
        let by = sy * (buf_h / vh.max(1.0));

        let (raw_wx, raw_wy) = self
            .cam2d
            .screen_to_world(bx, by, buf_w as u32, buf_h as u32);
        let phase = self.phase(ui);
        let (snap_wx, snap_wy) = self.project.snap_point(raw_wx, raw_wy, phase);
        self.hover_world = Some((snap_wx, snap_wy));
        ui.set_status_coords(format!("X: {snap_wx:.2} m | Y: {snap_wy:.2} m").into());

        let tool = ui.get_active_tool();

        match kind {
            "down" => {
                if button == 2 || button == 3 {
                    self.drag2d = Drag2DState::Panning {
                        start_sx: bx,
                        start_sy: by,
                        orig_pan_x: self.cam2d.pan_x,
                        orig_pan_y: self.cam2d.pan_y,
                    };
                    return;
                }

                // Always allow grabbing a resize handle of the currently selected Room (8 handles: 4 corners + 4 sides) or Wall (2 endpoints)
                if let Some(SelectedEntity::Room(r_id)) = self.selected
                    && let Some(r) = self.project.rooms.iter().find(|r| r.id == r_id)
                {
                    let mx = r.x + r.width * 0.5;
                    let my = r.y + r.depth * 0.5;
                    let handles: [(f32, f32, Option<f32>, Option<f32>); 8] = [
                        // 4 corners
                        (r.x, r.y, Some(r.x + r.width), Some(r.y + r.depth)),
                        (r.x + r.width, r.y, Some(r.x), Some(r.y + r.depth)),
                        (r.x + r.width, r.y + r.depth, Some(r.x), Some(r.y)),
                        (r.x, r.y + r.depth, Some(r.x + r.width), Some(r.y)),
                        // 4 wall midpoints (Top, Right, Bottom, Left)
                        (mx, r.y, None, Some(r.y + r.depth)),
                        (r.x + r.width, my, Some(r.x), None),
                        (mx, r.y + r.depth, None, Some(r.y)),
                        (r.x, my, Some(r.x + r.width), None),
                    ];
                    for &(hx, hy, ax, ay) in &handles {
                        if ((raw_wx - hx).powi(2) + (raw_wy - hy).powi(2)).sqrt() <= 0.42 {
                            self.push_undo();
                            self.drag2d = Drag2DState::ResizingRoom {
                                room_id: r_id,
                                anchor_x: ax,
                                anchor_y: ay,
                            };
                            self.refresh_all(ui);
                            return;
                        }
                    }
                }
                if let Some(SelectedEntity::Wall(w_id)) = self.selected
                    && let Some(w) = self.project.walls.iter().find(|w| w.id == w_id)
                {
                    let d1 = ((raw_wx - w.x1).powi(2) + (raw_wy - w.y1).powi(2)).sqrt();
                    let d2 = ((raw_wx - w.x2).powi(2) + (raw_wy - w.y2).powi(2)).sqrt();
                    if d1 <= 0.35 || d2 <= 0.35 {
                        self.push_undo();
                        self.drag2d = Drag2DState::ResizingWallEndpoint {
                            wall_id: w_id,
                            is_end2: d2 < d1,
                        };
                        self.refresh_all(ui);
                        return;
                    }
                }

                match tool.as_str() {
                    "wall" | "partition" => {
                        self.drag2d = Drag2DState::DrawingWall {
                            start_wx: snap_wx,
                            start_wy: snap_wy,
                            cur_wx: snap_wx,
                            cur_wy: snap_wy,
                            load_bearing: tool == "wall",
                        };
                    }
                    "room" | "boxed_room" => {
                        self.drag2d = Drag2DState::DrawingRoom {
                            start_wx: snap_wx,
                            start_wy: snap_wy,
                            cur_wx: snap_wx,
                            cur_wy: snap_wy,
                            boxed: tool == "boxed_room",
                        };
                    }
                    "demolish_click" => {
                        self.selected = self.project.hit_test(raw_wx, raw_wy, phase);
                        if self.selected.is_some() {
                            self.push_undo();
                            self.set_selected_status(RenoStatus::Demolish);
                            ui.set_right_tab(0);
                        }
                    }
                    "measure" => {
                        self.drag2d = Drag2DState::Measuring {
                            start_wx: snap_wx,
                            start_wy: snap_wy,
                            cur_wx: snap_wx,
                            cur_wy: snap_wy,
                        };
                    }
                    "door" | "double_door" | "sliding_door" | "passage" | "window"
                    | "small_window" | "bay_window" => {
                        if let Some((wall_id, t)) = self.project.closest_wall(raw_wx, raw_wy, phase)
                        {
                            self.push_undo();
                            let kind = match tool.as_str() {
                                "door" => OpeningKind::SingleDoor,
                                "double_door" => OpeningKind::DoubleDoor,
                                "sliding_door" => OpeningKind::SlidingDoor,
                                "passage" => OpeningKind::OpenPassage,
                                "small_window" => OpeningKind::SingleWindow,
                                "bay_window" => OpeningKind::BayWindow,
                                _ => OpeningKind::DoubleWindow,
                            };
                            let op_id = self.project.add_opening(
                                self.project.active_level,
                                wall_id,
                                t,
                                kind,
                                self.active_reno_status(ui),
                                false,
                            );
                            self.selected = Some(SelectedEntity::Opening(op_id));
                            ui.set_right_tab(0);
                        }
                    }
                    _ => {
                        self.selected = self.project.hit_test(raw_wx, raw_wy, phase);
                        if let Some(entity) = self.selected {
                            ui.set_right_tab(0);
                            self.push_undo();
                            self.drag2d = Drag2DState::MovingEntity {
                                entity,
                                last_wx: snap_wx,
                                last_wy: snap_wy,
                            };
                        } else if ui.get_guided_step() == 0 {
                            // En mode "+ Pièces & Murs", glisser sur le quadrillage vide dessine directement une pièce complète !
                            self.drag2d = Drag2DState::DrawingRoom {
                                start_wx: snap_wx,
                                start_wy: snap_wy,
                                cur_wx: snap_wx,
                                cur_wy: snap_wy,
                                boxed: true,
                            };
                        } else {
                            self.drag2d = Drag2DState::Panning {
                                start_sx: bx,
                                start_sy: by,
                                orig_pan_x: self.cam2d.pan_x,
                                orig_pan_y: self.cam2d.pan_y,
                            };
                        }
                    }
                }
            }
            "move" => {
                if button == 0 && !matches!(self.drag2d, Drag2DState::None) {
                    self.drag2d = Drag2DState::None;
                }
                match self.drag2d {
                    Drag2DState::Panning {
                        start_sx,
                        start_sy,
                        orig_pan_x,
                        orig_pan_y,
                    } => {
                        self.cam2d.pan_x = orig_pan_x - (bx - start_sx) / self.cam2d.zoom;
                        self.cam2d.pan_y = orig_pan_y - (by - start_sy) / self.cam2d.zoom;
                    }
                    Drag2DState::MovingEntity {
                        entity,
                        ref mut last_wx,
                        ref mut last_wy,
                    } => {
                        let dx = snap_wx - *last_wx;
                        let dy = snap_wy - *last_wy;
                        if dx.abs() > 1e-4 || dy.abs() > 1e-4 {
                            move_entity(&mut self.project, entity, dx, dy, raw_wx, raw_wy);
                            *last_wx = snap_wx;
                            *last_wy = snap_wy;
                        }
                    }
                    Drag2DState::ResizingWallEndpoint { wall_id, is_end2 } => {
                        if let Some(w) = self.project.walls.iter_mut().find(|w| w.id == wall_id) {
                            if is_end2 {
                                if ((snap_wx - w.x1).powi(2) + (snap_wy - w.y1).powi(2)).sqrt()
                                    >= 0.30
                                {
                                    w.x2 = snap_wx;
                                    w.y2 = snap_wy;
                                }
                            } else if ((snap_wx - w.x2).powi(2) + (snap_wy - w.y2).powi(2)).sqrt()
                                >= 0.30
                            {
                                w.x1 = snap_wx;
                                w.y1 = snap_wy;
                            }
                        }
                    }
                    Drag2DState::ResizingRoom {
                        room_id,
                        anchor_x,
                        anchor_y,
                    } => {
                        if let Some(r) =
                            self.project.rooms.iter().find(|r| r.id == room_id).cloned()
                        {
                            let (nx, nw) = match anchor_x {
                                Some(ax) => {
                                    let w = (snap_wx - ax).abs().max(1.0);
                                    let x = if snap_wx < ax { ax - w } else { ax };
                                    (x, w)
                                }
                                None => (r.x, r.width),
                            };
                            let (ny, nd) = match anchor_y {
                                Some(ay) => {
                                    let d = (snap_wy - ay).abs().max(1.0);
                                    let y = if snap_wy < ay { ay - d } else { ay };
                                    (y, d)
                                }
                                None => (r.y, r.depth),
                            };
                            self.project.update_room_rect(room_id, nx, ny, nw, nd);
                        }
                    }
                    Drag2DState::DrawingWall {
                        ref mut cur_wx,
                        ref mut cur_wy,
                        ..
                    }
                    | Drag2DState::DrawingRoom {
                        ref mut cur_wx,
                        ref mut cur_wy,
                        ..
                    }
                    | Drag2DState::Measuring {
                        ref mut cur_wx,
                        ref mut cur_wy,
                        ..
                    } => {
                        *cur_wx = snap_wx;
                        *cur_wy = snap_wy;
                    }
                    Drag2DState::None => {}
                }
            }
            "up" => {
                match self.drag2d {
                    Drag2DState::DrawingWall {
                        start_wx,
                        start_wy,
                        cur_wx,
                        cur_wy,
                        load_bearing,
                    } => {
                        let len =
                            ((cur_wx - start_wx).powi(2) + (cur_wy - start_wy).powi(2)).sqrt();
                        if len >= 0.25 {
                            self.push_undo();
                            let status = self.active_reno_status(ui);
                            let thick = if load_bearing { 0.25 } else { 0.10 };
                            let name = if load_bearing {
                                "Mur Porteur"
                            } else {
                                "Cloison BA13"
                            };
                            let id = self.project.add_wall(
                                self.project.active_level,
                                start_wx,
                                start_wy,
                                cur_wx,
                                cur_wy,
                                thick,
                                2.60,
                                load_bearing,
                                status,
                                name,
                            );
                            self.selected = Some(SelectedEntity::Wall(id));
                        }
                    }
                    Drag2DState::DrawingRoom {
                        start_wx,
                        start_wy,
                        cur_wx,
                        cur_wy,
                        boxed,
                    } => {
                        let w = (cur_wx - start_wx).abs();
                        let d = (cur_wy - start_wy).abs();
                        if w >= 0.75 && d >= 0.75 {
                            self.push_undo();
                            let status = self.active_reno_status(ui);
                            let id = if boxed {
                                self.project.add_boxed_room(
                                    self.project.active_level,
                                    "Nouvelle Pièce",
                                    "New Room",
                                    start_wx.min(cur_wx),
                                    start_wy.min(cur_wy),
                                    w,
                                    d,
                                    FloorMaterial::OakParquet,
                                    status,
                                )
                            } else {
                                self.project.add_room(
                                    self.project.active_level,
                                    "Nouvelle Pièce",
                                    "New Room",
                                    start_wx.min(cur_wx),
                                    start_wy.min(cur_wy),
                                    w,
                                    d,
                                    FloorMaterial::OakParquet,
                                    status,
                                )
                            };
                            self.selected = Some(SelectedEntity::Room(id));
                            ui.set_right_tab(0);
                            ui.set_active_tool("select".into());
                        }
                    }
                    Drag2DState::Measuring {
                        start_wx,
                        start_wy,
                        cur_wx,
                        cur_wy,
                    } => {
                        self.measure_line = Some((start_wx, start_wy, cur_wx, cur_wy));
                    }
                    _ => {}
                }
                self.drag2d = Drag2DState::None;
            }
            _ => {}
        }
        self.refresh_all(ui);
    }

    fn handle_pointer_3d(&mut self, ui: &RenovaWindow, kind: &str, sx: f32, sy: f32, button: i32) {
        match kind {
            "down" => {
                self.drag3d_last = Some((sx, sy, button));
            }
            "move" => {
                if let Some((lx, ly, btn)) = self.drag3d_last {
                    let dx = sx - lx;
                    let dy = sy - ly;
                    if btn == 2 {
                        self.cam3d.target_x -= dx * 0.015;
                        self.cam3d.target_y -= dy * 0.015;
                    } else {
                        self.cam3d.yaw_deg = (self.cam3d.yaw_deg + dx * 0.45) % 360.0;
                        self.cam3d.pitch_deg = (self.cam3d.pitch_deg + dy * 0.35).clamp(10.0, 84.0);
                    }
                    self.drag3d_last = Some((sx, sy, btn));
                    self.refresh_all(ui);
                }
            }
            "up" => {
                self.drag3d_last = None;
            }
            _ => {}
        }
    }
}

fn move_entity(
    project: &mut Project,
    entity: SelectedEntity,
    dx: f32,
    dy: f32,
    raw_wx: f32,
    raw_wy: f32,
) {
    match entity {
        SelectedEntity::Furniture(id) => {
            if let Some(f) = project.furniture.iter_mut().find(|f| f.id == id) {
                f.x += dx;
                f.y += dy;
            }
        }
        SelectedEntity::Wall(id) => {
            if let Some(w) = project.walls.iter_mut().find(|w| w.id == id) {
                w.x1 += dx;
                w.y1 += dy;
                w.x2 += dx;
                w.y2 += dy;
            }
        }
        SelectedEntity::Room(id) => {
            if let Some(r) = project.rooms.iter().find(|r| r.id == id).cloned() {
                project.update_room_rect(id, r.x + dx, r.y + dy, r.width, r.depth);
            }
        }
        SelectedEntity::Opening(id) => {
            if let Some(op) = project.openings.iter_mut().find(|o| o.id == id)
                && let Some(w) = project.walls.iter().find(|w| w.id == op.wall_id)
            {
                let (t, _) = w.project_point(raw_wx, raw_wy);
                op.offset_t = t.clamp(0.10, 0.90);
            }
        }
    }
}

fn status_to_int(s: RenoStatus) -> i32 {
    match s {
        RenoStatus::Existing => 0,
        RenoStatus::Demolish => 1,
        RenoStatus::New => 2,
    }
}

fn parse_furniture_kind(id: &str) -> Option<FurnitureKind> {
    FurnitureKind::ALL
        .iter()
        .copied()
        .find(|k| format!("{k:?}") == id)
}

fn is_headless() -> bool {
    std::env::var("RENOVA_SCREENSHOT").is_ok()
        || std::env::var("RENOVA_HEADLESS").is_ok()
        || std::env::args().any(|a| a == "--smoke-test")
}

fn export_dir() -> std::path::PathBuf {
    #[cfg(windows)]
    {
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            let docs = std::path::PathBuf::from(profile).join("Documents");
            if docs.is_dir() {
                let ren = docs.join("RenovaPlan");
                let _ = std::fs::create_dir_all(&ren);
                return ren;
            }
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let ren = std::path::PathBuf::from(home).join("RenovaPlan");
            let _ = std::fs::create_dir_all(&ren);
            return ren;
        }
    }
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

fn export_file_path(filename: &str) -> std::path::PathBuf {
    export_dir().join(filename)
}

fn show_toast(ui: &RenovaWindow, msg: String) {
    ui.set_toast_message(msg.into());
    let weak = ui.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(3200), move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_toast_message("".into());
        }
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .select()?;
    let _ = slint::set_xdg_app_id("io.github.tonyboysuper.RenovaPlan");

    let ui = RenovaWindow::new()?;
    let i18n = I18nData::load();
    let prefs = Preferences::load();

    ui.set_language_index(prefs.language as i32);
    ui.set_theme_index(prefs.theme as i32);
    ui.set_large_text(prefs.large_text);
    ui.set_expert_mode(prefs.expert_mode);

    let state = Rc::new(RefCell::new(AppState {
        project: Project::preset_creation_libre(),
        undo_stack: Vec::new(),
        redo_stack: Vec::new(),
        cam2d: Camera2D::default(),
        cam3d: Camera3D::default(),
        selected: None,
        drag2d: Drag2DState::None,
        drag3d_last: None,
        hover_world: None,
        measure_line: None,
        i18n,
        prefs,
        is_dirty: false,
        current_file_path: None,
    }));

    {
        let mut app = state.borrow_mut();
        app.apply_theme(&ui);
        app.refresh_all(&ui);
    }

    let s_tr = state.clone();
    ui.on_translate(move |key, lang_idx| s_tr.borrow().i18n.get(&key, lang_idx).into());

    let s_act = state.clone();
    let weak = ui.as_weak();
    ui.on_action(move |cmd| {
        let ui = weak.unwrap();
        s_act.borrow_mut().handle_action(&ui, &cmd);
    });

    let s_cat = state.clone();
    let weak = ui.as_weak();
    ui.on_select_catalog_cat(move |_| {
        let ui = weak.unwrap();
        s_cat.borrow().refresh_catalog(&ui);
    });

    let s_furn = state.clone();
    let weak = ui.as_weak();
    ui.on_add_furniture_item(move |id| {
        let ui = weak.unwrap();
        let mut app = s_furn.borrow_mut();
        if let Some(kind) = parse_furniture_kind(&id) {
            app.push_undo();
            let lvl = app.project.active_level;
            let st = app.active_reno_status(&ui);
            let cx = app.cam2d.pan_x;
            let cy = app.cam2d.pan_y;
            let item_id = app.project.add_furniture(lvl, kind, cx, cy, 0.0, st);
            app.selected = Some(SelectedEntity::Furniture(item_id));
            ui.set_active_tool("select".into());
            app.refresh_all(&ui);
        }
    });

    let s_p2d = state.clone();
    let weak = ui.as_weak();
    ui.on_pointer_2d(move |kind, x, y, w, h, btn| {
        let ui = weak.unwrap();
        s_p2d
            .borrow_mut()
            .handle_pointer_2d(&ui, &kind, x, y, w, h, btn);
    });

    let s_p3d = state.clone();
    let weak = ui.as_weak();
    ui.on_pointer_3d(move |kind, x, y, _w, _h, btn| {
        let ui = weak.unwrap();
        s_p3d.borrow_mut().handle_pointer_3d(&ui, &kind, x, y, btn);
    });

    let s_w2d = state.clone();
    let weak = ui.as_weak();
    ui.on_wheel_2d(move |_x, _y, _w, _h, delta_y| {
        let ui = weak.unwrap();
        let mut app = s_w2d.borrow_mut();
        let factor = if delta_y > 0.0 { 1.12 } else { 1.0 / 1.12 };
        app.cam2d.zoom = (app.cam2d.zoom * factor).clamp(24.0, 180.0);
        app.refresh_all(&ui);
    });

    let s_w3d = state.clone();
    let weak = ui.as_weak();
    ui.on_wheel_3d(move |delta_y| {
        let ui = weak.unwrap();
        let mut app = s_w3d.borrow_mut();
        let factor = if delta_y > 0.0 { 0.90 } else { 1.10 };
        app.cam3d.distance = (app.cam3d.distance * factor).clamp(4.0, 28.0);
        app.refresh_all(&ui);
    });

    // Winit Native Keyboard Shortcuts (Wayland / Pop!_OS)
    let s_key = state.clone();
    let weak = ui.as_weak();
    let mut modifiers = ModifiersState::empty();
    ui.window().on_winit_window_event(move |_, event| {
        if let WindowEvent::ModifiersChanged(m) = event {
            modifiers = m.state();
        }
        let WindowEvent::KeyboardInput { event, .. } = event else {
            return EventResult::Propagate;
        };
        if event.state != ElementState::Pressed {
            return EventResult::Propagate;
        }
        let ui = weak.unwrap();
        if event.logical_key == NativeKey::Named(NamedKey::Escape) {
            if ui.get_show_confirm_dialog() {
                ui.set_show_confirm_dialog(false);
            } else if ui.get_show_about() {
                ui.set_show_about(false);
            } else if ui.get_show_welcome() {
                ui.set_show_welcome(false);
            } else {
                ui.set_active_tool("select".into());
                let mut app = s_key.borrow_mut();
                app.selected = None;
                app.measure_line = None;
                app.refresh_all(&ui);
            }
            return EventResult::PreventDefault;
        }
        if matches!(
            event.logical_key,
            NativeKey::Named(NamedKey::Delete | NamedKey::Backspace)
        ) {
            s_key.borrow_mut().handle_action(&ui, "sel:delete");
            return EventResult::PreventDefault;
        }
        if matches!(event.logical_key, NativeKey::Named(NamedKey::F1)) {
            ui.set_show_about(!ui.get_show_about());
            return EventResult::PreventDefault;
        }
        if let NativeKey::Character(ch) = &event.logical_key {
            let lower = ch.to_lowercase();
            if lower == "?" {
                ui.set_show_about(!ui.get_show_about());
                return EventResult::PreventDefault;
            }
            if modifiers.control_key() || modifiers.super_key() {
                if modifiers.shift_key() && lower == "s" {
                    s_key.borrow_mut().handle_action(&ui, "project:save_as");
                    return EventResult::PreventDefault;
                }
                match lower.as_str() {
                    "z" => {
                        s_key.borrow_mut().handle_action(&ui, "undo");
                        return EventResult::PreventDefault;
                    }
                    "y" => {
                        s_key.borrow_mut().handle_action(&ui, "redo");
                        return EventResult::PreventDefault;
                    }
                    "s" => {
                        s_key.borrow_mut().handle_action(&ui, "project:save");
                        return EventResult::PreventDefault;
                    }
                    "o" => {
                        s_key.borrow_mut().handle_action(&ui, "project:load");
                        return EventResult::PreventDefault;
                    }
                    _ => {}
                }
            } else {
                match lower.as_str() {
                    "v" => ui.set_active_tool("select".into()),
                    "w" => ui.set_active_tool("wall".into()),
                    "p" => ui.set_active_tool("partition".into()),
                    "r" => ui.set_active_tool("boxed_room".into()),
                    "d" => ui.set_active_tool("door".into()),
                    "f" => ui.set_active_tool("window".into()),
                    "m" => ui.set_active_tool("measure".into()),
                    "t" => s_key.borrow_mut().handle_action(&ui, "sel:rotate"),
                    _ => return EventResult::Propagate,
                }
                s_key.borrow_mut().refresh_all(&ui);
                return EventResult::PreventDefault;
            }
        }
        EventResult::Propagate
    });

    #[cfg(debug_assertions)]
    if std::env::args().any(|a| a == "--smoke-test") {
        smoke::run(&ui);
        ui.show()?;
        slint::run_event_loop_until_quit()?;
        return Ok(());
    }

    ui.run()?;
    Ok(())
}
