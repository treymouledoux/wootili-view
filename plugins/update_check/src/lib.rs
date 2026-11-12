use std::{
    ffi::{c_char, CStr},
    sync::OnceLock,
    time::Duration,
};

use reqwest::header::{HeaderMap, USER_AGENT};
use scorched::{logf, set_logging_path, LogData, LogImportance};

static LATEST_VER: OnceLock<Option<String>> = OnceLock::new();

/// Returns the byte length of the cached UTF-8 release tag, or -1 on failure.
/// A null buffer with zero capacity queries the length. The tag is copied
/// without a terminator when the supplied buffer is large enough.
///
/// # Safety
///
/// A non-null `log_path` must point to a valid nul-terminated string. A
/// non-null `buffer` must point to writable storage of at least `capacity` bytes.
#[no_mangle]
pub unsafe extern "C" fn wootili_update_check_v1(
    log_path: *const c_char,
    buffer: *mut u8,
    capacity: usize,
) -> isize {
    if log_path.is_null() || (buffer.is_null() && capacity != 0) {
        return -1;
    }
    let Ok(log_path) = (unsafe { CStr::from_ptr(log_path) }).to_str() else {
        return -1;
    };
    let Some(version) = LATEST_VER
        .get_or_init(|| std::panic::catch_unwind(|| get_latest_version(log_path)).unwrap_or(None))
        .as_ref()
    else {
        return -1;
    };

    if !buffer.is_null() && capacity >= version.len() {
        unsafe { std::ptr::copy_nonoverlapping(version.as_ptr(), buffer, version.len()) };
    }
    isize::try_from(version.len()).unwrap_or(-1)
}

fn get_latest_version(log_path: &str) -> Option<String> {
    set_logging_path(log_path);

    match get_version_info() {
        Ok(version) => {
            logf!(Info, "Successfully fetched latest version: {}", version);
            Some(version)
        }
        Err(err) => {
            logf!(
                Error,
                "Failed to get latest version because of the following error: {}",
                err
            );
            None
        }
    }
}

fn get_version_info() -> Result<String, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Wootili-View Version Check".parse().unwrap());

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(10))
        .build()?;

    let content = client
        .get("https://api.github.com/repos/treymouledoux/Wootili-view/releases/latest")
        .send()?
        .text()?;

    let json = serde_json::from_str::<serde_json::Value>(&content)?;

    match json["tag_name"].as_str() {
        Some(tag_name) => Ok(tag_name.to_string()),
        None => Err("Failed to get tag_name element in json".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn query_returns_length_and_copies_cached_tag_when_it_fits() {
        let tag = "v1.2.3";
        LATEST_VER.set(Some(tag.to_owned())).unwrap();
        let log_path = c"unused".as_ptr();
        let expected_length = tag.len() as isize;
        let mut buffer = [b'!'; 8];

        unsafe {
            assert_eq!(wootili_update_check_v1(ptr::null(), ptr::null_mut(), 0), -1);
            assert_eq!(wootili_update_check_v1(log_path, ptr::null_mut(), 1), -1);
            assert_eq!(
                wootili_update_check_v1(log_path, ptr::null_mut(), 0),
                expected_length
            );
            assert_eq!(
                wootili_update_check_v1(log_path, buffer.as_mut_ptr(), tag.len() - 1),
                expected_length
            );
            assert_eq!(buffer, *b"!!!!!!!!");
            assert_eq!(
                wootili_update_check_v1(log_path, buffer.as_mut_ptr(), buffer.len()),
                expected_length
            );
            assert_eq!(buffer, *b"v1.2.3!!");
        }
    }
}
