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

This project is licensed under the [GNU GPL v3.0][gplv3], see [`LICENSE`][license].

Based on [MIT][mit]-licensed [Axum example code][axum-examples],
see [`tokio-rs/axum/LICENSE`][axum-license].

Website design based on the "Award Modular BIOS", all rights reserved by
[Award Software][award] / [Phoenix Technologies][phoenix].

[Energy Star][energy-star] is a trademark of the
[U.S. Environmental Protection Agency][epa].

[Raspberry Pi][raspberry] is a trademark of [Raspberry Pi Ltd][raspberry-foundation].

[raspberry]: https://www.raspberrypi.org/
[raspberry-foundation]: https://www.raspberrypi.org/about/
[bios]: https://en.wikipedia.org/wiki/BIOS
[axum]: https://github.com/tokio-rs/axum
[sysinfo]: https://github.com/GuillaumeGomez/sysinfo
[tuicss]: https://github.com/vinibiavatti1/TuiCss
[gplv3]: https://www.gnu.org/licenses/gpl-3.0.en.html
[license]: ./LICENSE
[mit]: https://opensource.org/license/mit
[axum-examples]: https://github.com/tokio-rs/axum/tree/main/examples
[axum-license]: https://github.com/tokio-rs/axum/blob/main/axum/LICENSE
[award]: https://en.wikipedia.org/wiki/Award_Software
[phoenix]: https://www.phoenix.com/
[energy-star]: https://www.energystar.gov/
[epa]: https://www.epa.gov/
