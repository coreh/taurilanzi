use std::collections::HashSet;
use std::path::Path;

use objc2::AllocAnyThread;
use objc2::rc::autoreleasepool;
use objc2_app_kit::{
    NSApplicationActivationPolicy, NSBitmapImageFileType, NSBitmapImageRep, NSImage,
    NSRunningApplication, NSWorkspace,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGBitmapContextCreateImage, CGColorSpace, CGContext, CGImage,
    CGImageAlphaInfo, CGInterpolationQuality, kCGColorSpaceSRGB,
};
use objc2_foundation::{NSBundle, NSDictionary, NSFileManager, NSPoint, NSRect, NSSize, NSString};

use super::{AppDetection, AppIconError};
use crate::config::AppRef;

/// Points; the UI shows icons at a third of this, so they stay sharp on Retina screens.
const ICON_SIZE: f64 = 64.0;

/// The app in front and its process. A binary outside a bundle, as in development, has no app.
pub struct Front {
    pub pid: u32,
    pub app: Option<AppRef>,
}

/// Only apps of the Dock count: the lock screen, Spotlight or a launcher in front for a moment
/// must not switch the profile and back. This app always counts, so leaving it is noticed.
pub fn frontmost() -> Option<Front> {
    autoreleasepool(|_| {
        let running = NSWorkspace::sharedWorkspace().frontmostApplication()?;
        let pid = running.processIdentifier() as u32;
        if pid != std::process::id()
            && running.activationPolicy() != NSApplicationActivationPolicy::Regular
        {
            return None;
        }
        Some(Front {
            pid,
            app: app_ref(&running),
        })
    })
}

/// Apps with a window and a place in the Dock, apart from this one, by name.
pub fn running_apps() -> Vec<AppRef> {
    let own = std::process::id();
    autoreleasepool(|_| {
        let mut seen = HashSet::new();
        let mut apps: Vec<AppRef> = NSWorkspace::sharedWorkspace()
            .runningApplications()
            .iter()
            .filter(|running| {
                running.activationPolicy() == NSApplicationActivationPolicy::Regular
                    && running.processIdentifier() as u32 != own
            })
            .filter_map(|running| app_ref(&running))
            .filter(|app| seen.insert(app.id.clone()))
            .collect();
        apps.sort_by_key(|app| app.name.to_lowercase());
        apps
    })
}

/// The app bundle at `path`, as Finder names it.
pub fn app_at(path: &Path) -> Option<AppRef> {
    let path = path.to_str()?;
    autoreleasepool(|_| {
        let bundle = NSBundle::bundleWithPath(&NSString::from_str(path))?;
        let id = bundle.bundleIdentifier()?.to_string();
        let name = display_name(&NSString::from_str(path));
        Some(AppRef { id, name })
    })
}

/// The icon of the installed app with this bundle identifier, as a PNG.
pub fn app_icon(id: &str) -> Option<Vec<u8>> {
    autoreleasepool(|_| {
        let workspace = NSWorkspace::sharedWorkspace();
        let path = workspace
            .URLForApplicationWithBundleIdentifier(&NSString::from_str(id))?
            .path()?;
        png_of(&workspace.iconForFile(&path), ICON_SIZE)
    })
}

/// The icon of the app bundle at `path`, as a PNG of `size` pixels a side.
pub fn app_icon_at(path: &Path, size: u32) -> Result<Vec<u8>, AppIconError> {
    let path = path.to_str().ok_or(AppIconError::NotAnApp)?;
    autoreleasepool(|_| {
        let path = NSString::from_str(path);
        // Any folder opens as a bundle; only apps carry an identifier.
        NSBundle::bundleWithPath(&path)
            .and_then(|bundle| bundle.bundleIdentifier())
            .ok_or(AppIconError::NotAnApp)?;
        let icon = NSWorkspace::sharedWorkspace().iconForFile(&path);
        png_at(&icon, size).ok_or(AppIconError::NotAnApp)
    })
}

/// `size` in points: drawn without a context, the image has the screen's scale.
fn png_of(icon: &NSImage, size: f64) -> Option<Vec<u8>> {
    let mut rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(size, size));
    // SAFETY: `rect` outlives the call, and no hints are passed.
    let image = unsafe { icon.CGImageForProposedRect_context_hints(&mut rect, None, None) }?;
    encode(&image)
}

/// Exactly `pixels` a side whatever the screen, from the icon's picture at least that large.
fn png_at(icon: &NSImage, pixels: u32) -> Option<Vec<u8>> {
    let side = f64::from(pixels);
    let mut rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(side, side));
    // SAFETY: `rect` outlives the call, and no hints are passed.
    let image = unsafe { icon.CGImageForProposedRect_context_hints(&mut rect, None, None) }?;
    // SAFETY: a static of Core Graphics, alive for the whole process.
    let space = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB }))?;
    // SAFETY: without a buffer of ours, Core Graphics allocates and frees the pixels itself.
    let context = unsafe {
        CGBitmapContextCreate(
            std::ptr::null_mut(),
            pixels as usize,
            pixels as usize,
            8,
            0,
            Some(&space),
            CGImageAlphaInfo::PremultipliedLast.0,
        )
    }?;
    CGContext::set_interpolation_quality(Some(&context), CGInterpolationQuality::High);
    CGContext::draw_image(
        Some(&context),
        CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(side, side)),
        Some(&image),
    );
    let scaled = CGBitmapContextCreateImage(Some(&context))?;
    encode(&scaled)
}

fn encode(image: &CGImage) -> Option<Vec<u8>> {
    let bitmap = NSBitmapImageRep::initWithCGImage(NSBitmapImageRep::alloc(), image);
    // SAFETY: an empty dictionary is a valid set of properties.
    let png = unsafe {
        bitmap.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
    }?;
    Some(png.to_vec())
}

fn app_ref(running: &NSRunningApplication) -> Option<AppRef> {
    let id = running.bundleIdentifier()?.to_string();
    let name = running
        .bundleURL()
        .and_then(|url| url.path())
        .map(|path| display_name(&path))
        .or_else(|| running.localizedName().map(|name| name.to_string()))
        .unwrap_or_else(|| id.clone());
    Some(AppRef { id, name })
}

/// The name Finder shows, in the user's language and without `.app`.
fn display_name(path: &NSString) -> String {
    let name = NSFileManager::defaultManager()
        .displayNameAtPath(path)
        .to_string();
    name.strip_suffix(".app").map(str::to_owned).unwrap_or(name)
}

pub fn detection() -> AppDetection {
    AppDetection::Ready
}

pub fn set_up_detection() -> std::io::Result<AppDetection> {
    Ok(AppDetection::Ready)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Writes Calculator's icon to the temporary folder, to look at it.
    #[test]
    #[ignore]
    fn draws_an_app_icon_at_its_size() {
        let png = app_icon_at(Path::new("/System/Applications/Calculator.app"), 256).unwrap();
        assert_eq!(png[16..24], [0, 0, 1, 0, 0, 0, 1, 0]);
        let out = std::env::temp_dir().join("calculator-icon.png");
        std::fs::write(&out, &png).unwrap();
        println!("{}", out.display());
        assert!(matches!(
            app_icon_at(Path::new("/System"), 256),
            Err(AppIconError::NotAnApp)
        ));
    }
}
