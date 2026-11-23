//! Ids of the menu bar's items. They share one event handler with the tray menu, so all start
//! with `menu:`.

/// Followed by the command the window carries out, as `MenuCommand` sends it.
pub const UI_PREFIX: &str = "menu:ui:";
/// Followed by the profile's id.
pub const PROFILE_PREFIX: &str = "menu:profile:";
pub const NEXT_PROFILE: &str = "menu:next-profile";
pub const PREVIOUS_PROFILE: &str = "menu:previous-profile";
pub const PAUSE: &str = "menu:pause";
pub const UPDATES: &str = "menu:updates";
pub const WEBSITE: &str = "menu:website";
pub const RELEASE_NOTES: &str = "menu:release-notes";
pub const REPORT_ISSUE: &str = "menu:report-issue";
