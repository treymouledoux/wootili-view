# Wootili-View

<img src="assets/icon.png" width="128" alt="Wootili-View W icon">

Wootili-View turns the colors on your screen into ambient lighting on a Wooting keyboard. It captures your chosen display, scales each frame to your keyboard’s RGB layout, and maps the resulting colors to individual keys.

![Wootili-View settings and RGB preview](.github/assets/app_preview.png)

## Features

- Choose which display to capture and preview the RGB layout in the app.
- Adjust brightness and scaling, dim bright scenes, correct red tint, or highlight WASD.
- Set separate frame-rate limits for the interface and keyboard lighting.
- Check for new GitHub releases on startup with the optional update-check plugin.

## Platforms and prerequisites

Install Rust 1.95 or newer to build from source.

| Platform | Status and build requirements |
| --- | --- |
| Windows | Original platform. Install Visual Studio C++ Build Tools with the Windows SDK and LLVM/libclang. Set `LIBCLANG_PATH` if Cargo can’t find libclang. |
| macOS | Experimental. The build, startup, device refresh, shutdown, and update-check plugin have been verified. Screen capture and lighting on a physical Wooting keyboard still need testing. Install the Xcode Command Line Tools with Clang and libclang. |
| Linux | The SDK wrapper supports Linux and needs libclang, `pkg-config`, and the libudev development headers (`libudev-dev` on Ubuntu). The full app remains unverified. |

macOS builds use Apple Clang through the project’s `.cargo/config.toml`, even if your shell sets a generic `CC` or `CXX` to GCC.

## Build and run

Clone with recursive submodules to fetch the RGB wrapper, the official Wooting RGB SDK, and HIDAPI.

```sh
git clone --recurse-submodules https://github.com/treymouledoux/wootili-view.git
cd wootili-view
cargo build --release
cargo run --release
```

For an existing checkout, run `git submodule update --init --recursive` before building. Use `cargo run` for a debug build and `cargo test --workspace` to run the app, plugin, and SDK wrapper tests.

`build.rs` builds the update-check plugin and places it beside the executable. Keep both files in the same directory when distributing the app with update checking enabled.

| Platform | Executable | Update-check plugin |
| --- | --- | --- |
| Windows | `target/release/wootili-view.exe` | `target/release/update_check.dll` |
| macOS | `target/release/wootili-view` | `target/release/libupdate_check.dylib` |

The Windows Inno Setup script, `Wootili-View.iss`, uses these release files. `cargo test` checks that the staged plugin loads. To run the optional UI test that calls the GitHub Releases API, use `cargo test staged_update_check_plugin_draws_across_frames -- --ignored`.

## Settings and logs

| Platform | Settings | Logs |
| --- | --- | --- |
| Windows | `~/AppData/Local/Wootili-View/config.ron` | `~/AppData/Local/Wootili-View/logs` |
| macOS | `~/Library/Application Support/Wootili-View/config.ron` | `~/Library/Logs/Wootili-View` |

## Updating the RGB SDK wrapper

The app uses [wooting-rgb-rust](https://github.com/treymouledoux/wooting-rgb-rust), pinned at `vendor/wooting-rgb-rust`. The wrapper builds its pinned Wooting RGB SDK and HIDAPI sources. The app reads keyboard layouts from SDK metadata, which includes the 80HE and 60HE V2.

To update the wrapper, check out a reviewed commit in `vendor/wooting-rgb-rust`, run `git -C vendor/wooting-rgb-rust submodule update --init --recursive` from the app repository, then build and test. Commit the updated `vendor/wooting-rgb-rust` submodule pointer in the app repository.

I’m planning a rewrite of the RGB controls and interface.
