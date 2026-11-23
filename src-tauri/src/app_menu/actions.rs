use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;
use tauri_specta::Event;
use uuid::Uuid;

use super::ids::{
    NEXT_PROFILE, PAUSE, PREVIOUS_PROFILE, PROFILE_PREFIX, RELEASE_NOTES, REPORT_ISSUE, UI_PREFIX,
    UPDATES, WEBSITE,
};
use crate::engine::{EngineHandle, Snapshot};
use crate::ipc::MenuCommand;
use crate::state::AppState;
use crate::updates::{self, Status};
use crate::window;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

pub fn on_menu(app: &AppHandle, id: &str) {
    if let Some(command) = id.strip_prefix(UI_PREFIX) {
        // The window may be closed to the menu bar while the app is still in front.
        window::show(app);
        let command = MenuCommand {
            command: command.to_owned(),
        };
        if let Err(err) = command.emit(app) {
            log::warn!("could not pass {id} to the window: {err}");
        }
        return;
    }
    match id {
        UPDATES => {
            if matches!(app.state::<AppState>().updates.status(), Status::Ready(_)) {
                updates::install(app);
            } else {
                updates::check(app);
            }
        }
        WEBSITE => open(app, env!("CARGO_PKG_HOMEPAGE")),
        RELEASE_NOTES => open(app, &format!("{REPOSITORY}/releases")),
        REPORT_ISSUE => open(app, &format!("{REPOSITORY}/issues")),
        PAUSE | NEXT_PROFILE | PREVIOUS_PROFILE => on_dock(app, id),
        _ if id.starts_with(PROFILE_PREFIX) => on_dock(app, id),
        _ => {}
    }
}

fn open(app: &AppHandle, url: &str) {
    if let Err(err) = app.opener().open_url(url, None::<&str>) {
        log::warn!("could not open {url}: {err}");
    }
}

fn on_dock(app: &AppHandle, id: &str) {
    let app = app.clone();
    let id = id.to_owned();
    tauri::async_runtime::spawn(async move {
        if let Err(err) = act(&app, &id).await {
            log::warn!("menu action {id} failed: {err}");
        }
    });
}

async fn act(app: &AppHandle, id: &str) -> Result<(), String> {
    let engine = app.state::<EngineHandle>().inner().clone();
    let snapshot = app
        .state::<AppState>()
        .snapshot()
        .ok_or("the engine has not started yet")?;
    let result = match id {
        PAUSE => engine.set_paused(!snapshot.paused).await,
        NEXT_PROFILE => engine.set_profile(neighbour(&snapshot, 1)).await,
        PREVIOUS_PROFILE => engine.set_profile(neighbour(&snapshot, -1)).await,
        _ => {
            let profile = id.strip_prefix(PROFILE_PREFIX).unwrap_or(id);
            let profile = profile.parse().map_err(|_| "bad profile id")?;
            engine.set_profile(profile).await
        }
    };
    result.map_err(|err| err.to_string())
}

/// The profile `step` places away from the active one in the sidebar's order, going round.
fn neighbour(snapshot: &Snapshot, step: isize) -> Uuid {
    let profiles = &snapshot.profiles;
    let Some(at) = profiles
        .iter()
        .position(|p| p.id == snapshot.active_profile)
    else {
        return snapshot.active_profile;
    };
    let count = profiles.len() as isize;
    let next = (at as isize + step).rem_euclid(count) as usize;
    profiles[next].id
}
