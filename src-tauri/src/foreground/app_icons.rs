/// Pixels of an app icon made into a key's image: more than a key's 196, so the image editor can
/// still crop into it.
pub const KEY_ICON_SIZE: u32 = 256;

/// Why an app chosen by the user gave no icon.
#[derive(Debug)]
pub enum AppIconError {
    #[cfg_attr(not(target_os = "macos"), expect(dead_code))]
    NotAnApp,
    /// This system has no way to read it yet.
    #[cfg_attr(target_os = "macos", expect(dead_code))]
    Unsupported,
}
