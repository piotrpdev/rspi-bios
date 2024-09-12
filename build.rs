//! RSPI-BIOS build script
//! Author: Piotr Placzek (piotrpdev) <https://github.com/piotrpdev>
//! SPDX-License-Identifier: GPL-3.0-only

/// Parses `RSPI_BIOS_VERSION` environment variable and passes it to `main.rs`.
///
/// Used in CI for automatic versioning instead of manually creating tags.
fn main() {
    let version = std::env::var("RSPI_BIOS_VERSION").unwrap_or_else(|_| "dev".to_string());
    println!("cargo:rustc-env=RSPI_BIOS_VERSION={version}");
}
