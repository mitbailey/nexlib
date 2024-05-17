#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use eframe::egui::Visuals;
use egui_dock::{DockArea, DockState, NodeIndex};
use mount::Mount;
use std::vec;

pub mod mount;
pub use mount::{AzEl, CelestronMount, NonGpsDevice, RADec};

// TODO: Fix issue where the serial port always waits the 3.5 second timeout before returning the buffer, even when something has been read. Perhaps this has to do with the fact that the buffer hasn't been filled to capacity?

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([300.0, 220.0]),
        // .with_icon(
        //     // NOTE: Adding an icon is optional
        //     eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
        //         .expect("Failed to load icon"),
        // ),
        ..Default::default()
    };

    eframe::run_native(
        "NexLib Test Program",
        native_options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<Gui>::default()
        }),
    )
}

struct GuiTabs {
    mount: Option<CelestronMount>,
    connected: bool,

    curr_ra_dec: RADec,
    goto_ra_dec: RADec,
}

struct Gui {
    tree: DockState<String>,
    tabs: GuiTabs,
}

impl egui_dock::TabViewer for GuiTabs {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        // ui.set_enabled(!self.modal_active);

        match tab.as_str() {
            "Device Controls" => self.device_controls(ui),
            "Data Plot" => self.data_plot(ui),
            "Data Log" => self.data_log(ui),
            _ => self.default_tab(ui),
        }
    }
}

impl GuiTabs {
    fn device_controls(&mut self, ui: &mut egui::Ui) {
        // Button
        ui.horizontal(|ui| match self.mount {
            Some(_) => {
                self.connected = true;
                ui.add_enabled(false, egui::Button::new("Connected"));
            }
            None => {
                self.connected = false;
                if ui.button("Connect").clicked() {
                    ui.add(egui::Spinner::new().color(egui::Color32::WHITE));
                    ui.label("Connecting...");

                    self.mount = match CelestronMount::new() {
                        Ok(m) => Some(m),
                        Err(e) => {
                            println!("Error: {:?}", e);
                            None
                        }
                    };
                }
            }
        });

        egui::Grid::new("grid").show(ui, |ui| {
            ui.set_enabled(self.connected);

            // ui.label("Current RA/Dec:");
            // ui.end_row();

            ui.add(egui::Label::new(format!("{}", self.curr_ra_dec.ra)));
            ui.add(egui::Label::new(format!("{}", self.curr_ra_dec.dec)));
            if ui.button("Refresh").clicked() {
                self.curr_ra_dec = self
                    .mount
                    .as_mut()
                    .unwrap()
                    .get_position_ra_dec()
                    .expect("Failed to get position.");
            }
            ui.end_row();

            // ui.label("Go to RA/Dec:");
            // ui.end_row();

            ui.add(egui::DragValue::new(&mut self.goto_ra_dec.ra).speed(0.1));
            ui.add(egui::DragValue::new(&mut self.goto_ra_dec.dec).speed(0.1));
            if ui.button("Go").clicked() {
                self.mount
                    .as_mut()
                    .unwrap()
                    .goto_ra_dec(self.goto_ra_dec) // Pass the cloned value
                    .expect("Failed to goto position.");
            }
            ui.end_row();
        });
    }

    fn data_plot(&mut self, ui: &mut egui::Ui) {
        // Button
        ui.horizontal(|ui| {
            if ui.button("Generate Random Datapoint").clicked() {
                print!("Button pushed.");
            }
        });
    }

    fn data_log(&mut self, ui: &mut egui::Ui) {
        ui.label("This is the data log tab.");
    }

    fn default_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("This is a default tab.");
    }
}

impl Default for Gui {
    fn default() -> Self {
        let mut tree = DockState::new(vec!["Device Controls".to_owned()]);

        // You can modify the tree before constructing the dock
        let [a, _b] = tree.main_surface_mut().split_right(
            NodeIndex::root(),
            0.5,
            vec!["Data Plot".to_owned()],
        );
        let [_, _] = tree
            .main_surface_mut()
            .split_below(a, 0.8, vec!["Data Log".to_owned()]);

        let tabs = GuiTabs {
            mount: None,
            connected: false,
            curr_ra_dec: RADec::new(0.0, 0.0),
            goto_ra_dec: RADec::new(0.0, 0.0),
        };

        Self { tree, tabs }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);
        ctx.set_visuals(Visuals::dark());

