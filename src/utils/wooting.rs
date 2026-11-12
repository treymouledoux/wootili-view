use std::sync::Mutex;

use image::GenericImageView;
use scorched::{logf, LogData, LogImportance};
use wooting_rgb_rust::{Device, DeviceInfo, DeviceType, Error, Frame, Rgb, Sdk};

// UI queries and the capture thread share one native SDK session. Hold the lock
// through an entire frame so refresh and shutdown cannot interrupt RGB writes.
static SDK: Mutex<Option<Sdk>> = Mutex::new(None);

fn with_device<T>(operation: impl FnOnce(&mut Device<'_>) -> Result<T, Error>) -> Result<T, Error> {
    let mut state = SDK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if crate::CLOSE_APP.load(std::sync::atomic::Ordering::Acquire) {
        return Err(Error::OperationFailed(
            "access SDK after application shutdown",
        ));
    }
    if state.is_none() {
        *state = Some(Sdk::new()?);
    }
    let sdk = state.as_mut().unwrap();
    if sdk.devices()?.is_empty() {
        sdk.refresh()?;
    }
    operation(&mut sdk.device(0)?)
}

fn capture_dimensions(info: &DeviceInfo) -> (u32, u32, u32) {
    // The 60% matrix retains an empty function-key row. Other layouts use row 0.
    let first_row = u32::from(info.device_type == DeviceType::SixtyPercent);
    if info.led_index_max == 0 {
        return (0, 0, first_row);
    }
    (
        info.columns.into(),
        u32::from(info.rows).saturating_sub(first_row),
        first_row,
    )
}

pub fn get_rgb_size() -> Option<(u32, u32)> {
    with_device(|device| {
        let (width, height, _) = capture_dimensions(&device.info()?);
        Ok((width, height))
    })
    .ok()
}

pub fn get_device_name() -> String {
    with_device(|device| Ok(device.info()?.model)).unwrap_or_else(|_| "N/A".into())
}

fn device_response(command: u8) -> Result<Vec<u8>, Error> {
    with_device(|device| {
        let info = device.info()?;
        let response = device.feature_response(command, [0; 4])?;
        // V2 responses include a report-ID byte before the legacy header.
        let header = if info.v2_interface { 5 } else { 4 };
        response
            .get(header..)
            .map(<[u8]>::to_vec)
            .ok_or(Error::OperationFailed("decode device information"))
    })
}

pub fn get_device_creation(_depth: u8) -> String {
    for _ in 0..3 {
        if let Ok(response) = device_response(3) {
            if let Some(date) = response.get(2..4) {
                if (1..=53).contains(&date[1]) {
                    return format!("Week {} of {}", date[1], 2000 + u16::from(date[0]));
                }
            }
        }
    }
    "N/A".into()
}

pub fn get_device_version() -> String {
    device_response(1)
        .ok()
        .and_then(|response| {
            response
                .get(..3)
                .map(|version| format!("{}.{}.{}", version[0], version[1], version[2]))
        })
        .unwrap_or_else(|| "N/A".into())
}

pub fn draw_rgb(
    resized_capture: image::DynamicImage,
    brightness: u8,
    red_shift_fix: bool,
    highlight_wasd: bool,
) -> bool {
    with_device(|device| {
        let info = device.info()?;
        let (width, height, first_row) = capture_dimensions(&info);
        if resized_capture.dimensions() != (width, height) {
            return Err(Error::OperationFailed(
                "draw image with outdated keyboard dimensions",
            ));
        }
        let mut frame = Frame::new();
        for (x, y, pixel) in resized_capture.pixels() {
            let row = y + first_row;
            let image::Rgba([r, g, b, _]) = pixel;
            // W is at row 2, and A/S/D at row 3 of the SDK's keyboard matrix.
            let wasd = info.device_type != DeviceType::Keypad
                && ((row == 2 && x == 2) || (row == 3 && (1..=3).contains(&x)));
            let color = if highlight_wasd && wasd {
                Rgb::new(255, 0, 0)
            } else {
                let exposed_spacebar =
                    info.device_type == DeviceType::SixtyPercent && row == 5 && x > 3 && x < 10;
                let (red, blue) = if red_shift_fix && !exposed_spacebar {
                    (r as f32 * 0.55, b as f32 * 1.2)
                } else {
                    (r as f32, b as f32)
                };
                let scale = f32::from(brightness.min(100)) / 100.0;
                Rgb::new(
                    (red * scale).round() as u8,
                    (g as f32 * scale).round() as u8,
                    (blue * scale).round() as u8,
                )
            };
            frame.set(row as usize, x as usize, color)?;
        }
        device.set_frame(&frame)?;
        device.flush()
    })
    .is_ok()
}

pub fn reconnect_device() {
    let mut state = SDK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(sdk) = state.as_mut() {
        let _ = sdk.refresh();
    }
}

pub fn exit_rgb() {
    let mut state = SDK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(sdk) = state.take() {
        if let Err(error) = sdk.close() {
            logf!(Warning, "RGB shutdown: {}", error);
        }
    }
}

pub fn update_rgb() {
    let _ = with_device(|device| device.flush());
}

#[cfg(test)]
mod tests {
    use super::*;
    use wooting_rgb_rust::Layout;

    #[test]
    fn capture_uses_device_dimensions_and_only_skips_the_60_percent_spacer() {
        let mut info = DeviceInfo {
            index: 0,
            model: "Test keyboard".into(),
            rows: 6,
            columns: 14,
            led_index_max: 116,
            device_type: DeviceType::SixtyPercent,
            layout: Layout::Ansi,
            v2_interface: true,
            uses_small_packets: false,
            uses_multi_report: true,
        };
        assert_eq!(capture_dimensions(&info), (14, 5, 1));
        info.device_type = DeviceType::FullSize;
        info.columns = 21;
        assert_eq!(capture_dimensions(&info), (21, 6, 0));
        info.device_type = DeviceType::EightyPercent;
        info.columns = 17;
        assert_eq!(capture_dimensions(&info), (17, 6, 0));
        info.device_type = DeviceType::Keypad;
        info.rows = 5;
        info.columns = 7;
        assert_eq!(capture_dimensions(&info), (7, 5, 0));
        info.led_index_max = 0;
        assert_eq!(capture_dimensions(&info), (0, 0, 0));
    }
}
