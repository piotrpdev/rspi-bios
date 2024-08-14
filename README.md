# RSPI BIOS

[Raspberry Pi][raspberry] dashboard that mimics the style of old [BIOS][bios] designs.

## Features

<!-- TODO: Add more features and packages/crates -->

- [x] Served by a Rust-powered web server
  - *...using the [axum][axum] crate*
- [x] Displays real-time system data from the Raspberry Pi
  - *...using the [sysinfo][sysinfo] crate*
- [x] Mimics old [BIOS](bios) designs
  - *...using the [TuiCss][tuicss] package*

## License

This project is licensed under the GNU General Public License v3.0.

Based on [MIT-licensed Axum example code][axum-license].

Website design based on the "Award Modular BIOS", all rights reserved by
[Award Software][award] / [Phoenix Technologies][phoenix].

[Energy Star][energy-star] is a trademark of the [U.S. Environmental Protection Agency][epa].

[Raspberry Pi][raspberry] is a trademark of [Raspberry Pi Ltd][raspberry-foundation].

[raspberry]: https://www.raspberrypi.org/
[raspberry-foundation]: https://www.raspberrypi.org/about/
[bios]: https://en.wikipedia.org/wiki/BIOS
[axum]: https://github.com/tokio-rs/axum
[sysinfo]: https://github.com/GuillaumeGomez/sysinfo
[tuicss]: https://github.com/vinibiavatti1/TuiCss
[axum-license]: https://github.com/tokio-rs/axum/blob/main/axum/LICENSE
[award]: https://en.wikipedia.org/wiki/Award_Software
[phoenix]: https://www.phoenix.com/
[energy-star]: https://www.energystar.gov/
[epa]: https://www.epa.gov/
