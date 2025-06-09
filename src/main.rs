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
    services::{cam::Cam, gfx::BottomScreen},
};
use fruit::prelude::*;

pub mod qr;
pub mod ui;

#[derive(Default)]
pub struct AppState;

impl StateImpl<AppState> for State<'_, AppState> {
    fn main(&mut self) -> Result<(), Error> {
        let mut c3d_instance = citro3d::Instance::new().unwrap();

        self.handles.apt.set_sleep_allowed(true);
        self.handles.apt.set_home_allowed(true);
        let cfgu = ctru::services::cfgu::Cfgu::new().unwrap();

        let _console = Console::new(self.handles.gfx.top_screen.borrow_mut());

        let c2d_instance = citro2d::Instance::new(&mut c3d_instance, &cfgu)?;
        let mut target = citro2d::render::Target::<BottomScreen, _>::new(
            self.handles.gfx.bottom_screen.borrow_mut(),
            &c2d_instance,
        );

        let mut cam = Cam::new()?;
        let mut scanner = qr::scan::Scanner::default();
        let mut camera = qr::camera::CameraState::new(&mut cam)?;

        let mut ui = ui::background::Background::default();

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
            println!("capturing");
            camera.capture()?;
            println!("converting");
            camera.convert(&mut scanner.image);
            println!("scanning");
            scanner.scan()?;
            println!("scanned!");
            {
                let mut draw_target = target.begin();

                draw_target.clear((0xfb, 0xdb, 0x65, 0xff));
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

    let mut state = State::new(AppState, handles);

    if let Err(e) = state.main() {
        let _ = state.handle_error(e);
    }
}
