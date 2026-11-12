use eframe::egui;
use libloading::Library;
use std::sync::{Arc, OnceLock};

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
                    if lib
                        .get::<extern "C" fn(&mut egui::Ui, String)>(b"update_check_ui")
                        .is_ok()
                    {
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
    unsafe {
        let update_check_ui: libloading::Symbol<extern "C" fn(&mut egui::Ui, String)> =
            match loaded_lib.get("update_check_ui".as_bytes()) {
                Ok(update_check_ui) => update_check_ui,
                Err(_) => return,
            };
        update_check_ui(ui, log_path);
    }
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
            library
                .get::<extern "C" fn(&mut egui::Ui, String)>(b"update_check_ui")
                .unwrap();
        }
    }
}
