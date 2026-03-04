use gio::prelude::*;
use glib::SendWeakRef;
use gtk::{gio, glib};
use ksni::TrayMethods;

pub struct HarvuxTray {
    app: SendWeakRef<gio::Application>,
}

impl HarvuxTray {
    pub fn new(app: &impl IsA<gio::Application>) -> Self {
        Self {
            app: app.upcast_ref::<gio::Application>().downgrade().into(),
        }
    }

    /// Spawn the tray icon. Keeps it alive until the future is cancelled.
    pub async fn run(self) {
        match self.disable_dbus_name(true).spawn().await {
            Ok(_handle) => {
                // Keep the handle alive so the tray persists.
                // When the tokio runtime drops (app quit), this task is cancelled.
                std::future::pending::<()>().await;
            }
            Err(e) => eprintln!("Failed to spawn tray icon: {e}"),
        }
    }
}

impl ksni::Tray for HarvuxTray {
    fn id(&self) -> String {
        "com.github.samsonasu.Harvux".into()
    }

    fn icon_name(&self) -> String {
        "com.github.samsonasu.Harvux".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![generate_clock_icon(24)]
    }

    fn title(&self) -> String {
        "Harvux".into()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let app_ref = self.app.clone();
        glib::MainContext::default().invoke(move || {
            if let Some(app) = app_ref.upgrade() {
                app.activate();
            }
        });
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        vec![
            ksni::MenuItem::Standard(ksni::menu::StandardItem {
                label: "Show/Hide".into(),
                activate: Box::new(|tray: &mut Self| {
                    let app_ref = tray.app.clone();
                    glib::MainContext::default().invoke(move || {
                        if let Some(app) = app_ref.upgrade() {
                            app.activate();
                        }
                    });
                }),
                ..Default::default()
            }),
            ksni::MenuItem::Separator,
            ksni::MenuItem::Standard(ksni::menu::StandardItem {
                label: "Quit".into(),
                activate: Box::new(|tray: &mut Self| {
                    let app_ref = tray.app.clone();
                    glib::MainContext::default().invoke(move || {
                        if let Some(app) = app_ref.upgrade() {
                            app.quit();
                        }
                    });
                }),
                ..Default::default()
            }),
        ]
    }
}

/// Generate a simple clock icon as ARGB32 pixmap data.
/// Matches the app's blue-circle-with-white-clock-face design.
fn generate_clock_icon(size: i32) -> ksni::Icon {
    let s = size as f64;
    let center = s / 2.0;
    let outer_r = center - 0.5;
    let border_r = outer_r - 2.0;
    let mut data = vec![0u8; (size * size * 4) as usize];

    // Blue (#3584E4) and dark blue (#1C71D8) from the SVG
    let blue: [u8; 4] = [0xFF, 0x35, 0x84, 0xE4];
    let dark: [u8; 4] = [0xFF, 0x1C, 0x71, 0xD8];
    let white: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

    for y in 0..size {
        for x in 0..size {
            let dx = x as f64 - center + 0.5;
            let dy = y as f64 - center + 0.5;
            let dist = (dx * dx + dy * dy).sqrt();

            let color = if dist > outer_r {
                continue; // transparent
            } else if dist > border_r {
                &blue
            } else {
                &white
            };

            let idx = ((y * size + x) * 4) as usize;
            data[idx..idx + 4].copy_from_slice(color);
        }
    }

    // Helper to plot a thick pixel
    let mut plot = |x: i32, y: i32, color: &[u8; 4]| {
        if x >= 0 && x < size && y >= 0 && y < size {
            let idx = ((y * size + x) * 4) as usize;
            data[idx..idx + 4].copy_from_slice(color);
        }
    };

    let cx = size / 2;
    let cy = size / 2;

    // Hour hand — pointing to ~12 (straight up)
    for i in 2..=6 {
        plot(cx, cy - i, &dark);
    }

    // Minute hand — pointing to ~2 (up-right diagonal)
    for i in 2..=7 {
        let mx = cx + (i * 3 / 4);
        let my = cy - (i / 2);
        plot(mx, my, &dark);
    }

    // Center dot
    plot(cx, cy, &dark);

    ksni::Icon {
        width: size,
        height: size,
        data,
    }
}
