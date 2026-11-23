use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::i18n::Language;
use crate::ipc::MenuContext;
use crate::state::AppState;
use crate::tray;

/// What the menu bar is made of: a change here builds it again.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    /// None until the engine has sent its first state.
    pub language: Option<Language>,
    pub profiles: Vec<(Uuid, String)>,
    /// Only the installed app updates itself, so only it offers to.
    pub updates: bool,
}

/// What its items show that changes in place, so an open menu stays open.
#[derive(Debug, Clone, PartialEq)]
pub struct Marks {
    pub active_profile: Option<Uuid>,
    pub paused: bool,
    pub update: (String, bool),
    pub context: MenuContext,
}

/// Before the app's state exists (the first menu is built before setup) everything is empty.
pub fn read(app: &AppHandle, context: MenuContext) -> (Layout, Marks) {
    let state = app.try_state::<AppState>();
    let snapshot = state.as_ref().and_then(|state| state.snapshot());
    let managed = state
        .as_ref()
        .is_some_and(|state| state.placement.login_item().is_some());
    let update = state
        .as_ref()
        .map(|state| tray::updates_item(managed, &state.updates.status()))
        .unwrap_or_default();
    let layout = Layout {
        language: snapshot.as_ref().map(|s| s.language),
        profiles: snapshot
            .as_ref()
            .map(|s| s.profiles.iter().map(|p| (p.id, p.name.clone())).collect())
            .unwrap_or_default(),
        updates: managed,
    };
    let marks = Marks {
        active_profile: snapshot.as_ref().map(|s| s.active_profile),
        paused: snapshot.as_ref().is_some_and(|s| s.paused),
        update,
        context,
    };
    (layout, marks)
}
