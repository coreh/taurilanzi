use tauri::Wry;
use tauri::menu::{CheckMenuItem, MenuItem};
use uuid::Uuid;

use super::model::Marks;

/// When an item the window carries out can be clicked.
#[derive(Debug, Clone, Copy)]
pub enum When {
    Always,
    Undo,
    Redo,
    /// The canvas of pages is on screen.
    Editor,
    /// A key is selected on the canvas, for the items that act on it.
    Key,
}

impl When {
    fn holds(self, marks: &Marks) -> bool {
        let context = &marks.context;
        match self {
            Self::Always => true,
            Self::Undo => context.undo,
            Self::Redo => context.redo,
            Self::Editor => context.editor,
            Self::Key => context.editor && context.key,
        }
    }
}

/// The items whose state follows the app, kept to update them without building the menu again.
#[derive(Default)]
pub struct Items {
    pub ui: Vec<(MenuItem<Wry>, When)>,
    pub profiles: Vec<(Uuid, CheckMenuItem<Wry>)>,
    pub pause: Option<CheckMenuItem<Wry>>,
    pub updates: Option<MenuItem<Wry>>,
}

impl Items {
    pub fn apply(&self, marks: &Marks) -> tauri::Result<()> {
        for (item, when) in &self.ui {
            item.set_enabled(when.holds(marks))?;
        }
        for (id, item) in &self.profiles {
            item.set_checked(marks.active_profile == Some(*id))?;
        }
        if let Some(pause) = &self.pause {
            pause.set_checked(marks.paused)?;
        }
        if let Some(updates) = &self.updates {
            let (text, enabled) = &marks.update;
            updates.set_text(text)?;
            updates.set_enabled(*enabled)?;
        }
        Ok(())
    }
}
