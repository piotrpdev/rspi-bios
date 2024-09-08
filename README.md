<!-- markdownlint-configure-file {
  "MD033": false,
  "MD041": false
} -->
<div align="center">

# RSPI BIOS

[Raspberry Pi][raspberry] dashboard that mimics the style of old [BIOS][bios] designs.

<img alt="demo gif" width="480" src="./.github/img/rspi-bios.gif" />

</div>

## Features

- [x] Served by a Rust-powered web server
  - *...using the [axum] crate*
- [x] Displays real system data from the Raspberry Pi
  - *...using the [sysinfo] crate*
- [x] Implements [Server-Sent Events (SSE)][sse] for real-time updates
  - *...using the axum and [tokio] crates*
- [x] Uses templating for HTML generation
  - *...using the [askama] crate*
- [x] Mimics old [BIOS](bios) designs
  - *...using the [TuiCss][tuicss] package*
- [x] Served over HTTPS
  - *...preferably using [Cloudflare Tunnels][tunnel] and an [Origin CA cert][origin]*

## Usage

> [!NOTE]
> The package's [`postinst`][postinst] script allows `rspi-bios` to bind to low ports.

```bash
# Download package from releases
wget https://github.com/piotrpdev/rspi-bios/releases/latest/download/rspi-bios_1.0.0-1_arm64.deb

# Install package
sudo dpkg -i rspi-bios_1.0.0-1_arm64.deb

# Copy certs (you need to provide your own)
sudo cp ~/rspi-bios/certs/{cert,key}.pem /etc/rspi-bios/certs/

# Run and set to launch on boot
sudo systemctl enable --now rspi-bios
```

## Development

### Cross-compile for Raspberry Pi

#### Build

```bash
# Install dependencies (Ubuntu 24.04 LTS)
# You may need some or all of these
# See https://aws.github.io/aws-lc-rs/requirements/linux.html
sudo apt-get install -y clang libclang1 cmake gcc-aarch64-linux-gnu
# (32-bit ARMv7) sudo apt-get install -y gcc-arm-linux-gnueabihf
cargo install --force --locked bindgen-cli

# Add target (linker already configured for you in ./.cargo/config.toml)
rustup target add aarch64-unknown-linux-gnu
# (32-bit ARMv7) rustup target add armv7-unknown-linux-gnueabihf

# Build
cargo build --release --target=aarch64-unknown-linux-gnu
# (32-bit ARMv7) cargo build --release --target=armv7-unknown-linux-gnueabihf
# Set RUSTFLAGS='-C target-feature=+crt-static' if target uses older version of glibc
```

#### Run

```bash
# Copy
ssh piotrpdev@192.168.0.200 "mkdir -p ~/rspi-bios" && \
scp -r target/armv7-unknown-linux-gnueabihf/release/rspi-bios templates/ certs/ \
  piotrpdev@192.168.0.200:~/rspi-bios/

# Run
ssh -t piotrpdev@192.168.0.200 "chmod +x ~/rspi-bios/rspi-bios && ~/rspi-bios/rspi-bios"
```

#### Bind to low port (optional)

```bash
# (Optional) Allow rspi-bios to bind low ports without root
# https://stackoverflow.com/a/414258/19020549
sudo setcap CAP_NET_BIND_SERVICE=+eip ./rspi-bios

# Run on low port
./rspi-bios 443
```

#### Start on boot (optional)

A template [`rspi-bios.service`][service] file is included in the repo if you're
using `systemd` and want to automatically run the script on startup e.g.:

```bash
# Edit the template
vim ./debian/rspi-bios.service

# Copy it
sudo cp ./debian/rspi-bios.service /lib/systemd/system/

# Start on boot (--now to start now)
sudo systemctl enable rspi-bios.service
```

#### Directories

In production, I recommend creating/using these directories:

- `/usr/local/bin/` (place `rspi-bios` here)
- `/var/log/rspi-bios/`
- `/etc/rspi-bios/certs/` (place `{cert,key}.pem` here)
- `/usr/share/rspi-bios/html/` (place `index.html` here)

### Build `.deb`

You will need [`cargo-deb`][cargo-deb].

```bash
cargo install cargo-deb
cargo deb --locked --target=aarch64-unknown-linux-gnu
# Set RUSTFLAGS='-C target-feature=+crt-static' if target uses older version of glibc
```

### Tools

I recommend installing [`cargo-watch`][cargo-watch] if you plan on making a lot
of changes.

```bash
cargo install cargo-watch
cargo-watch --watch src --watch templates -x run
```

## License

This project is licensed under the [GNU GPL v3.0][license].

Made using the following resources:

| Resource                                  | License                           |
|:-----------------------------------------:|:---------------------------------:|
| [Axum example code][axum-examples]        | [MIT][axum-license]               |
| [TuiCSS "PC Startup" demo][pc-startup]    | [MIT][tuicss-license]             |
| "Award Medallion BIOS" design             | [Copyrighted][phoenix]            |
| [Energy Star logo][energy-star]           | [Trademark][epa]                  |
| [Raspberry Pi logo][raspberry]            | [Trademark][raspberry-foundation] |

[raspberry]: https://www.raspberrypi.org/
[raspberry-foundation]: https://www.raspberrypi.org/about/
[bios]: https://en.wikipedia.org/wiki/BIOS
[axum]: https://github.com/tokio-rs/axum
[sysinfo]: https://github.com/GuillaumeGomez/sysinfo
[sse]: https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events
[tokio]: https://crates.io/crates/tokio
[askama]: https://crates.io/crates/askama
[tuicss]: https://github.com/vinibiavatti1/TuiCss
[tunnel]: https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/
[origin]: https://developers.cloudflare.com/ssl/origin-configuration/origin-ca/
[postinst]: ./debian/postinst
[service]: ./debian/rspi-bios.service
[cargo-deb]: https://github.com/kornelski/cargo-deb
[cargo-watch]: https://github.com/watchexec/cargo-watch
[license]: ./LICENSE
[axum-examples]: https://github.com/tokio-rs/axum/tree/main/examples
[axum-license]: https://github.com/tokio-rs/axum/blob/main/axum/LICENSE
[phoenix]: https://www.phoenix.com/
[pc-startup]: https://github.com/vinibiavatti1/TuiCss/blob/6a021ecc2abb1fbe6da62bd370d1f2a764da1195/examples/pc-startup.html
[tuicss-license]: https://github.com/vinibiavatti1/TuiCss/blob/6a021ecc2abb1fbe6da62bd370d1f2a764da1195/LICENSE.md
[energy-star]: https://www.energystar.gov/
[epa]: https://www.epa.gov/
