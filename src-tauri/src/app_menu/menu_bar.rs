use std::sync::Mutex;

use tauri::menu::Menu;
use tauri::{AppHandle, Manager, Wry};

use super::items::Items;
use super::layout;
use super::model::{self, Layout, Marks};
use crate::ipc::MenuContext;

/// What the menu bar shows now, and what the window last said it is showing.
#[derive(Default)]
pub struct MenuBar {
    shown: Mutex<Option<Shown>>,
    context: Mutex<MenuContext>,
}

struct Shown {
    layout: Layout,
    marks: Option<Marks>,
    items: Items,
}

impl MenuBar {
    fn context(&self) -> MenuContext {
        *self.context.lock().expect("menu context")
    }
}

/// The first menu bar, which Tauri sets up itself.
pub fn build(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let bar = app.state::<MenuBar>();
    let (layout, marks) = model::read(app, bar.context());
    let (menu, items) = layout::build(app, &layout)?;
    items.apply(&marks)?;
    *bar.shown.lock().expect("menu bar") = Some(Shown {
        layout,
        marks: Some(marks),
        items,
    });
    Ok(menu)
}

/// Brings the menu bar in step with the app. It is built again only when its items change, as
/// that closes a menu the user has open.
pub fn refresh(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Err(err) = update(&handle) {
            log::warn!("app menu could not be updated: {err}");
        }
    });
}

pub fn set_context(app: &AppHandle, context: MenuContext) {
    let bar = app.state::<MenuBar>();
    let changed = {
        let mut current = bar.context.lock().expect("menu context");
        let changed = *current != context;
        *current = context;
        changed
    };
    if changed {
        refresh(app);
    }
}

fn update(app: &AppHandle) -> tauri::Result<()> {
    let bar = app.state::<MenuBar>();
    let (layout, marks) = model::read(app, bar.context());
    let mut shown = bar.shown.lock().expect("menu bar");
    if shown.as_ref().is_none_or(|shown| shown.layout != layout) {
        let (menu, items) = layout::build(app, &layout)?;
        app.set_menu(menu)?;
        *shown = Some(Shown {
            layout,
            marks: None,
            items,
        });
    }
    let shown = shown.as_mut().expect("just built");
    if shown.marks.as_ref() != Some(&marks) {
        shown.items.apply(&marks)?;
        shown.marks = Some(marks);
    }
    Ok(())
}
