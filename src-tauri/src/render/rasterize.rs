use std::sync::{Arc, Mutex};

use resvg::tiny_skia::{self, Pixmap};
use resvg::usvg::{self, fontdb::Database, fontdb::ID};

use super::fonts::DEFAULT_FAMILY;

#[derive(Debug, thiserror::Error)]
pub enum RasterizeError {
    #[error("invalid svg: {0}")]
    Svg(#[from] usvg::Error),
    #[error("invalid canvas size {0}x{1}")]
    Size(u32, u32),
    #[error("png encoding: {0}")]
    Png(String),
}

/// Renders an SVG document to an opaque PNG of the given size, also telling which font faces its
/// text was drawn with.
pub fn to_png(
    svg: &str,
    (width, height): (u32, u32),
    fonts: &Arc<Database>,
) -> Result<(Vec<u8>, Vec<ID>), RasterizeError> {
    let used = Arc::new(Mutex::new(Vec::new()));
    let note = |id: Option<ID>, used: &Mutex<Vec<ID>>| {
        if let Some(id) = id {
            used.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(id);
        }
        id
    };
    let select_font = usvg::FontResolver::default_font_selector();
    let select_fallback = usvg::FontResolver::default_fallback_selector();
    let (for_font, for_fallback) = (Arc::clone(&used), Arc::clone(&used));
    let options = usvg::Options {
        font_family: DEFAULT_FAMILY.to_owned(),
        fontdb: Arc::clone(fonts),
        font_resolver: usvg::FontResolver {
            select_font: Box::new(move |font, db| note(select_font(font, db), &for_font)),
            select_fallback: Box::new(move |c, exclude, db| {
                note(select_fallback(c, exclude, db), &for_fallback)
            }),
        },
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(svg, &options)?;
    drop(options);

    let mut pixmap = Pixmap::new(width, height).ok_or(RasterizeError::Size(width, height))?;
    // The dock shows transparent pixels badly, so everything is painted over black.
    pixmap.fill(tiny_skia::Color::BLACK);
    let scale = tiny_skia::Transform::from_scale(
        width as f32 / tree.size().width(),
        height as f32 / tree.size().height(),
    );
    resvg::render(&tree, scale, &mut pixmap.as_mut());

    let png = pixmap
        .encode_png()
        .map_err(|err| RasterizeError::Png(err.to_string()))?;
    let used = std::mem::take(
        &mut *used
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    );
    Ok((png, used))
}
