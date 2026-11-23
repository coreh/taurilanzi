mod apps;
mod icons;
mod names;

pub use apps::{app_at, frontmost, running_apps};
pub use icons::app_icon;

use std::path::Path;

use super::{AppDetection, AppIconError};

/// Unsupported until it is tried on Windows: `SHDefExtractIconW` (see `icons.rs`) takes sizes up
/// to 256 pixels.
pub fn app_icon_at(_path: &Path, _size: u32) -> Result<Vec<u8>, AppIconError> {
    Err(AppIconError::Unsupported)
}

pub fn detection() -> AppDetection {
    AppDetection::Ready
}

pub fn set_up_detection() -> std::io::Result<AppDetection> {
    Ok(AppDetection::Ready)
}
