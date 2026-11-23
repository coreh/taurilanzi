//! The menus of the menu bar, in the order and with the shortcuts of Apple's guidelines.

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Wry};

use super::ids::{
    NEXT_PROFILE, PAUSE, PREVIOUS_PROFILE, PROFILE_PREFIX, RELEASE_NOTES, REPORT_ISSUE, UI_PREFIX,
    UPDATES, WEBSITE,
};
use super::items::{Items, When};
use super::model::Layout;

/// Profiles past these have no shortcut.
const PROFILE_SHORTCUTS: usize = 9;

struct Builder<'a> {
    app: &'a AppHandle,
    items: Items,
}

impl Builder<'_> {
    /// An item the window carries out: `command` reaches it in `MenuCommand`.
    fn ui(
        &mut self,
        command: &str,
        text: &str,
        accelerator: Option<&str>,
        when: When,
    ) -> tauri::Result<MenuItem<Wry>> {
        let id = format!("{UI_PREFIX}{command}");
        let item = MenuItem::with_id(self.app, id, text, true, accelerator)?;
        self.items.ui.push((item.clone(), when));
        Ok(item)
    }

    fn native(&self, id: &str, text: &str) -> tauri::Result<MenuItem<Wry>> {
        MenuItem::with_id(self.app, id, text, true, None::<&str>)
    }

    fn separator(&self) -> tauri::Result<PredefinedMenuItem<Wry>> {
        PredefinedMenuItem::separator(self.app)
    }
}

pub fn build(app: &AppHandle, layout: &Layout) -> tauri::Result<(Menu<Wry>, Items)> {
    let mut b = Builder {
        app,
        items: Items::default(),
    };
    let menus = [
        app_menu(&mut b, layout)?,
        file(&mut b)?,
        edit(&mut b)?,
        view(&mut b)?,
        dock(&mut b, layout)?,
        window(&b)?,
        help(&b)?,
    ];
    let refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> = menus.iter().map(|m| m as _).collect();
    Ok((Menu::with_items(app, &refs)?, b.items))
}

fn app_menu(b: &mut Builder, layout: &Layout) -> tauri::Result<Submenu<Wry>> {
    let app = b.app;
    let name = &app.package_info().name;
    let main = Submenu::new(app, name, true)?;
    main.append(&PredefinedMenuItem::about(
        app,
        Some(trf!("menu.app.about", name = name).as_str()),
        None,
    )?)?;
    if layout.updates {
        let updates = b.native(UPDATES, tr!("tray.updates.check"))?;
        main.append(&updates)?;
        b.items.updates = Some(updates);
    }
    main.append_items(&[
        &b.separator()?,
        &b.ui(
            "app.settings",
            tr!("menu.app.settings"),
            Some("CmdOrCtrl+,"),
            When::Always,
        )?,
        &b.ui("app.account", tr!("menu.app.account"), None, When::Always)?,
        &b.ui("app.mcp", tr!("menu.app.mcp"), None, When::Always)?,
        &b.separator()?,
        &PredefinedMenuItem::services(app, Some(tr!("menu.app.services")))?,
        &b.separator()?,
        &PredefinedMenuItem::hide(app, Some(trf!("menu.app.hide", name = name).as_str()))?,
        &PredefinedMenuItem::hide_others(app, Some(tr!("menu.app.hideOthers")))?,
        &PredefinedMenuItem::show_all(app, Some(tr!("menu.app.showAll")))?,
        &b.separator()?,
        &PredefinedMenuItem::quit(app, Some(trf!("menu.app.quit", name = name).as_str()))?,
    ])?;
    Ok(main)
}

fn file(b: &mut Builder) -> tauri::Result<Submenu<Wry>> {
    Submenu::with_items(
        b.app,
        tr!("menu.file.title"),
        true,
        &[
            &b.ui(
                "file.newPage",
                tr!("menu.file.newPage"),
                Some("CmdOrCtrl+N"),
                When::Editor,
            )?,
            &b.ui(
                "file.newProfile",
                tr!("menu.file.newProfile"),
                Some("CmdOrCtrl+Shift+N"),
                When::Always,
            )?,
            &b.separator()?,
            &b.ui(
                "file.pluginStore",
                tr!("menu.file.pluginStore"),
                None,
                When::Always,
            )?,
            &b.ui(
                "file.installPluginFile",
                tr!("menu.file.installPluginFile"),
                None,
                When::Always,
            )?,
            &b.ui(
                "file.installPluginFolder",
                tr!("menu.file.installPluginFolder"),
                None,
                When::Always,
            )?,
            &b.ui(
                "file.reloadPlugins",
                tr!("menu.file.reloadPlugins"),
                None,
                When::Always,
            )?,
            &b.ui(
                "file.showPluginsFolder",
                tr!("menu.file.showPluginsFolder"),
                None,
                When::Always,
            )?,
            &b.separator()?,
            &PredefinedMenuItem::close_window(b.app, Some(tr!("menu.file.close")))?,
        ],
    )
}

