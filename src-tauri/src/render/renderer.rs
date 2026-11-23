use std::collections::HashSet;
use std::sync::Arc;

use resvg::usvg::fontdb::{Database, ID};

use super::cache::RenderCache;
use super::design::KeyDesign;
use super::rasterize::{self, RasterizeError};
use super::{fonts, svg};

/// Turns key designs into PNGs, caching identical results.
pub struct Renderer {
    fonts: Arc<Database>,
    /// Faces whose file is already mapped; see [`Renderer::keep_mapped`].
    mapped: HashSet<ID>,
    cache: RenderCache,
}

impl Renderer {
    pub fn new() -> Self {
        Self::with_fonts(fonts::load(true))
    }

    pub fn with_fonts(fonts: Arc<Database>) -> Self {
        Self {
            fonts,
            mapped: HashSet::new(),
            cache: RenderCache::default(),
        }
    }

    pub fn render(
        &mut self,
        design: &KeyDesign,
        size: (u32, u32),
    ) -> Result<Arc<Vec<u8>>, RasterizeError> {
        let svg = svg::build(design, size);
        let key = RenderCache::key(&svg, size);
        if let Some(png) = self.cache.get(key) {
            return Ok(png);
        }
        let (png, used) = rasterize::to_png(&svg, size, &self.fonts)?;
        self.keep_mapped(used);
        let png = Arc::new(png);
        self.cache.insert(key, Arc::clone(&png));
        Ok(png)
    }

    /// System fonts are only indexed by path, so without this every glyph outline usvg needs
    /// opens and maps the font file again, which made text several times slower to draw than the
    /// shapes around it. A face that has been drawn with stays mapped from then on: only its pages
    /// that are read take memory, and the system can drop them under pressure.
    fn keep_mapped(&mut self, used: Vec<ID>) {
        let fresh: Vec<ID> = used
            .into_iter()
            .filter(|id| !self.mapped.contains(id))
            .collect();
        if fresh.is_empty() {
            return;
        }
        // Nothing else holds the database between renders; if something does, try again later.
        let Some(db) = Arc::get_mut(&mut self.fonts) else {
            return;
        };
        for id in fresh {
            // SAFETY: font files are not expected to change on disk while the app runs.
            unsafe {
                db.make_shared_face_data(id);
            }
            self.mapped.insert(id);
        }
    }

    pub fn font_families(&self) -> Vec<String> {
        fonts::families(&self.fonts)
    }

    #[cfg(test)]
    pub fn cached_renders(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::design::{Background, Color, Layer, VerticalPosition};

    fn renderer() -> Renderer {
        Renderer::with_fonts(fonts::load(false))
    }

    fn decode(png: &[u8]) -> (u32, u32, Vec<u8>) {
        let decoder = png::Decoder::new(std::io::Cursor::new(png));
        let mut reader = decoder.read_info().unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        (info.width, info.height, pixels)
    }

    fn label(text: &str) -> KeyDesign {
        KeyDesign {
            background: Background::Solid {
                color: Color::BLACK,
            },
            layers: vec![Layer::Text {
                content: text.into(),
                font_family: fonts::DEFAULT_FAMILY.into(),
                font_weight: 700,
                size: 48.0,
                color: Color::WHITE,
                position: VerticalPosition::Middle,
                outline: None,
            }],
        }
    }

    #[test]
    fn renders_requested_size() {
        let png = renderer()
            .render(&KeyDesign::solid(Color::rgb(255, 0, 0)), (452, 196))
            .unwrap();
        let (w, h, pixels) = decode(&png);
        assert_eq!((w, h), (452, 196));
        assert_eq!(&pixels[..3], &[255, 0, 0]);
    }

    #[test]
    fn bundled_font_draws_text() {
        let png = renderer().render(&label("Hi"), (196, 196)).unwrap();
        let (_, _, pixels) = decode(&png);
        let lit = pixels.chunks(4).filter(|px| px[0] > 200).count();
        assert!(lit > 200, "text should light up pixels, got {lit}");
    }

    #[test]
    fn identical_designs_hit_the_cache() {
        let mut renderer = renderer();
        let first = renderer.render(&label("A"), (196, 196)).unwrap();
        let second = renderer.render(&label("A"), (196, 196)).unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        renderer.render(&label("B"), (196, 196)).unwrap();
        assert_eq!(renderer.cached_renders(), 2);
    }

    #[test]
    fn embeds_svg_images() {
        let icon = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="#00ff00"/></svg>"##;
        let design = KeyDesign {
            background: Background::Solid {
                color: Color::BLACK,
            },
            layers: vec![Layer::Image {
                image: crate::render::design::ImageData {
                    format: crate::render::design::ImageFormat::Svg,
                    bytes: Arc::from(icon.as_bytes()),
                },
                fit: crate::render::design::Fit::Contain,
                inset: 0.0,
                tint: None,
            }],
        };
        let png = renderer().render(&design, (196, 196)).unwrap();
        let (_, _, pixels) = decode(&png);
        let center = (98 * 196 + 98) * 4;
        assert_eq!(&pixels[center..center + 3], &[0, 255, 0]);
    }

    fn tinted_icon(icon: &str, tint: Color) -> Vec<u8> {
        let design = KeyDesign {
            background: Background::Solid {
                color: Color::BLACK,
            },
            layers: vec![Layer::Image {
                image: crate::render::design::ImageData {
                    format: crate::render::design::ImageFormat::Svg,
                    bytes: Arc::from(icon.as_bytes()),
                },
                fit: crate::render::design::Fit::Contain,
                inset: 0.25,
                tint: Some(tint),
            }],
        };
        decode(&renderer().render(&design, (196, 196)).unwrap()).2
    }

    #[test]
    fn tints_icons_of_any_color_and_leaves_their_background_alone() {
        let blue = Color::rgb(0, 102, 255);
        let white = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="white"/></svg>"##;
        let current = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="currentColor"/></svg>"##;
        for icon in [white, current] {
            let pixels = tinted_icon(icon, blue);
            let center = (98 * 196 + 98) * 4;
            assert_eq!(&pixels[center..center + 3], &[0, 102, 255]);
            let corner = (4 * 196 + 4) * 4;
            assert_eq!(&pixels[corner..corner + 3], &[0, 0, 0], "outside the icon");
        }
    }
}
