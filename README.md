# RSPI BIOS

[Raspberry Pi][raspberry] dashboard that mimics the style of old [BIOS][bios] designs.

## TODO

- [ ] in release, derive paths from path of current exe
  - [ ] see <https://doc.rust-lang.org/stable/std/env/fn.current_exe.html>
  - [ ] maybe let user pass cmd arg with path(s)
- [ ] remove unused features and packages
- [ ] implement HTTPS/TLS
- [ ] create Dockerfile
- [ ] add compression layer
- [ ] add CORS
- [ ] add 404
- [ ] add proper error handling
- [ ] add graceful tls shutdown

## Features

<!-- TODO: Add more features and packages/crates -->

- [x] Served by a Rust-powered web server
  - *...using the [axum][axum] crate*
- [x] Displays real-time system data from the Raspberry Pi
  - *...using the [sysinfo][sysinfo] crate*
- [x] Mimics old [BIOS](bios) designs
  - *...using the [TuiCss][tuicss] package*

## Usage

### Build

```bash
# Install GCC for ARM (Ubuntu 24.04 LTS)
sudo apt-get install gcc-arm-linux-gnueabihf

# Add target
rustup target add armv7-unknown-linux-gnueabihf

# Build
cargo build --release --target=armv7-unknown-linux-gnueabihf
```

### Development

```bash
cargo install cargo-watch
cargo-watch --watch src --watch templates -x run
```

## License

This project is licensed under the [GNU GPL v3.0][license].

Made using the following resources:

| Resource                                  | License                           |
|:-----------------------------------------:|:---------------------------------:|
| [Axum Vite template][axum-vite-template]  | N/A                               |
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
[tuicss]: https://github.com/vinibiavatti1/TuiCss
[license]: ./LICENSE
[axum-examples]: https://github.com/tokio-rs/axum/tree/main/examples
[axum-license]: https://github.com/tokio-rs/axum/blob/main/axum/LICENSE
[axum-vite-template]: https://github.com/varonroy/template-axum-htmx-vite-tailwind
[phoenix]: https://www.phoenix.com/
[pc-startup]: https://github.com/vinibiavatti1/TuiCss/blob/6a021ecc2abb1fbe6da62bd370d1f2a764da1195/examples/pc-startup.html
[tuicss-license]: https://github.com/vinibiavatti1/TuiCss/blob/6a021ecc2abb1fbe6da62bd370d1f2a764da1195/LICENSE.md
[energy-star]: https://www.energystar.gov/
[epa]: https://www.epa.gov/
