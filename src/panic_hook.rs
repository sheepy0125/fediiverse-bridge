//! Just a silly little panic hook!

use std::{panic::PanicHookInfo, sync::Mutex};

pub static PANICKING_LOCK: Mutex<()> = Mutex::new(());
pub fn panic_hook(info: &PanicHookInfo) {
    let _lock = loop {
        let Ok(lock) = PANICKING_LOCK.try_lock() else {
            std::hint::spin_loop();
            continue;
        };
        break lock;
    };

    unsafe { ctru_sys::consoleInit(ctru_sys::GFX_TOP, core::ptr::null_mut()) };

    println!();
    println!("panicked!");
    if let Some(location) = info.location() {
        println!(
            "({can_unwind}) {f} @ line {l}:{c}",
            can_unwind = if info.can_unwind() {
                "can unwind"
            } else {
                "cannot unwind"
            },
            f = location.file(),
            l = location.line(),
            c = location.column()
        );
    }
    if let Some(message) = info.payload_as_str() {
        println!("{message}")
    };
    println!("SELECT to continue");

    while unsafe { ctru_sys::aptMainLoop() } {
        unsafe { ctru_sys::hidScanInput() };
        let keys_down = unsafe { ctru_sys::hidKeysDown() };
        if (keys_down & ctru_sys::KEY_SELECT) != 0 {
            println!("SELECT pressed");
            unsafe { ctru_sys::gfxSwapBuffers() };
            unsafe { ctru_sys::gfxFlushBuffers() };
            break;
        }
    }
}
