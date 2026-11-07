use crate::canvas::{CanvasBuffer, Color};
use crate::model::{Lang, RenoStatus, SelectedEntity, ViewPhase};
use crate::project::Project;

#[derive(Clone, Debug)]
pub struct Camera2D {
    pub pan_x: f32, // center of view in meters
    pub pan_y: f32,
    pub zoom: f32, // pixels per meter (default 58.0)
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            pan_x: 5.5,
            pan_y: 4.0,
            zoom: 42.0,
        }
    }
}

impl Camera2D {
    pub fn world_to_screen(&self, wx: f32, wy: f32, width: u32, height: u32) -> (f32, f32) {
        let cx = width as f32 * 0.5;
        let cy = height as f32 * 0.5;
        (
            cx + (wx - self.pan_x) * self.zoom,
            cy + (wy - self.pan_y) * self.zoom,
        )
    }

    pub fn screen_to_world(&self, sx: f32, sy: f32, width: u32, height: u32) -> (f32, f32) {
        let cx = width as f32 * 0.5;
        let cy = height as f32 * 0.5;
        (
            self.pan_x + (sx - cx) / self.zoom,
            self.pan_y + (sy - cy) / self.zoom,
        )
    }
}

pub struct Render2DOptions {
    pub dark_mode: bool,
    pub phase: ViewPhase,
    pub lang: Lang,
    pub selected: Option<SelectedEntity>,
    pub draft_line: Option<(f32, f32, f32, f32, RenoStatus)>,
    pub measure_line: Option<(f32, f32, f32, f32)>,
    pub hover_world: Option<(f32, f32)>,
}

