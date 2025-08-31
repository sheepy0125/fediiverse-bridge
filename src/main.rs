//! Fediiverse bridge, for
//! - installing the OLV/miiverse applet patch file
//! - scanning a QR code for the ouath token
//! - saving the token into OLV's "local storage"

#![allow(unused_imports)]

use std::{mem::MaybeUninit, str::FromStr};

use anyhow::anyhow;
use cabbage::prelude::*;
use citro2d::prelude::*;
use ctru::{
    error::ResultCode,
    prelude::*,
    services::{
        cam::Cam,
        fs,
        gfx::{BottomScreen, TopScreen},
    },
};
use fruit::prelude::*;

pub mod olv;
pub mod qr;
pub mod ui;

/// fediiverse;<setup host>;<token>
#[derive(Debug)]
pub struct QrCode {
    pub setup_host: String,
    pub token: String,
}
impl FromStr for QrCode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chunks = s.split(';');
        let Some("fediiverse") = chunks.next() else {
            return Err(anyhow!("expected 'fediiverse' magic"));
        };
        let Some(url) = chunks.next() else {
            return Err(anyhow!("no url"));
        };
        let Some(token) = chunks.next() else {
            return Err(anyhow!("no token"));
        };

        Ok(Self {
            setup_host: url.to_string(),
            token: token.to_string(),
        })
    }
}

pub struct AppState<'a> {
    pub scanner: qr::scanner::Scanner,
    pub camera: qr::camera::CameraState<'a>,
    pub qr: Option<QrCode>,
}
impl<'a> AppState<'a> {
    fn new(cam: &'a mut Cam) -> anyhow::Result<Self> {
        let scanner = qr::scanner::Scanner::default();
        let camera = qr::camera::CameraState::new(cam)?;
        Ok(Self {
            camera,
            scanner,
            qr: None,
        })
    }
}

impl StateImpl<AppState<'_>> for State<'_, AppState<'_>> {
    fn main(&mut self) -> Result<(), Error> {
        let _console = Console::new(self.handles.gfx.bottom_screen.borrow_mut());

        self.handles.apt.set_sleep_allowed(true);
        self.handles.apt.set_home_allowed(true);

        let mut c3d_instance = citro3d::Instance::new().unwrap();
        let cfgu = ctru::services::cfgu::Cfgu::new().unwrap();
        let soc = ctru::services::soc::Soc::new().unwrap();

        let c2d_instance = citro2d::Instance::new(&mut c3d_instance, &cfgu)?;
        let mut top_target = citro2d::render::Target::<TopScreen, _>::new(
            self.handles.gfx.top_screen.borrow_mut(),
            &c2d_instance,
        );

        let mut qr_scan = ui::scan::QrScan::default();
        let mut scanned = false;

        println!("Welcome to Fediiverse setup!");
        println!("Please scan the QR code...");

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
            let mut draw_target = top_target.begin();
            draw_target.clear(0xffffffff);
            if !scanned {
                qr_scan.update_state(&self.handles, &mut self.app)?;
            }
            qr_scan.blit_mut()?;
            top_target = draw_target.end();

            if let Some(qr) = &self.app.qr.take() {
                println!("Scanned!");
                scanned = true;
                olv::patch::download(&qr.setup_host, &soc)?;
                olv::local_storage::patch_local_storage("fediiverse_token", &qr.token)?;
            }

            std::thread::yield_now();
        }
        Ok(())
    }
}

fn main() {
    ctru::set_panic_hook(false);

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