        DockArea::new(&mut self.tree)
            // .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.tabs);
    }
}

/// Tests prefixed with `nocon` require exclusive communication access to a mount and cannot be run concurrently. These tests should only be run using `cargo test nocon -- --test-threads=1`. If all tests are to be run, then `cargo test -- --test-threads=1` should be used since some will require exclusive access to the same hardware device.
#[cfg(test)]
mod tests {
    pub use crate::mount::CelestronMount;
    use crate::{
        mount::{Gps, Mount, RADec, Rtc, SlewAxis, SlewDir, TrackingMode}, // + SlewRate ?
        AzEl,
        NonGpsDevice,
    };
    use chrono::Utc;
    use std::{thread::sleep, time::Duration};

    const ERR_MSG_1: &str = "Failed to connect to mount. Are you in WSL?";

    #[test]
    fn nocon_basic_build() {
        let _mount = CelestronMount::new().expect(ERR_MSG_1);
    }

    #[test]
    fn nocon_get_gps_expect() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let _gps = mount.get_gps().expect("Failed to get GPS.");
    }

    #[test]
    #[should_panic]
    fn nocon_get_gps_panic() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let _gps = mount.get_gps().expect("Failed to get GPS.");
    }

    #[test]
    #[should_panic]
    fn nocon_gps_is_linked() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let mut gps = mount.get_gps().expect("Failed to get GPS.");
        gps.is_linked().expect("Failed to get GPS link status.");
    }

    #[test]
    #[should_panic]
    fn nocon_gps_get_location() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let mut gps = mount.get_gps().expect("Failed to get GPS.");
        gps.get_location().expect("Failed to get GPS link status.");
    }

    #[test]
    #[should_panic]
    fn nocon_gps_get_datetime() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let mut gps = mount.get_gps().expect("Failed to get GPS.");
        gps.get_datetime().expect("Failed to get GPS link status.");
    }

    #[test]
    fn nocon_get_ra_dec() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        println!("{}", pos);
    }

    #[test]
    fn nocon_get_az_el() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let _pos = mount.get_position_az_el().expect("Failed to get position.");
    }

    #[test]
    fn nocon_goto_ra_dec() {
        const DX: f64 = 1.0;
        const ACC: f64 = 0.5;

        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        let pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        mount
            .goto_ra_dec(RADec::new(pos.ra + DX, pos.dec + DX))
            .expect("Failed to goto position.");

        while mount
            .goto_in_progress()
            .expect("Failed to get goto in progress.")
        {
            sleep(Duration::from_secs(1));
        }

        // Verify that we are within 1 degree of the target
        let new_pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        assert!(
            (new_pos.ra - (pos.ra + DX)).abs() < ACC,
            "RA: {} -> {}",
            pos.ra,
            new_pos.ra
        );
        assert!(
            (new_pos.dec - (pos.dec + DX)).abs() < ACC,
            "Dec: {} -> {}",
            pos.dec,
            new_pos.dec
        );
    }

    #[test]
    fn nocon_get_goto_in_progress() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let _in_progress = mount
            .goto_in_progress()
            .expect("Failed to get goto in progress.");
    }

    #[test]
    fn nocon_goto_az_el() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let pos = mount.get_position_az_el().expect("Failed to get position.");
        mount
            .goto_az_el(AzEl::new(pos.az + 5., pos.el + 5.))
            .expect("Failed to goto position.");
        while mount
            .goto_in_progress()
            .expect("Failed to get goto in progress.")
        {
            sleep(Duration::from_secs(1));
        }
        // Verify that we are within 1 degree of the target
        let new_pos = mount.get_position_az_el().expect("Failed to get position.");
        assert!((new_pos.az - 5.).abs() < 1.0);
        assert!((new_pos.el - 5.).abs() < 1.0);
    }

    // Sync

    #[test]
    fn nocon_get_tracking_mode() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        let mode = mount.get_tracking_mode().unwrap();

        println!("{:?}", mode as u8);
    }

    // Set tracking mode (verify!)

    // Slew variable (and wait til done!)
    #[test]
    fn nocon_slew_variable_decel() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        let pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        println!("Current position: {}", pos);

        mount
            .slew_variable(SlewAxis::DecEl, SlewDir::Positive, 1800)
            .expect("Failed to slew.");

        for _ in 0..3 {
            println!("Sleeping...");
            sleep(Duration::from_secs(1));
        }

        mount
            .slew_variable(SlewAxis::DecEl, SlewDir::Negative, 1800)
            .expect("Failed to slew.");

        for _ in 0..3 {
            println!("Sleeping...");
            sleep(Duration::from_secs(1));
        }

        mount
            .stop_slew(SlewAxis::RAAz)
            .expect("Failed to stop slew.");

        let pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        println!("Current position: {}", pos);
    }

    #[test]
    fn nocon_slew_variable_raaz() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        let pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        println!("Current position: {}", pos);

        mount
            .slew_variable(SlewAxis::RAAz, SlewDir::Negative, 1800)
            .expect("Failed to slew.");

        for _ in 0..3 {
            println!("Sleeping...");
            sleep(Duration::from_secs(1));
        }

        mount
            .slew_variable(SlewAxis::RAAz, SlewDir::Positive, 1800)
            .expect("Failed to slew.");

        for _ in 0..3 {
            println!("Sleeping...");
            sleep(Duration::from_secs(1));
        }

        mount
            .stop_slew(SlewAxis::RAAz)
            .expect("Failed to stop slew.");

        let pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        println!("Current position: {}", pos);
    }

    // Slew fixed (and wait til done!)

    // Get location...
    // Set location... (verify!)

    // Get time

    // Set time (verify!)

    #[test]
    fn nocon_rtc_get_datetime() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        let datetime = mount.get_datetime();

        match datetime {
            Ok(dt) => {
                println!("{}", dt.format("%Y-%m-%d %H:%M:%S"));
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    #[test]
    fn nocon_rtc_set_datetime_now() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let datetime = Utc::now();
        mount.set_datetime_now().expect("Failed to set datetime.");
        sleep(Duration::from_secs(1));
        println!("Getting... ");
        let ndatetime = mount.get_datetime().expect("Failed to get datetime.");
        println!("Got: {}", ndatetime.format("%Y-%m-%d %H:%M:%S"));
        assert!(ndatetime - datetime > chrono::Duration::seconds(1));
    }

    #[test]
    fn nocon_get_version() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        let version = mount.get_version().unwrap();

        println!("{:?}", version);
    }

    #[test]
    fn nocon_get_device_version() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        // let version = mount.get_device_version(Device::AzRaMotor).unwrap();

        match mount.get_gps() {
            Ok(mut gps) => {
                println!("GPS: {:?}", gps.get_device_version().unwrap());
            }
            Err(e) => {
                println!("No GPS device found: {:?}", e);
            }
        }

        println!(
            "AzRaMotor Version: {:?}",
            mount.get_device_version(NonGpsDevice::AzRaMotor).unwrap()
        );
        println!(
            "ElDecMotor Version: {:?}",
            mount.get_device_version(NonGpsDevice::ElDecMotor).unwrap()
        );
        println!(
            "RtcUnit Version: {:?}",
            mount.get_device_version(NonGpsDevice::RtcUnit).unwrap()
        );
    }

    #[test]
    fn nocon_get_model() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let _model = mount.get_model().unwrap();
    }

    #[test]
    fn nocon_set_tracking_mode_eqsouth() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        mount.set_tracking_mode(TrackingMode::EQNorth).unwrap();
    }

    #[test]
    fn nocon_set_tracking_mode_eqnorth() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        mount.set_tracking_mode(TrackingMode::EQNorth).unwrap();
    }

    // echo

    // is aligned

    // goto_in_progress

    // cancel goto (check if its still moving after a cancellation - measure amount of time it takes to stop?)
}