pub fn render_plan_2d(
    project: &Project,
    cam: &Camera2D,
    width: u32,
    height: u32,
    opts: &Render2DOptions,
) -> CanvasBuffer {
    let bg = if opts.dark_mode {
        Color::rgb(20, 23, 30)
    } else {
        Color::rgb(248, 250, 252)
    };
    let mut buf = CanvasBuffer::new(width, height, bg);

    // 1. Grid & Underlay Blueprint
    if project.show_grid {
        let minor_col = if opts.dark_mode {
            Color::rgb(31, 36, 46)
        } else {
            Color::rgb(233, 238, 244)
        };
        let major_col = if opts.dark_mode {
            Color::rgb(44, 51, 66)
        } else {
            Color::rgb(212, 220, 232)
        };

        let (min_wx, min_wy) = cam.screen_to_world(0.0, 0.0, width, height);
        let (max_wx, max_wy) = cam.screen_to_world(width as f32, height as f32, width, height);

        let step = 0.25_f32;
        if cam.zoom * step >= 9.0 {
            let mut gx = (min_wx / step).floor() * step;
            while gx <= max_wx {
                let (sx, _) = cam.world_to_screen(gx, 0.0, width, height);
                let is_meter = (gx.round() - gx).abs() < 0.01;
                buf.fill_rect(
                    sx as i32,
                    0,
                    1,
                    height as i32,
                    if is_meter { major_col } else { minor_col },
                );
                gx += step;
            }
            let mut gy = (min_wy / step).floor() * step;
            while gy <= max_wy {
                let (_, sy) = cam.world_to_screen(0.0, gy, width, height);
                let is_meter = (gy.round() - gy).abs() < 0.01;
                buf.fill_rect(
                    0,
                    sy as i32,
                    width as i32,
                    1,
                    if is_meter { major_col } else { minor_col },
                );
                gy += step;
            }
        }
    }

    // 2. Underlay Reference Blueprint (Calque de fond architecte étalonné)
    if project.show_underlay {
        let alpha = (project.underlay_opacity.clamp(0.05, 0.9) * 140.0) as u8;
        let blueprint_col = if opts.dark_mode {
            Color::rgba(56, 189, 248, alpha)
        } else {
            Color::rgba(2, 132, 199, alpha)
        };
        let (ux0, uy0) = cam.world_to_screen(-0.6, -0.6, width, height);
        let (ux1, uy1) = cam.world_to_screen(
            project.underlay_scale_m,
            project.underlay_scale_m * 0.72,
            width,
            height,
        );
        buf.stroke_rect(
            ux0 as i32,
            uy0 as i32,
            (ux1 - ux0) as i32,
            (uy1 - uy0) as i32,
            1,
            blueprint_col,
        );
        let label = match opts.lang {
            Lang::Fr => format!("CALQUE ARCHITECTE ({:.1}m)", project.underlay_scale_m),
            Lang::En => format!("BLUEPRINT UNDERLAY ({:.1}m)", project.underlay_scale_m),
        };
        buf.draw_text(ux0 as i32 + 6, uy0 as i32 + 6, &label, 1, blueprint_col);
    }

    // 3. Rooms & Flooring textures
    for r in &project.rooms {
        if r.level != project.active_level || !r.status.is_visible_in(opts.phase) {
            continue;
        }
        let (sx0, sy0) = cam.world_to_screen(r.x, r.y, width, height);
        let (sx1, sy1) = cam.world_to_screen(r.x + r.width, r.y + r.depth, width, height);
        let rx = sx0.min(sx1) as i32;
        let ry = sy0.min(sy1) as i32;
        let rw = (sx1 - sx0).abs() as i32;
        let rh = (sy1 - sy0).abs() as i32;

        let (mr, mg, mb) = r.material.rgb();
        let fill_alpha = if opts.dark_mode { 120 } else { 95 };
        buf.fill_rect(rx, ry, rw, rh, Color::rgba(mr, mg, mb, fill_alpha));

        // Subtle parquet/tile pattern lines
        let stripe_col = Color::rgba(mr, mg, mb, if opts.dark_mode { 65 } else { 55 });
        let step_px = (cam.zoom * 0.5).max(8.0) as i32;
        let mut py = ry + step_px;
        while py < ry + rh {
            buf.fill_rect(rx, py, rw, 1, stripe_col);
            py += step_px;
        }

        let is_sel = opts.selected == Some(SelectedEntity::Room(r.id));
        if is_sel {
            buf.stroke_rect(rx, ry, rw, rh, 2, Color::rgb(245, 158, 11));
        }
    }

    // 4. Furniture items (with architectural symbols)
    for item in &project.furniture {
        if item.level != project.active_level || !item.status.is_visible_in(opts.phase) {
            continue;
        }
        let (cx, cy) = cam.world_to_screen(item.x, item.y, width, height);
        let hw = item.width * 0.5 * cam.zoom;
        let hd = item.depth * 0.5 * cam.zoom;
        let rad = item.rotation_deg.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();

        let rot = |lx: f32, ly: f32| -> (f32, f32) {
            (cx + lx * cos_a - ly * sin_a, cy + lx * sin_a + ly * cos_a)
        };

        let p0 = rot(-hw, -hd);
        let p1 = rot(hw, -hd);
        let p2 = rot(hw, hd);
        let p3 = rot(-hw, hd);

        let (fr, fg, fb) = item.kind.color_rgb();
        let fill_c = match item.status {
            RenoStatus::Demolish => Color::rgba(239, 68, 68, 110),
            RenoStatus::New => Color::rgba(fr, fg, fb, 210),
            RenoStatus::Existing => Color::rgba(fr, fg, fb, 180),
        };
        buf.fill_quad_2d(p0, p1, p2, p3, fill_c);

        let is_sel = opts.selected == Some(SelectedEntity::Furniture(item.id));
        let border_c = if is_sel {
            Color::rgb(245, 158, 11)
        } else if item.status == RenoStatus::New {
            Color::rgb(16, 185, 129)
        } else if item.status == RenoStatus::Demolish {
            Color::rgb(239, 68, 68)
        } else {
            Color::rgb(71, 85, 105)
        };
        let b_thick = if is_sel { 2.5 } else { 1.4 };
        buf.draw_thick_line(p0.0, p0.1, p1.0, p1.1, b_thick, border_c, false);
        buf.draw_thick_line(p1.0, p1.1, p2.0, p2.1, b_thick, border_c, false);
        buf.draw_thick_line(p2.0, p2.1, p3.0, p3.1, b_thick, border_c, false);
        buf.draw_thick_line(p3.0, p3.1, p0.0, p0.1, b_thick, border_c, false);
        if is_sel {
            for &(hx, hy) in &[p0, p1, p2, p3] {
                buf.fill_circle(hx, hy, 5.5, Color::rgb(217, 119, 6));
                buf.fill_circle(hx, hy, 3.0, Color::rgb(255, 255, 255));
            }
        }

        // Inner backrest / detail line for orientation clarity
        let b0 = rot(-hw * 0.85, -hd * 0.72);
        let b1 = rot(hw * 0.85, -hd * 0.72);
        buf.draw_thick_line(
            b0.0,
            b0.1,
            b1.0,
            b1.1,
            1.2,
            Color::rgba(15, 23, 42, 130),
            false,
        );

        if is_sel && cam.zoom >= 34.0 {
            let short = item.display_name(opts.lang);
            let short_label: String = short.chars().take(16).collect();
            let badge_bg = if opts.dark_mode {
                Color::rgba(15, 23, 42, 230)
            } else {
                Color::rgba(255, 255, 255, 240)
            };
            buf.draw_badge_text(
                cx as i32,
                cy as i32,
                &short_label,
                1,
                Color::rgb(217, 119, 6),
                badge_bg,
            );
        }
    }

    // 5. Walls (Existing Slate, Demolition Dashed Red, New Creation Emerald)
    for w in &project.walls {
        if w.level != project.active_level || !w.status.is_visible_in(opts.phase) {
            continue;
        }
        let (x1, y1) = cam.world_to_screen(w.x1, w.y1, width, height);
        let (x2, y2) = cam.world_to_screen(w.x2, w.y2, width, height);
        let px_thick = (w.thickness * cam.zoom).max(5.0);
        let is_sel = opts.selected == Some(SelectedEntity::Wall(w.id));

        let wall_col = if is_sel {
            Color::rgb(245, 158, 11)
        } else {
            match w.status {
                RenoStatus::Existing => {
                    if opts.dark_mode {
                        if w.load_bearing {
                            Color::rgb(148, 163, 184)
                        } else {
                            Color::rgb(100, 116, 139)
                        }
                    } else if w.load_bearing {
                        Color::rgb(30, 41, 59)
                    } else {
                        Color::rgb(71, 85, 105)
                    }
                }
                RenoStatus::Demolish => Color::rgb(239, 68, 68),
                RenoStatus::New => {
                    if opts.dark_mode {
                        Color::rgb(52, 211, 153)
                    } else {
                        Color::rgb(5, 150, 105)
                    }
                }
            }
        };

        let dashed = w.status == RenoStatus::Demolish;
        buf.draw_thick_line(x1, y1, x2, y2, px_thick, wall_col, dashed);

        // Endpoint joint caps
        buf.fill_circle(x1, y1, px_thick * 0.48, wall_col);
        buf.fill_circle(x2, y2, px_thick * 0.48, wall_col);

        // Selection handles (RoomSketcher / Floorplanner draggable endpoints)
        if is_sel {
            buf.fill_circle(x1, y1, 8.0, Color::rgb(217, 119, 6));
            buf.fill_circle(x1, y1, 4.5, Color::rgb(255, 255, 255));
            buf.fill_circle(x2, y2, 8.0, Color::rgb(217, 119, 6));
            buf.fill_circle(x2, y2, 4.5, Color::rgb(255, 255, 255));
        }

        // Architectural dimension lines (Cotations)
        if project.show_dimensions && w.length() >= 0.90 && cam.zoom >= 34.0 {
            let (nx, ny) = w.normal();
            let offset_px = px_thick * 0.5 + 16.0;
            let dx1 = x1 + nx * offset_px;
            let dy1 = y1 + ny * offset_px;
            let dx2 = x2 + nx * offset_px;
            let dy2 = y2 + ny * offset_px;

            let dim_col = if is_sel {
                Color::rgb(217, 119, 6)
            } else if opts.dark_mode {
                Color::rgba(56, 189, 248, 210)
            } else {
                Color::rgba(3, 105, 161, 230)
            };

            buf.draw_thick_line(dx1, dy1, dx2, dy2, 1.3, dim_col, false);
            // Extension ticks
            buf.draw_thick_line(
                x1 + nx * 4.0,
                y1 + ny * 4.0,
                dx1 + nx * 4.0,
                dy1 + ny * 4.0,
                1.0,
                dim_col,
                false,
            );
            buf.draw_thick_line(
                x2 + nx * 4.0,
                y2 + ny * 4.0,
                dx2 + nx * 4.0,
                dy2 + ny * 4.0,
                1.0,
                dim_col,
                false,
            );

            let mx = (dx1 + dx2) * 0.5;
            let my = (dy1 + dy2) * 0.5;
            let text = format!("{:.2}m", w.length());
            let badge_bg = if opts.dark_mode {
                Color::rgba(15, 23, 42, 235)
            } else {
                Color::rgba(255, 255, 255, 245)
            };
            buf.draw_badge_text(mx as i32, my as i32, &text, 1, dim_col, badge_bg);
        }
    }

    // 6. Openings (Doors with swing arc & Windows with glass casing)
    for op in &project.openings {
        if op.level != project.active_level || !op.status.is_visible_in(opts.phase) {
            continue;
        }
        let Some(w) = project.walls.iter().find(|w| w.id == op.wall_id) else {
            continue;
        };
        if !w.status.is_visible_in(opts.phase) {
            continue;
        }

        let (wx, wy) = w.point_at(op.offset_t);
        let (ux, uy) = w.direction();
        let (nx, ny) = w.normal();
        let half_w = op.width * 0.5;
        let (sx1, sy1) = cam.world_to_screen(wx - ux * half_w, wy - uy * half_w, width, height);
        let (sx2, sy2) = cam.world_to_screen(wx + ux * half_w, wy + uy * half_w, width, height);
        let px_thick = (w.thickness * cam.zoom).max(5.0) + 3.0;

        // Cut out wall opening with background color
        buf.draw_thick_line(sx1, sy1, sx2, sy2, px_thick, bg, false);

        let is_sel = opts.selected == Some(SelectedEntity::Opening(op.id));
        let op_col = if is_sel {
            Color::rgb(245, 158, 11)
        } else {
            match op.status {
                RenoStatus::Demolish => Color::rgb(239, 68, 68),
                RenoStatus::New => {
                    if opts.dark_mode {
                        Color::rgb(52, 211, 153)
                    } else {
                        Color::rgb(5, 150, 105)
                    }
                }
                RenoStatus::Existing => Color::rgb(56, 189, 248),
            }
        };

        if op.kind.is_door() {
            let sign = if op.flip_swing { -1.0 } else { 1.0 };
            let r_px = op.width * cam.zoom;
            let leaf_x = sx1 + nx * sign * r_px;
            let leaf_y = sy1 + ny * sign * r_px;
            // Door leaf
            buf.draw_thick_line(sx1, sy1, leaf_x, leaf_y, 2.2, op_col, false);
            // Swing arc
            let a0 = (sy2 - sy1).atan2(sx2 - sx1);
            let a1 = (leaf_y - sy1).atan2(leaf_x - sx1);
            let (start, end) = if (a1 - a0).abs() <= std::f32::consts::PI {
                (a0.min(a1), a0.max(a1))
            } else {
                (a0, a0 + sign * std::f32::consts::FRAC_PI_2)
            };
            buf.draw_arc(sx1, sy1, r_px, start, end, 1.2, op_col.with_alpha(180));
        } else {
            // Double glass line for window
            buf.draw_thick_line(
                sx1 + nx * 2.5,
                sy1 + ny * 2.5,
                sx2 + nx * 2.5,
                sy2 + ny * 2.5,
                1.8,
                op_col,
                false,
            );
            buf.draw_thick_line(
                sx1 - nx * 2.5,
                sy1 - ny * 2.5,
                sx2 - nx * 2.5,
                sy2 - ny * 2.5,
                1.8,
                op_col,
                false,
            );
        }
    }

    // 7. Room Labels & Loi Carrez Surface Badges (drawn on top for readability)
    for r in &project.rooms {
        if r.level != project.active_level || !r.status.is_visible_in(opts.phase) {
            continue;
        }
        let (cx, cy) = cam.world_to_screen(r.x + r.width * 0.5, r.y + r.depth * 0.5, width, height);
        let badge_bg = if opts.dark_mode {
            Color::rgba(15, 23, 42, 235)
        } else {
            Color::rgba(255, 255, 255, 245)
        };
        let title_col = if opts.dark_mode {
            Color::rgb(248, 250, 252)
        } else {
            Color::rgb(15, 23, 42)
        };
        let area_col = if r.status == RenoStatus::New {
            if opts.dark_mode {
                Color::rgb(16, 185, 129)
            } else {
                Color::rgb(5, 150, 105)
            }
        } else if opts.dark_mode {
            Color::rgb(56, 189, 248)
        } else {
            Color::rgb(2, 132, 199)
        };

        let short_name: String = r.name(opts.lang).chars().take(18).collect();
        buf.draw_badge_text(
            cx as i32,
            cy as i32 - 12,
            &short_name,
            1,
            title_col,
            badge_bg,
        );
        let area_str = format!("{:.2}x{:.2}m ({:.1} m2)", r.width, r.depth, r.area_m2());
        buf.draw_badge_text(cx as i32, cy as i32 + 8, &area_str, 1, area_col, badge_bg);

        // Draw 8 resize handles (4 corners + 4 wall midpoints) ON TOP of walls when room is selected!
        if opts.selected == Some(SelectedEntity::Room(r.id)) {
            let (sx0, sy0) = cam.world_to_screen(r.x, r.y, width, height);
            let (sx1, sy1) = cam.world_to_screen(r.x + r.width, r.y + r.depth, width, height);
            let rx0 = sx0.min(sx1);
            let ry0 = sy0.min(sy1);
            let rx1 = sx0.max(sx1);
            let ry1 = sy0.max(sy1);
            let rmx = (rx0 + rx1) * 0.5;
            let rmy = (ry0 + ry1) * 0.5;
            buf.stroke_rect(
                rx0 as i32,
                ry0 as i32,
                (rx1 - rx0) as i32,
                (ry1 - ry0) as i32,
                3,
                Color::rgb(249, 115, 22),
            );
            for &(hx, hy) in &[
                (rx0, ry0),
                (rx1, ry0),
                (rx1, ry1),
                (rx0, ry1),
                (rmx, ry0),
                (rx1, rmy),
                (rmx, ry1),
                (rx0, rmy),
            ] {
                buf.fill_circle(hx, hy, 8.5, Color::rgb(234, 88, 12));
                buf.fill_circle(hx, hy, 4.8, Color::rgb(255, 255, 255));
            }
        }
    }

    // 7b. Empty floor plan onboarding banner right on the 2D canvas
    let has_any_items_on_level = project
        .rooms
        .iter()
        .any(|r| r.level == project.active_level)
        || project
            .walls
            .iter()
            .any(|w| w.level == project.active_level);

    let has_visible_items_on_level = project
        .rooms
        .iter()
        .any(|r| r.level == project.active_level && r.status.is_visible_in(opts.phase))
        || project
            .walls
            .iter()
            .any(|w| w.level == project.active_level && w.status.is_visible_in(opts.phase));

    if !has_visible_items_on_level && opts.draft_line.is_none() {
        let cx = (width / 2) as i32;
        let cy = (height / 2) as i32;
        let bg = if opts.dark_mode {
            Color::rgba(15, 23, 42, 235)
        } else {
            Color::rgba(255, 255, 255, 245)
        };
        if !has_any_items_on_level {
            buf.draw_badge_text(
                cx,
                cy - 14,
                "CLIQUEZ SUR UNE PIECE A GAUCHE POUR L'AJOUTER EN 1 CLIC",
                1,
                Color::rgb(3, 105, 161),
                bg,
            );
            buf.draw_badge_text(
                cx,
                cy + 12,
                "OU GLISSEZ LA SOURIS ICI POUR DESSINER UNE PIECE SUR MESURE",
                1,
                if opts.dark_mode {
                    Color::rgb(52, 211, 153)
                } else {
                    Color::rgb(5, 150, 105)
                },
                bg,
            );
        } else {
            buf.draw_badge_text(
                cx,
                cy - 12,
                "AUCUN ÉLÉMENT VISIBLE DANS CETTE PHASE (AVANT / APRÈS)",
                1,
                Color::rgb(245, 158, 11),
                bg,
            );
            buf.draw_badge_text(
                cx,
                cy + 12,
                "CLIQUEZ SUR LE BOUTON 'CHANTIER' EN HAUT POUR TOUT VOIR",
                1,
                if opts.dark_mode {
                    Color::rgb(52, 211, 153)
                } else {
                    Color::rgb(5, 150, 105)
                },
                bg,
            );
        }
    }

    // 8. Interactive Draft Wall / Room Preview
    if let Some((wx1, wy1, wx2, wy2, status)) = opts.draft_line {
        let (sx1, sy1) = cam.world_to_screen(wx1, wy1, width, height);
        let (sx2, sy2) = cam.world_to_screen(wx2, wy2, width, height);
        let col = match status {
            RenoStatus::Demolish => Color::rgb(220, 38, 38),
            RenoStatus::New => Color::rgb(5, 150, 105),
            RenoStatus::Existing => Color::rgb(217, 119, 6),
        };
        let dx_m = (wx2 - wx1).abs();
        let dy_m = (wy2 - wy1).abs();
        if dx_m >= 0.45 && dy_m >= 0.45 {
            let rx = sx1.min(sx2) as i32;
            let ry = sy1.min(sy2) as i32;
            let rw = (sx2 - sx1).abs() as i32;
            let rh = (sy2 - sy1).abs() as i32;
            buf.fill_rect(rx, ry, rw, rh, col.with_alpha(45));
            buf.stroke_rect(rx, ry, rw, rh, 2, col);
        }
        buf.draw_thick_line(sx1, sy1, sx2, sy2, 5.0, col, true);
        buf.fill_circle(sx1, sy1, 5.5, col);
        buf.fill_circle(sx2, sy2, 5.5, col);
        let len = ((wx2 - wx1).powi(2) + (wy2 - wy1).powi(2)).sqrt();
        let area = dx_m * dy_m;
        let label = if dx_m >= 0.5 && dy_m >= 0.5 {
            format!("{dx_m:.1}m x {dy_m:.1}m ({area:.1} m2)")
        } else {
            format!("{len:.2} m")
        };
        buf.draw_badge_text(
            ((sx1 + sx2) * 0.5) as i32,
            ((sy1 + sy2) * 0.5 - 18.0) as i32,
            &label,
            1,
            Color::rgb(255, 255, 255),
            Color::rgb(15, 23, 42),
        );
    }

    // 9. Interactive Measure Ruler
    if let Some((mx1, my1, mx2, my2)) = opts.measure_line {
        let (sx1, sy1) = cam.world_to_screen(mx1, my1, width, height);
        let (sx2, sy2) = cam.world_to_screen(mx2, my2, width, height);
        let col = Color::rgb(245, 158, 11);
        buf.draw_thick_line(sx1, sy1, sx2, sy2, 2.5, col, true);
        buf.fill_circle(sx1, sy1, 4.5, col);
        buf.fill_circle(sx2, sy2, 4.5, col);
        let len = ((mx2 - mx1).powi(2) + (my2 - my1).powi(2)).sqrt();
        let label = format!("MESURE: {len:.2} m");
        buf.draw_badge_text(
            ((sx1 + sx2) * 0.5) as i32,
            ((sy1 + sy2) * 0.5 - 16.0) as i32,
            &label,
            1,
            Color::rgb(15, 23, 42),
            col,
        );
    }

    // 10. Snap Crosshair Indicator
    if let Some((hx, hy)) = opts.hover_world {
        let (sx, sy) = cam.world_to_screen(hx, hy, width, height);
        let c = Color::rgba(245, 158, 11, 190);
        buf.fill_rect(sx as i32 - 6, sy as i32, 13, 1, c);
        buf.fill_rect(sx as i32, sy as i32 - 6, 1, 13, c);
    }

    // 11. Scale Bar & Compass Rose in bottom-left of 2D canvas
    let bar_m = if cam.zoom > 80.0 { 1.0 } else { 2.0 };
    let bar_px = (bar_m * cam.zoom) as i32;
    let bx = 18;
    let by = height as i32 - 22;
    let fg = if opts.dark_mode {
        Color::rgb(203, 213, 225)
    } else {
        Color::rgb(51, 65, 85)
    };
    buf.fill_rect(bx, by, bar_px, 3, fg);
    buf.fill_rect(bx, by - 5, 2, 8, fg);
    buf.fill_rect(bx + bar_px - 2, by - 5, 2, 8, fg);
    buf.draw_text(bx + 4, by - 12, &format!("{bar_m:.0} m"), 1, fg);

    buf
}
