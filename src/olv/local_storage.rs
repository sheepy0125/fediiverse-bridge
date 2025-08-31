//! OLV local storage patching to save a token

use ctru::prelude::*;

use std::io::{Seek, SeekFrom, Write as _};

const STORAGE_LEN: u64 = 8_433_950;
const KEY_OFFSET: u64 = 0x3000;
const VALUE_OFFSET: u64 = 0xb000;
/// OLV largeStorage.bin offsets, as seen by a fresh OLV install.
/// This is consistent between N3DS and O3DS.
const OFFSETS: [(u64, &[u8]); 9] = [
    (0x0000, b"SMFC"),
    (0x0004, b"\x09"),
    (0x1000, b"\x01"),
    (0x1004, b"\x01"),
    (0x1008, b"\x01"),
    (0x1048, b"\xFF\xFF\xFF\xFF"),
    (0x1148, b"\x01"),
    (0x114c, b"\x01"),
    (0x114d, b"\x08"),
];

#[track_caller]
fn assert_result(result: ctru_sys::Result) {
    if ctru_sys::R_FAILED(result) {
        let err = ctru::Error::from(result);
        panic!("{err:?}");
    }
}

/// Create `largeStorage.bin` for OLV with `key: value` saved in localStorage
pub fn patch_local_storage(key: &str, value: &str) -> anyhow::Result<()> {
    println!("Patching Miiverse (OLV) save...");

    let mut save_info = ctru_sys::FS_SystemSaveDataInfo::default();
    save_info.set_mediaType(ctru::services::fs::MediaType::Nand.into());
    // FIXME
    save_info.saveId = 0x000200BD; // olv (US)
    // save_info.saveId = 0x000200BE; // olv (Europe)

    let save_info_ptr =
        &save_info as *const ctru_sys::FS_SystemSaveDataInfo as *const core::ffi::c_void;

    let fs_path = ctru_sys::FS_Path {
        type_: ctru::services::fs::PathType::Binary.into(),
        size: std::mem::size_of::<ctru_sys::FS_SystemSaveDataInfo>() as u32,
        data: save_info_ptr,
    };
    assert_result(unsafe {
        ctru_sys::archiveMount(
            ctru::services::fs::ArchiveID::SystemSavedata.into(),
            fs_path,
            c"olv".as_ptr(),
        )
    });

    // Even if we truncate the file, holes aren't guaranteed to be NULLed on the 3DS--
    // so, delete the save first
    let _ = std::fs::remove_file("olv:/largeStorage.bin");

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .read(true)
        .open("olv:/largeStorage.bin")?;

    // Most of the file is zeroed out, except for the offsets
    file.set_len(STORAGE_LEN)?;
    for (offset, patch) in OFFSETS {
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(patch)?;
    }

    file.seek(SeekFrom::Start(KEY_OFFSET))?;
    file.write_all(key.as_bytes())?;
    file.seek(SeekFrom::Start(VALUE_OFFSET))?;
    file.write_all(value.as_bytes())?;

    file.flush()?;

    println!("Patched largeStorage.bin!");

    Ok(())
}
