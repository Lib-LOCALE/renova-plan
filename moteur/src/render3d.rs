use crate::canvas::{CanvasBuffer, Color};
use crate::model::{Lang, OpeningKind, RenoStatus, SelectedEntity, ViewPhase};
use crate::project::Project;

#[derive(Clone, Debug)]
pub struct Camera3D {
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub distance: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub target_z: f32,
    pub wall_cut_ratio: f32, // 0.25 (low cutaway) to 1.0 (full wall height)
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            yaw_deg: -38.0,
            pitch_deg: 36.0,
            distance: 14.5,
            target_x: 5.5,
            target_y: 4.0,
            target_z: 0.8,
            wall_cut_ratio: 0.82,
        }
    }
}

impl Camera3D {
    pub fn preset_iso(&mut self) {
        self.yaw_deg = -38.0;
        self.pitch_deg = 36.0;
        self.distance = 14.5;
        self.target_x = 5.5;
        self.target_y = 4.0;
        self.target_z = 0.8;
    }

    pub fn preset_top(&mut self) {
        self.yaw_deg = 0.0;
        self.pitch_deg = 78.0;
        self.distance = 15.0;
        self.target_x = 5.5;
        self.target_y = 4.0;
        self.target_z = 0.5;
    }

    pub fn preset_south(&mut self) {
        self.yaw_deg = 12.0;
        self.pitch_deg = 22.0;
        self.distance = 13.0;
        self.target_x = 5.5;
        self.target_y = 4.0;
        self.target_z = 1.0;
    }

    pub fn preset_interior(&mut self) {
        self.yaw_deg = -125.0;
        self.pitch_deg = 20.0;
        self.distance = 8.2;
        self.target_x = 4.8;
        self.target_y = 3.5;
        self.target_z = 1.1;
    }

    #[allow(clippy::type_complexity)]
    fn basis(
        &self,
    ) -> (
        (f32, f32, f32),
        (f32, f32, f32),
        (f32, f32, f32),
        (f32, f32, f32),
    ) {
        // World coordinate system: X right (East), Y down-plan (South), Z up (Height)
        let yaw = self.yaw_deg.to_radians();
        let pitch = self.pitch_deg.clamp(8.0, 84.0).to_radians();

        let cam_x = self.target_x + self.distance * pitch.cos() * yaw.sin();
        let cam_y = self.target_y + self.distance * pitch.cos() * yaw.cos();
        let cam_z = self.target_z + self.distance * pitch.sin();

        let fx = self.target_x - cam_x;
        let fy = self.target_y - cam_y;
        let fz = self.target_z - cam_z;
        let flen = (fx * fx + fy * fy + fz * fz).sqrt().max(1e-5);
        let fwd = (fx / flen, fy / flen, fz / flen);

        // Right = normalize(fwd x world_up(0,0,1))
        let rx = fwd.1;
        let ry = -fwd.0;
        let rlen = (rx * rx + ry * ry).sqrt().max(1e-5);
        let right = (rx / rlen, ry / rlen, 0.0);

        // Up = right x fwd
        let up = (
            right.1 * fwd.2 - right.2 * fwd.1,
            right.2 * fwd.0 - right.0 * fwd.2,
            right.0 * fwd.1 - right.1 * fwd.0,
        );

        ((cam_x, cam_y, cam_z), right, up, fwd)
    }

