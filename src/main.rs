//! Fediiverse bridge, for
//! - installing the OLV/miiverse applet patch file
//! - scanning a QR code for the ouath token
//! - saving the token into OLV's "local storage"

#![feature(panic_can_unwind)]
#![feature(panic_payload_as_str)]
pub mod panic_hook;

use cabbage::prelude::*;
use citro2d::prelude::*;
use ctru::{
    prelude::*,
    services::{cam::Cam, gfx::TopScreen},
};
use fruit::prelude::*;

pub mod qr;
pub mod ui;

pub struct AppState<'a> {
    pub scanner: qr::scanner::Scanner,
    pub camera: qr::camera::CameraState<'a>,
    pub token: Option<String>,
}
impl<'a> AppState<'a> {
    fn new(cam: &'a mut Cam) -> anyhow::Result<Self> {
        let scanner = qr::scanner::Scanner::default();
        let camera = qr::camera::CameraState::new(cam)?;
        Ok(Self {
            camera,
            scanner,
            token: None,
        })
    }
}

impl StateImpl<AppState<'_>> for State<'_, AppState<'_>> {
    fn main(&mut self) -> Result<(), Error> {
        let mut c3d_instance = citro3d::Instance::new().unwrap();

        self.handles.apt.set_sleep_allowed(true);
        self.handles.apt.set_home_allowed(true);
        let cfgu = ctru::services::cfgu::Cfgu::new().unwrap();

        let _console = Console::new(self.handles.gfx.bottom_screen.borrow_mut());

        let c2d_instance = citro2d::Instance::new(&mut c3d_instance, &cfgu)?;
        let mut target = citro2d::render::Target::<TopScreen, _>::new(
            self.handles.gfx.top_screen.borrow_mut(),
            &c2d_instance,
        );

        let mut ui = ui::scan::QrScan::default();

        #[allow(unused_variables)]
        let (mut keys_held, mut keys_down, mut keys_up);
        #[allow(unused_assignments)]
        while {
            self.handles.hid.scan_input();
            keys_held = self.handles.hid.keys_held();
            keys_down = self.handles.hid.keys_down();
            keys_up = self.handles.hid.keys_up();
            !keys_held.contains(KeyPad::START) && self.handles.apt.main_loop()
        } {
            {
                let mut draw_target = target.begin();

                draw_target.clear((0xff, 0xff, 0xff, 0xff));
                ui.update_state(&self.handles, &mut self.app)?;
                ui.blit_mut()?;

                target = draw_target.end();
            }
            std::thread::yield_now();
            self.handles.gfx.wait_for_vblank();
        }
        Ok(())
    }
}

fn main() {
    std::panic::set_hook(Box::new(panic_hook::panic_hook));

    let mut apt = Apt::new().expect("failed to obtain applet service handle");
    let mut hid = Hid::new().expect("failed to obtain HID handle");
    let mut gfx = Gfx::new().expect("failed to obtain GFX handle");
    let handles = Handles {
        apt: &mut apt,
        hid: &mut hid,
        gfx: &mut gfx,
    };

    let mut cam = Cam::new().expect("failed to obtain Cam service");
    let app_state = AppState::new(&mut cam).unwrap();
    let mut state = State::new(app_state, handles);

    if let Err(e) = state.main() {
        let _ = state.handle_error(e);
    }
}
