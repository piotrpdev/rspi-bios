# RSPI BIOS

[Raspberry Pi][raspberry] dashboard that mimics the style of old [BIOS][bios] designs.

## TODO

- [ ] send built info
- [ ] send versions of the CI
- [ ] send versions of rust, npm, etc. use npm built equivalent
- [ ] highlight rust/axum process in the list of processes
- [x] switch to using JS framework for the frontend
- [x] setup hot reloading for front and back end
- [ ] optimize perf
- [x] pre-commit lint, rust fmt, clippy, build check, etc.
- [x] switch from websockets back to sse
- [ ] implement HTTPS/TLS
- [ ] create Dockerfile
- [ ] add compression layer
- [ ] add CORS
- [ ] add 404
- [ ] add proper error handling
- [ ] add graceful tls shutdown
- [ ] maybe handle HEAD
- [x] show/hide pages in screens

## Features

<!-- TODO: Add more features and packages/crates -->

- [x] Served by a Rust-powered web server
  - *...using the [axum][axum] crate*
- [x] Displays real-time system data from the Raspberry Pi
  - *...using the [sysinfo][sysinfo] crate*
- [x] Mimics old [BIOS](bios) designs
  - *...using the [TuiCss][tuicss] package*

## Usage

### Release

```bash
NODE_ENV=production npm run --prefix ./web/ build && cargo build --release
```

### Development

> [!NOTE]
> You might need to check `Disable cache` in Chrome DevTools to avoid `404`s.

```bash
# Listen for ./web/ file changes and rebuild
npm run --prefix ./web/ build:watch
# Listen for ./src/ file changes and rebuild (run in another terminal)
cargo-watch --watch src -x run
```

## License

This project is licensed under the [GNU GPL v3.0][gplv3], see [`LICENSE`][license].

Based on [MIT][mit]-licensed [Axum example code][axum-examples],
see [`tokio-rs/axum/LICENSE`][axum-license].

Some code copied from [this template][axum-vite-template] (unknown license).

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
[axum-vite-template]: https://github.com/varonroy/template-axum-htmx-vite-tailwind
[award]: https://en.wikipedia.org/wiki/Award_Software
[phoenix]: https://www.phoenix.com/
[energy-star]: https://www.energystar.gov/
[epa]: https://www.epa.gov/
