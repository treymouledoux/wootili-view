# Wootili-View

![app_preview](https://github.com/treymouledoux/wootili-view/blob/master/.github/assets/app_preview.png?raw=true)

> [!IMPORTANT]
> I plan on doing a full rewrite to this software, this will overhaul the rgb controlling and the ui!

## About

Wootili-View is a project designed to enhance your Wooting device by providing an ambient lighting effect. This software allows you to map a downscaled version of your screen onto your Wooting keyboard, creating a dynamic and immersive ambilight experience.

## How It Works

The functionality of this project is straightforward process. The program captures screenshots of your specified display, then the program downscales it to match the size of your Wooting device's RGB layout. Subsequently, it maps the colors from the downscaled screenshots to the corresponding keys on your Wooting device, producing a captivating ambilight effect.

## Getting Started

> [!IMPORTANT]  
> At this current time this project is only supported on Windows.

### Prerequisites

You will need Rust 1.95+, Visual Studio C++ Build Tools with the Windows SDK, and LLVM/libclang to compile the program. Set `LIBCLANG_PATH` if LLVM is outside its usual installation directory.

Clone the nested SDK submodules before building.

```sh
git clone --recurse-submodules https://github.com/treymouledoux/wootili-view.git
cd wootili-view
cargo build --release
```

For an existing checkout, run `git submodule update --init --recursive`.

`cargo build --release` builds the app and the update-check plugin through the root `build.rs`. On Windows, the finished files are `target/release/wootili-view.exe` and `target/release/update_check.dll`. Keep both files together when copying the app. The Inno Setup script reads these release outputs directly. macOS builds require Xcode Command Line Tools with libclang and produce `target/release/wootili-view` with `target/release/libupdate_check.dylib` beside it.

Use `cargo run` for a debug build with the plugin, `cargo test` to check the app and plugin loading, and `cargo clean` to remove Cargo build outputs. The plugin and app share the workspace lockfile. The plugin returns version text through a C-compatible interface, and the app renders the update-check footer. The build script uses a separate output directory for its nested plugin build.

RGB control uses [wooting-rgb-rust](https://github.com/treymouledoux/wooting-rgb-rust), pinned as a submodule in `vendor/wooting-rgb-rust`. That wrapper builds its pinned official Wooting SDK and HIDAPI sources. Keyboard dimensions come from the SDK metadata, including 80HE and 60HE V2 models.

To update the wrapper, check out a reviewed commit inside `vendor/wooting-rgb-rust`, run `git submodule update --init --recursive`, build the app, and commit the new submodule pointer.
