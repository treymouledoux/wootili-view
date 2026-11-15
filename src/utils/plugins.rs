use eframe::egui;
use libloading::Library;
use std::{
    cmp::Ordering,
    ffi::{c_char, CStr, CString},
    ptr,
    sync::{Arc, OnceLock},
};

const UPDATE_CHECK_SYMBOL: &CStr = c"wootili_update_check_v1";
type QueryVersion = unsafe extern "C" fn(*const c_char, *mut u8, usize) -> isize;

#[derive(Clone)]
pub struct Plugin {
    pub name: String,
    pub lib: Arc<Library>,
}

static AVAILABLE_PLUGINS: OnceLock<Vec<Plugin>> = OnceLock::new();

pub fn get_available_plugins() -> Vec<Plugin> {
    unsafe {
        AVAILABLE_PLUGINS
            .get_or_init(|| {
                let mut detected_plugins = Vec::new();

                // Update Check Plugin
                let Some(directory) = std::env::current_exe()
                    .ok()
                    .and_then(|path| path.parent().map(std::path::Path::to_path_buf))
                else {
                    return detected_plugins;
                };
                let plugin_path = directory.join(libloading::library_filename("update_check"));
                if let Ok(lib) = Library::new(plugin_path) {
                    if lib.get::<QueryVersion>(UPDATE_CHECK_SYMBOL).is_ok() {
                        detected_plugins.push(Plugin {
                            name: "update_check".to_string(),
                            lib: Arc::new(lib),
                        });
                    }
                }
                detected_plugins
            })
            .to_vec()
    }
}

pub fn update_check_ui(loaded_lib: Arc<Library>, ui: &mut egui::Ui, log_path: String) {
    let Ok(query_version) = (unsafe { loaded_lib.get::<QueryVersion>(UPDATE_CHECK_SYMBOL) }) else {
        return;
    };
    let latest_version = latest_version(*query_version, &log_path);
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
    const RELEASE_URL: &str = "https://github.com/treymouledoux/wootili-view/releases/tag/";

    ui.horizontal(|ui| {
        ui.hyperlink_to(
            format!("Wootili-View {CURRENT_VERSION}"),
            format!("{RELEASE_URL}{CURRENT_VERSION}"),
        );
        let Some(latest_version) = latest_version else {
            ui.separator();
            ui.label("Failed to check for updates").on_hover_text(
                "Failed to check for updates, try checking your internet connection",
            );
            return;
        };

        match ver_cmp::compare_versions(CURRENT_VERSION, &latest_version) {
            Ok(Ordering::Less) => {
                ui.separator();
                ui.label("New Version Available").on_hover_ui(|ui| {
                    ui.label(format!("New version available: {latest_version}"));
                    ui.hyperlink_to("Download", format!("{RELEASE_URL}{latest_version}"));
                });
            }
            Ok(Ordering::Greater) => {
                ui.separator();
                ui.label("Developer Build").highlight().on_hover_text(
                    "This build has been detected as unpublished meaning this is most likely a developer build or a pulled release; this build may be unstable or have unfinished/broken features.",
                );
            }
            Ok(Ordering::Equal) => {}
            Err(_) => scorched::log_this(scorched::LogData {
                importance: scorched::LogImportance::Error,
                message: "Failed to compare versions, this is likely due to a version format error"
                    .to_string(),
            }),
        }
    });
}

fn latest_version(query_version: QueryVersion, log_path: &str) -> Option<String> {
    let log_path = CString::new(log_path).ok()?;
    // The library writes to a host-owned buffer so ownership stays in one binary.
    let length = unsafe { query_version(log_path.as_ptr(), ptr::null_mut(), 0) };
    let size = usize::try_from(length).ok()?;
    let mut version = Vec::new();
    version.try_reserve_exact(size).ok()?;
    version.resize(size, 0);
    let written = unsafe { query_version(log_path.as_ptr(), version.as_mut_ptr(), version.len()) };
    if written != length {
        return None;
    }
    String::from_utf8(version).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_builds_a_loadable_update_check_plugin() {
        // Loading and resolving the symbol exercises the actual staged library
        // without calling the UI or making an update-check network request.
        unsafe {
            let library = Library::new(env!("WOOTILI_UPDATE_CHECK_PATH")).unwrap();
            library.get::<QueryVersion>(UPDATE_CHECK_SYMBOL).unwrap();
        }
    }

    #[test]
    #[ignore = "makes one request to the GitHub releases API"]
    fn staged_update_check_plugin_draws_across_frames() {
        let library = Arc::new(unsafe { Library::new(env!("WOOTILI_UPDATE_CHECK_PATH")) }.unwrap());
        let ctx = egui::Context::default();
        let log_path =
            std::env::temp_dir().join(format!("wootili-update-check-test-{}", std::process::id()));
        std::fs::create_dir(&log_path).unwrap();

        let version_label = format!("Wootili-View {}", env!("CARGO_PKG_VERSION"));
        let frames_drew_version = (0..2)
            .map(|_| {
                let output = ctx.run_ui(egui::RawInput::default(), |ui| {
                    update_check_ui(library.clone(), ui, log_path.to_string_lossy().into_owned());
                });
                let drew_version = output.shapes.iter().any(|shape| {
                    matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text() == version_label)
                });
                output.drop_without_applying_deltas();
                drew_version
            })
            .collect::<Vec<_>>();

        std::fs::remove_dir_all(&log_path).unwrap();
        assert_eq!(frames_drew_version, [true, true]);
    }
}
