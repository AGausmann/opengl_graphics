
//! Glyph caching

use error::Error;
use freetype::ffi;
use freetype;
use freetype::error::Error::MissingFontField;
use Texture;
use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};

pub type FontSize = u32;

/// Struct used to hold rendered character data.
pub struct Character {
    /// The glyph.
    pub glyph: freetype::Glyph,
    /// The bitmap of the character.
    pub bitmap_glyph: freetype::BitmapGlyph,
    /// The texture of the character.
    pub texture: Texture,
}

impl Character {
    /// The top offset.
    pub fn top(&self) -> i32 {
        self.bitmap_glyph.top()
    }

    /// The left offset.
    pub fn left(&self) -> i32 {
        self.bitmap_glyph.left()
    }
}

/// A struct used for caching rendered font.
pub struct GlyphCache {
    /// The font face.
    pub face: freetype::Face,
    data: HashMap<FontSize, HashMap<char, Character>>,
}

impl GlyphCache {

    /// Constructor for a GlyphCache.
    pub fn new(font: &Path) -> Result<GlyphCache, Error> {
        let freetype = match freetype::Library::init() {
            Ok(freetype) => freetype,
            Err(why) => return Err(Error::FreetypeError(why)),
        };
        let font_str = match font.as_str() {
            Some(font_str) => font_str,
            None => return Err(Error::FreetypeError(MissingFontField)),
        };
        let face = match freetype.new_face(font_str, 0) {
            Ok(face) => face,
            Err(why) => return Err(Error::FreetypeError(why)),
        };
        Ok(GlyphCache {
            face: face,
            data: HashMap::new(),
        })
    }

    /// Return a reference to a `Character`. If there is not yet a `Character` for
    /// the given `FontSize` and `char`, load the `Character`.
    pub fn get_character(&mut self, size: FontSize, ch: char) -> &Character {
        match {
            match self.data.entry(size) {
                Vacant(entry) => entry.set(HashMap::new()),
                Occupied(entry) => entry.into_mut(),
            }
        }.contains_key(&ch) {
            true => &self.data[size][ch],
            false => { self.load_character(size, ch); &self.data[size][ch] }
        }
    }

    /// Load a `Character` from a given `FontSize` and `char`.
    fn load_character(&mut self, size: FontSize, ch: char) {
        self.face.set_pixel_sizes(0, size).unwrap();
        self.face.load_char(ch as ffi::FT_ULong, freetype::face::DEFAULT).unwrap();
        let glyph = self.face.glyph().get_glyph().unwrap();
        let bitmap_glyph = glyph.to_bitmap(freetype::render_mode::RenderMode::Normal, None)
            .unwrap();
        let bitmap = bitmap_glyph.bitmap();
        let texture = Texture::from_memory_alpha(bitmap.buffer(),
                                                 bitmap.width() as u32,
                                                 bitmap.rows() as u32).unwrap();
        self.data[size].insert(ch, Character {
            glyph: glyph,
            bitmap_glyph: bitmap_glyph,
            texture: texture,
        });
    }

}
