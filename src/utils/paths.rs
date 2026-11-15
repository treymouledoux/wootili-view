use std::{env::home_dir, path::PathBuf};

use scorched::{log_this, LogData, LogExpect, LogImportance};

pub fn logging_path() -> PathBuf {
    match home_dir() {
        Some(path) if cfg!(target_os = "macos") => {
            path.join("Library").join("Logs").join("Wootili-View")
        }
        Some(path) => path
            .join("AppData")
            .join("Local")
            .join("Wootili-View")
            .join("logs"),
        None => {
            log_this(LogData {
                importance: LogImportance::Warning,
                message: "Unable to get home directory, defaulting to current path".to_string(),
            });
            std::env::current_dir()
                .log_expect(LogImportance::Error, "Unable to get current directory")
                .join("logs")
        }
    }
}

pub fn config_path() -> PathBuf {
    match home_dir() {
        Some(path) if cfg!(target_os = "macos") => path
            .join("Library")
            .join("Application Support")
            .join("Wootili-View"),
        Some(path) => path.join("AppData").join("Local").join("Wootili-View"),
        None => {
            log_this(LogData {
                importance: LogImportance::Warning,
                message: "Unable to get home directory, defaulting to current path".to_string(),
            });
            std::env::current_dir()
                .log_expect(LogImportance::Error, "Unable to get current directory")
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn uses_macos_config_and_log_directories() {
        let home = home_dir().expect("test requires a home directory");
        assert_eq!(
            config_path(),
            home.join("Library/Application Support/Wootili-View")
        );
        assert_eq!(logging_path(), home.join("Library/Logs/Wootili-View"));
    }
}
