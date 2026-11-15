use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=plugins/update_check/src");
    println!("cargo:rerun-if-changed=plugins/update_check/Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=.cargo/config.toml");

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let cargo_profile = if profile == "debug" { "dev" } else { &profile };
    let filename = match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => "update_check.dll",
        "macos" => "libupdate_check.dylib",
        "linux" => "libupdate_check.so",
        os => panic!("Unsupported plugin target OS: {os}"),
    };

    // A separate target directory and jobserver prevent nested Cargo from
    // waiting for locks or worker slots held by this parent build.
    let plugin_target = out_dir.join("plugin-target");
    let status = Command::new(env::var_os("CARGO").unwrap())
        .current_dir(&manifest_dir)
        .args(["build", "--locked", "--package", "update_check", "--lib"])
        .arg("--manifest-path")
        .arg(manifest_dir.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(&plugin_target)
        .arg("--target")
        .arg(&target)
        .arg("--profile")
        .arg(cargo_profile)
        .arg("--jobs")
        .arg(env::var("NUM_JOBS").unwrap())
        .env_remove("CARGO_MAKEFLAGS")
        .env_remove("MAKEFLAGS")
        .env_remove("MFLAGS")
        .status()
        .expect("Could not start Cargo to build the update-check plugin");
    assert!(
        status.success(),
        "Update-check plugin build failed: {status}"
    );

    let plugin = plugin_target.join(&target).join(&profile).join(filename);
    // Standard Cargo OUT_DIR is <target>/<profile>/build/<package>/out.
    // This also handles CARGO_TARGET_DIR and explicit --target builds.
    let binary_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("Unexpected Cargo OUT_DIR layout");
    let destination = binary_dir.join(filename);
    let plugin_bytes = fs::read(&plugin).expect("Could not read the built update-check plugin");
    let needs_staging = fs::read(&destination).map_or(true, |bytes| bytes != plugin_bytes);
    if needs_staging {
        let staged = out_dir.join(filename);
        fs::copy(&plugin, &staged).expect("Could not stage the built update-check plugin");
        fs::rename(&staged, &destination).unwrap_or_else(|error| {
            panic!(
                "Could not stage {} beside the app: {error}",
                destination.display()
            )
        });
    }
    // Restore the plugin if it is deleted while Cargo's compilation cache remains.
    println!("cargo:rerun-if-changed={}", destination.display());
    println!(
        "cargo:rustc-env=WOOTILI_UPDATE_CHECK_PATH={}",
        destination.display()
    );
}
