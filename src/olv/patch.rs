//! Download patches for OLV

use std::{
    fs::OpenOptions,
    io::{Read, Write as _},
    path::{Path, PathBuf},
};

use anyhow::Context;
use ctru::prelude::*;

fn download_helper(uri: &str, save_to: &Path) -> anyhow::Result<()> {
    let resp = ureq::get(uri).call()?;
    let bytes = resp.into_body().read_to_vec()?;

    std::fs::create_dir_all(save_to.parent().unwrap()).context("making parent directory")?;
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(save_to)
        .context("opening file")?;
    file.write_all(&bytes)?;
    file.flush()?;
    println!("Saved to `{}`!", save_to.display());

    Ok(())
}

pub fn download(host: &str, _soc: &Soc) -> anyhow::Result<()> {
    println!("Downloading patches from {host}...");
    let host = host.strip_prefix("http://").unwrap_or(host);

    println!("Downloading certificate...");
    download_helper(
        &format!("http://{host}/out/ca_cert.pem"),
        &PathBuf::from("sdmc:/3ds/fediiverse.pem"),
    )?;

    println!("Downloading patch IPS...");
    // download_helper(
    //     &format!("http://{host}/em"),
    //     "3ds/fediiverse.pem",
    // )?;

    Ok(())
}
