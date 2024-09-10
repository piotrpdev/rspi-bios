fn main() {
    let version = std::env::var("RSPI_BIOS_VERSION").unwrap_or_else(|_| "dev".to_string());
    println!("cargo:rustc-env=RSPI_BIOS_VERSION={version}");
}
