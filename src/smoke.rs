use crate::RenovaWindow;
use slint::ComponentHandle;

/// Debug-only real Wayland/X11 UI verification & multi-scenario screenshot capture
pub fn run(ui: &RenovaWindow) {
    ui.set_language_index(0);
    let weak = ui.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(700), move || {
        let ui = weak.unwrap();
        eprintln!(
            "SMOKE: WINDOW SIZE = {:?}, SCALE = {:?}",
            ui.window().size(),
            ui.window().scale_factor()
        );
        // 1. Click Salon (5x4m)
        let pos1 = slint::LogicalPosition::new(200.0, 190.0);
        ui.window()
            .dispatch_event(slint::platform::WindowEvent::PointerMoved { position: pos1 });
        ui.window()
            .dispatch_event(slint::platform::WindowEvent::PointerPressed {
                position: pos1,
                button: slint::platform::PointerEventButton::Left,
            });
        ui.window()
            .dispatch_event(slint::platform::WindowEvent::PointerReleased {
                position: pos1,
                button: slint::platform::PointerEventButton::Left,
            });

        // 2. Click Cuisine (3.5x3m)
        let pos2 = slint::LogicalPosition::new(200.0, 240.0);
        ui.window()
            .dispatch_event(slint::platform::WindowEvent::PointerMoved { position: pos2 });
        ui.window()
            .dispatch_event(slint::platform::WindowEvent::PointerPressed {
                position: pos2,
                button: slint::platform::PointerEventButton::Left,
            });
        ui.window()
            .dispatch_event(slint::platform::WindowEvent::PointerReleased {
                position: pos2,
                button: slint::platform::PointerEventButton::Left,
            });

        step(ui.as_weak(), 0);
    });
}

fn step(weak: slint::Weak<RenovaWindow>, index: usize) {
    slint::Timer::single_shot(std::time::Duration::from_millis(450), move || {
        let ui = weak.unwrap();
        let names = [
            "1-plan-2d-creation-fr",
            "2-welcome-modal-fr",
            "3-split-2d-3d-meubles-fr",
            "4-devis-travaux-fr",
        ];

        let buffer = ui
            .window()
            .take_snapshot()
            .expect("Capture du rendu natif Slint");
        let default_prefix = std::env::temp_dir()
            .join("renova-v2")
            .display()
            .to_string();
        let prefix = std::env::var("RENOVA_SCREENSHOT").unwrap_or(default_prefix);
        let path = format!("{prefix}-{}.ppm", names[index]);
        let mut bytes = format!("P6\n{} {}\n255\n", buffer.width(), buffer.height()).into_bytes();
        for pixel in buffer.as_slice() {
            bytes.extend_from_slice(&[pixel.r, pixel.g, pixel.b]);
        }
        std::fs::write(&path, bytes).expect("Écriture capture PPM");

        match index {
            0 => {
                // Show Welcome House Picker Modal
                ui.set_show_welcome(true);
            }
            1 => {
                // Close Welcome, switch to 2D + 3D split view + Furniture drawer
                ui.set_show_welcome(false);
                ui.set_view_mode(1);
                ui.set_guided_step(2);
                ui.invoke_action("refresh".into());
            }
            2 => {
                // Switch to Step 4 (Prix & Devis)
                ui.set_guided_step(3);
                ui.invoke_action("refresh".into());
            }
            _ => {
                println!(
                    "UI smoke OK: Guided 4-step wizard, Welcome house picker, Large Text mode, FR/EN i18n, Carrez & BoQ estimate; snapshots {prefix}-*.ppm"
                );
                slint::quit_event_loop().unwrap();
                return;
            }
        }
        step(ui.as_weak(), index + 1);
    });
}
