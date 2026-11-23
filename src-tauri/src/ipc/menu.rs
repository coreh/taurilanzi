//! What the window tells the menu bar, and what the menu bar asks of the window.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

/// What the window is showing, for the menu bar to enable only what can be done there.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, specta::Type)]
pub struct MenuContext {
    /// Undo would take something back: typing, an edit not saved yet or one in the history.
    pub undo: bool,
    pub redo: bool,
    /// The canvas of pages is on screen.
    pub editor: bool,
    /// A key is selected on the canvas.
    pub key: bool,
}

/// Only macOS has a menu bar of the app's; elsewhere the window keeps it to itself.
#[tauri::command]
#[specta::specta]
pub fn set_menu_context(app: AppHandle, context: MenuContext) {
    #[cfg(target_os = "macos")]
    crate::app_menu::set_context(&app, context);
    #[cfg(not(target_os = "macos"))]
    let _ = (app, context);
}