    pub fn project(
        &self,
        wx: f32,
        wy: f32,
        wz: f32,
        width: u32,
        height: u32,
    ) -> Option<(f32, f32, f32)> {
        let (eye, right, up, fwd) = self.basis();
        let dx = wx - eye.0;
        let dy = wy - eye.1;
        let dz = wz - eye.2;

        let cx = dx * right.0 + dy * right.1 + dz * right.2;
        let cy = dx * up.0 + dy * up.1 + dz * up.2;
        let cz = dx * fwd.0 + dy * fwd.1 + dz * fwd.2;

        if cz <= 0.35 {
            return None;
        }

        let fov_scale = (height as f32) * 0.92;
        let sx = (width as f32) * 0.5 + (cx / cz) * fov_scale;
        let sy = (height as f32) * 0.5 - (cy / cz) * fov_scale;
        Some((sx, sy, cz))
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_scene_3d(
    project: &Project,
    cam: &Camera3D,
    width: u32,
    height: u32,
    dark_mode: bool,
    phase: ViewPhase,
    lang: Lang,
    selected: Option<SelectedEntity>,
) -> CanvasBuffer {
    let mut buf = CanvasBuffer::new(width, height, Color::rgb(18, 22, 30));

    // 1. Sky & Horizon Gradient
    for y in 0..height {
        let t = y as f32 / (height.max(1) as f32);
        let c = if dark_mode {
            Color::rgb(
                (15.0 + t * 18.0) as u8,
                (19.0 + t * 22.0) as u8,
                (28.0 + t * 28.0) as u8,
            )
        } else {
            Color::rgb(
                (218.0 + t * 24.0) as u8,
                (230.0 + t * 18.0) as u8,
                (244.0 + t * 8.0) as u8,
            )
        };
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            buf.pixels[idx] = c.r;
            buf.pixels[idx + 1] = c.g;
            buf.pixels[idx + 2] = c.b;
            buf.pixels[idx + 3] = 255;
        }
    }

    // 2. Ground reference grid plane (z = -0.02)
    let grid_col = if dark_mode {
        Color::rgba(71, 85, 105, 95)
    } else {
        Color::rgba(148, 163, 184, 120)
    };
    for i in -2..=14 {
        let gx = i as f32;
        draw_line_3d(
            &mut buf,
            cam,
            (gx, -2.0, -0.02),
            (gx, 11.0, -0.02),
            1.0,
            grid_col,
        );
    }
    for j in -2..=11 {
        let gy = j as f32;
        draw_line_3d(
            &mut buf,
            cam,
            (-2.0, gy, -0.02),
            (14.0, gy, -0.02),
            1.0,
            grid_col,
        );
    }

    let z_base = (project.active_level as f32) * 0.0; // Active level rendered at ground reference

    // 3. Room Floor Slabs
    for r in &project.rooms {
        if r.level != project.active_level || !r.status.is_visible_in(phase) {
            continue;
        }
        let (mr, mg, mb) = r.material.rgb();
        let base_c = if selected == Some(SelectedEntity::Room(r.id)) {
            Color::rgb(245, 188, 66)
        } else {
            Color::rgb(mr, mg, mb)
        };
        draw_box_3d(
            &mut buf,
            cam,
            r.x + r.width * 0.5,
            r.y + r.depth * 0.5,
            z_base - 0.04,
            r.width,
            r.depth,
            0.08,
            0.0,
            base_c,
        );
    }

    // 4. Furniture 3D Volumes (Multi-part realistic architectural blocks)
    for item in &project.furniture {
        if item.level != project.active_level || !item.status.is_visible_in(phase) {
            continue;
        }
        let (fr, fg, fb) = item.kind.color_rgb();
        let is_sel = selected == Some(SelectedEntity::Furniture(item.id));
        let col = if is_sel {
            Color::rgb(245, 158, 11)
        } else {
            match item.status {
                RenoStatus::Demolish => Color::rgba(239, 68, 68, 185),
                RenoStatus::New => Color::rgb(fr, fg, fb),
                RenoStatus::Existing => Color::rgb(fr, fg, fb).scale(0.92),
            }
        };

        // Soft drop shadow on the floor
        draw_box_3d(
            &mut buf,
            cam,
            item.x + 0.05,
            item.y + 0.05,
            z_base + 0.005,
            item.width * 0.98,
            item.depth * 0.98,
            0.01,
            item.rotation_deg,
            Color::rgba(15, 23, 42, 95),
        );

        // Main volume
        draw_box_3d(
            &mut buf,
            cam,
            item.x,
            item.y,
            z_base + item.height * 0.5,
            item.width,
            item.depth,
            item.height,
            item.rotation_deg,
            col,
        );
    }

    // 5. Walls with real 3D door & window cutouts (sills, lintels, jambs, glass panes)
    let cut_factor = cam.wall_cut_ratio.clamp(0.20, 1.0);
    for w in &project.walls {
        if w.level != project.active_level || !w.status.is_visible_in(phase) {
            continue;
        }
        let is_sel = selected == Some(SelectedEntity::Wall(w.id));
        let wall_col = if is_sel {
            Color::rgb(245, 158, 11)
        } else {
            match w.status {
                RenoStatus::Existing => {
                    if dark_mode {
                        Color::rgb(203, 213, 225)
                    } else {
                        Color::rgb(226, 232, 240)
                    }
                }
                RenoStatus::Demolish => Color::rgba(239, 68, 68, 195),
                RenoStatus::New => Color::rgb(16, 185, 129),
            }
        };

        let wall_h = w.height * cut_factor;
        let len = w.length();
        if len < 0.05 {
            continue;
        }
        let angle = w.angle_deg();

        // Collect openings on this wall sorted along t
        let mut ops: Vec<_> = project
            .openings
            .iter()
            .filter(|o| o.wall_id == w.id && o.status.is_visible_in(phase))
            .collect();
        ops.sort_by(|a, b| a.offset_t.partial_cmp(&b.offset_t).unwrap());

        if ops.is_empty() {
            let (mx, my) = w.midpoint();
            draw_box_3d(
                &mut buf,
                cam,
                mx,
                my,
                z_base + wall_h * 0.5,
                len,
                w.thickness,
                wall_h,
                angle,
                wall_col,
            );
        } else {
            let mut cursor_t = 0.0_f32;
            for op in ops {
                let half_t = (op.width / len) * 0.5;
                let t0 = (op.offset_t - half_t).clamp(cursor_t, 1.0);
                let t1 = (op.offset_t + half_t).clamp(t0, 1.0);

                // Solid wall segment before opening
                if t0 > cursor_t + 0.01 {
                    let seg_len = (t0 - cursor_t) * len;
                    let (sx, sy) = w.point_at((cursor_t + t0) * 0.5);
                    draw_box_3d(
                        &mut buf,
                        cam,
                        sx,
                        sy,
                        z_base + wall_h * 0.5,
                        seg_len,
                        w.thickness,
                        wall_h,
                        angle,
                        wall_col,
                    );
                }

                // Sill below window (Allège)
                let (ox, oy) = w.point_at((t0 + t1) * 0.5);
                let op_w = (t1 - t0) * len;
                let sill_h = op.sill_height.min(wall_h);
                if sill_h > 0.05 {
                    draw_box_3d(
                        &mut buf,
                        cam,
                        ox,
                        oy,
                        z_base + sill_h * 0.5,
                        op_w,
                        w.thickness,
                        sill_h,
                        angle,
                        wall_col,
                    );
                }

                // Lintel above opening (Linteau)
                let top_open = (op.sill_height + op.height).min(wall_h);
                if wall_h > top_open + 0.05 {
                    let lintel_h = wall_h - top_open;
                    draw_box_3d(
                        &mut buf,
                        cam,
                        ox,
                        oy,
                        z_base + top_open + lintel_h * 0.5,
                        op_w,
                        w.thickness,
                        lintel_h,
                        angle,
                        wall_col,
                    );
                }

                // Translucent window glass or door frame
                let glass_h = (top_open - sill_h).max(0.0);
                if glass_h > 0.05 && op.kind != OpeningKind::OpenPassage {
                    let glass_col = if op.kind.is_door() {
                        Color::rgba(180, 142, 104, 175)
                    } else {
                        Color::rgba(56, 189, 248, 115)
                    };
                    draw_box_3d(
                        &mut buf,
                        cam,
                        ox,
                        oy,
                        z_base + sill_h + glass_h * 0.5,
                        op_w * 0.94,
                        w.thickness * 0.32,
                        glass_h,
                        angle,
                        glass_col,
                    );
                }

                cursor_t = t1;
            }

            // Remaining wall segment after last opening
            if cursor_t < 0.99 {
                let seg_len = (1.0 - cursor_t) * len;
                let (sx, sy) = w.point_at((cursor_t + 1.0) * 0.5);
                draw_box_3d(
                    &mut buf,
                    cam,
                    sx,
                    sy,
                    z_base + wall_h * 0.5,
                    seg_len,
                    w.thickness,
                    wall_h,
                    angle,
                    wall_col,
                );
            }
        }
    }

    // 6. 3D Room Floating Tags
    for r in &project.rooms {
        if r.level != project.active_level || !r.status.is_visible_in(phase) {
            continue;
        }
        if let Some((sx, sy, _)) = cam.project(
            r.x + r.width * 0.5,
            r.y + r.depth * 0.5,
            z_base + 0.15,
            width,
            height,
        ) {
            let tag = format!("{} ({:.1}m2)", r.name(lang), r.area_m2());
            buf.draw_badge_text(
                sx as i32,
                sy as i32,
                &tag,
                1,
                Color::rgb(248, 250, 252),
                Color::rgba(15, 23, 42, 200),
            );
        }
    }

    // 7. 3D Orientation HUD Overlay
    let hud_text = match lang {
        Lang::Fr => format!(
            "3D TEMPS REEL | Azimut {:.0} deg | Coupe murs {:.0}%",
            cam.yaw_deg,
            cam.wall_cut_ratio * 100.0
        ),
        Lang::En => format!(
            "LIVE 3D | Yaw {:.0} deg | Wall Cut {:.0}%",
            cam.yaw_deg,
            cam.wall_cut_ratio * 100.0
        ),
    };
    buf.fill_rect(
        12,
        height as i32 - 28,
        330,
        18,
        Color::rgba(15, 23, 42, 195),
    );
    buf.draw_text(
        18,
        height as i32 - 22,
        &hud_text,
        1,
        Color::rgb(56, 189, 248),
    );

    buf
}

fn draw_line_3d(
    buf: &mut CanvasBuffer,
    cam: &Camera3D,
    a: (f32, f32, f32),
    b: (f32, f32, f32),
    thickness: f32,
    c: Color,
) {
    if let (Some(pa), Some(pb)) = (
        cam.project(a.0, a.1, a.2, buf.width, buf.height),
        cam.project(b.0, b.1, b.2, buf.width, buf.height),
    ) {
        buf.draw_thick_line(pa.0, pa.1, pb.0, pb.1, thickness, c, false);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_box_3d(
    buf: &mut CanvasBuffer,
    cam: &Camera3D,
    cx: f32,
    cy: f32,
    cz: f32,
    w: f32,
    d: f32,
    h: f32,
    rot_deg: f32,
    base_col: Color,
) {
    let hw = w * 0.5;
    let hd = d * 0.5;
    let hh = h * 0.5;
    let rad = rot_deg.to_radians();
    let cos_a = rad.cos();
    let sin_a = rad.sin();

    let v = |lx: f32, ly: f32, lz: f32| -> Option<(f32, f32, f32)> {
        let wx = cx + lx * cos_a - ly * sin_a;
        let wy = cy + lx * sin_a + ly * cos_a;
        let wz = cz + lz;
        cam.project(wx, wy, wz, buf.width, buf.height)
    };

    // 8 vertices of the box
    let (Some(b0), Some(b1), Some(b2), Some(b3), Some(t0), Some(t1), Some(t2), Some(t3)) = (
        v(-hw, -hd, -hh),
        v(hw, -hd, -hh),
        v(hw, hd, -hh),
        v(-hw, hd, -hh),
        v(-hw, -hd, hh),
        v(hw, -hd, hh),
        v(hw, hd, hh),
        v(-hw, hd, hh),
    ) else {
        return;
    };

    // Directional solar shading per face
    let top_col = base_col.scale(1.12);
    let south_col = base_col.scale(0.95);
    let north_col = base_col.scale(0.76);
    let east_col = base_col.scale(0.86);
    let west_col = base_col.scale(0.68);

    // Side faces
    buf.fill_quad_3d(b0, b1, t1, t0, north_col);
    buf.fill_quad_3d(b1, b2, t2, t1, east_col);
    buf.fill_quad_3d(b2, b3, t3, t2, south_col);
    buf.fill_quad_3d(b3, b0, t0, t3, west_col);
    // Top cap
    buf.fill_quad_3d(t0, t1, t2, t3, top_col);
}