/// Undo and redo are the app's own: the predefined ones reach only text being typed, and the
/// window decides which of both it undoes.
fn edit(b: &mut Builder) -> tauri::Result<Submenu<Wry>> {
    Submenu::with_items(
        b.app,
        tr!("menu.edit.title"),
        true,
        &[
            &b.ui(
                "edit.undo",
                tr!("menu.edit.undo"),
                Some("CmdOrCtrl+Z"),
                When::Undo,
            )?,
            &b.ui(
                "edit.redo",
                tr!("menu.edit.redo"),
                Some("CmdOrCtrl+Shift+Z"),
                When::Redo,
            )?,
            &b.separator()?,
            &PredefinedMenuItem::cut(b.app, Some(tr!("menu.edit.cut")))?,
            &PredefinedMenuItem::copy(b.app, Some(tr!("menu.edit.copy")))?,
            &PredefinedMenuItem::paste(b.app, Some(tr!("menu.edit.paste")))?,
            &PredefinedMenuItem::select_all(b.app, Some(tr!("menu.edit.selectAll")))?,
        ],
    )
}

fn view(b: &mut Builder) -> tauri::Result<Submenu<Wry>> {
    Submenu::with_items(
        b.app,
        tr!("menu.view.title"),
        true,
        &[
            &b.ui(
                "view.zoomIn",
                tr!("menu.view.zoomIn"),
                Some("CmdOrCtrl+="),
                When::Editor,
            )?,
            &b.ui(
                "view.zoomOut",
                tr!("menu.view.zoomOut"),
                Some("CmdOrCtrl+-"),
                When::Editor,
            )?,
            &b.ui(
                "view.actualSize",
                tr!("menu.view.actualSize"),
                Some("CmdOrCtrl+0"),
                When::Editor,
            )?,
            &b.ui(
                "view.fit",
                tr!("menu.view.fit"),
                Some("CmdOrCtrl+9"),
                When::Editor,
            )?,
            &b.ui(
                "view.pageOnDock",
                tr!("menu.view.pageOnDock"),
                None,
                When::Editor,
            )?,
            &b.separator()?,
            &PredefinedMenuItem::fullscreen(b.app, Some(tr!("menu.view.fullScreen")))?,
        ],
    )
}

fn dock(b: &mut Builder, layout: &Layout) -> tauri::Result<Submenu<Wry>> {
    let app = b.app;
    let menu = Submenu::new(app, tr!("menu.dock.title"), true)?;
    for (n, (id, name)) in layout.profiles.iter().enumerate() {
        let shortcut = (n < PROFILE_SHORTCUTS).then(|| format!("Alt+CmdOrCtrl+{}", n + 1));
        let item = CheckMenuItem::with_id(
            app,
            format!("{PROFILE_PREFIX}{id}"),
            name,
            true,
            false,
            shortcut.as_deref(),
        )?;
        menu.append(&item)?;
        b.items.profiles.push((*id, item));
    }
    let several = layout.profiles.len() > 1;
    if !layout.profiles.is_empty() {
        menu.append(&b.separator()?)?;
    }
    let pause = CheckMenuItem::with_id(
        app,
        PAUSE,
        tr!("menu.dock.pause"),
        true,
        false,
        None::<&str>,
    )?;
    menu.append_items(&[
        &MenuItem::with_id(
            app,
            NEXT_PROFILE,
            tr!("menu.dock.nextProfile"),
            several,
            Some("CmdOrCtrl+Shift+]"),
        )?,
        &MenuItem::with_id(
            app,
            PREVIOUS_PROFILE,
            tr!("menu.dock.previousProfile"),
            several,
            Some("CmdOrCtrl+Shift+["),
        )?,
        &b.separator()?,
        &pause,
    ])?;
    b.items.pause = Some(pause);
    Ok(menu)
}

/// macOS adds the open windows and its own window items to the menu set as the Windows menu.
fn window(b: &Builder) -> tauri::Result<Submenu<Wry>> {
    let menu = Submenu::with_items(
        b.app,
        tr!("menu.window.title"),
        true,
        &[
            &PredefinedMenuItem::minimize(b.app, Some(tr!("menu.window.minimize")))?,
            &PredefinedMenuItem::maximize(b.app, Some(tr!("menu.window.zoom")))?,
            &b.separator()?,
            &PredefinedMenuItem::bring_all_to_front(
                b.app,
                Some(tr!("menu.window.bringAllToFront")),
            )?,
        ],
    )?;
    menu.set_as_windows_menu_for_nsapp()?;
    Ok(menu)
}

/// Set as the Help menu, macOS puts its search field first.
fn help(b: &Builder) -> tauri::Result<Submenu<Wry>> {
    let name = &b.app.package_info().name;
    let menu = Submenu::with_items(
        b.app,
        tr!("menu.help.title"),
        true,
        &[
            &b.native(WEBSITE, &trf!("menu.help.website", name = name))?,
            &b.native(RELEASE_NOTES, tr!("menu.help.releaseNotes"))?,
            &b.separator()?,
            &b.native(REPORT_ISSUE, tr!("menu.help.reportIssue"))?,
        ],
    )?;
    menu.set_as_help_menu_for_nsapp()?;
    Ok(menu)
}
