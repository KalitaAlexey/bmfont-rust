//! Parser for bitmap fonts

mod char;
mod config_parse_error;
mod error;
mod kerning_value;
mod page;
mod rect;
mod sections;
mod string_parse_error;
mod utils;

pub use self::config_parse_error::ConfigParseError;
pub use self::error::Error;
pub use self::rect::Rect;
pub use self::string_parse_error::StringParseError;

use self::char::Char;
use self::kerning_value::KerningValue;
use self::page::Page;
use self::sections::Sections;
use std::io::Read;

/// Alias of either [`Result<Vec<CharPosition>, StringParseError>`] _or_ [`Vec<CharPosition>`],
/// returned by [`BMFont::parse()`].
///
/// The output type depends on the value of the `parse-error` package feature.
///
/// **_NOTE:_** This documentation was generated _with_ the `parse-error` feature.
#[cfg(feature = "parse-error")]
pub type Parse<'a> = Result<Vec<CharPosition>, StringParseError>;

/// Alias of either [`Result<Vec<CharPosition>, StringParseError>`] _or_ [`Vec<CharPosition>`],
/// returned by [`BMFont::parse()`].
///
/// The output type depends on the value of the `parse-error` package feature.
///
/// **_NOTE:_** This documentation was generated _without_ the `parse-error` feature.
#[cfg(not(feature = "parse-error"))]
pub type Parse<'a> = Vec<CharPosition>;

#[cfg(feature = "parse-error")]
type ParseLines<'a> = Result<Vec<Vec<&'a Char>>, StringParseError>;

#[cfg(not(feature = "parse-error"))]
type ParseLines<'a> = Vec<Vec<&'a Char>>;

#[derive(Clone, Debug)]
pub struct CharPosition {
    pub page_rect: Rect,
    pub screen_rect: Rect,
    pub page_index: u32,
}

#[derive(Clone, Debug)]
pub enum OrdinateOrientation {
    BottomToTop,
    TopToBottom,
}

/// Holds a decoded bitmap font defintion, including all character advance and kerning values.
#[derive(Clone, Debug)]
pub struct BMFont {
    base_height: u32,
    line_height: u32,
    characters: Vec<Char>,
    kerning_values: Vec<KerningValue>,
    pages: Vec<Page>,
    ordinate_orientation: OrdinateOrientation,
}

impl BMFont {
    /// Constructs a new [BMFont].
    ///
    /// ## Examples
    ///
    /// From a file:
    ///
    /// ```rust
    /// # use bmfont::*;
    /// # fn main() -> Result<(), Error> {
    /// let file = std::fs::File::open("font.fnt")?;
    /// let font = BMFont::new(file, OrdinateOrientation::TopToBottom)?;
    /// assert_eq!(font.line_height(), 80);
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// From a slice of bytes:
    ///
    /// ```rust
    /// # use bmfont::*;
    /// # fn main() -> Result<(), Error> {
    /// # let my_font_bytes = std::fs::read("font.fnt")?;
    /// let data = std::io::Cursor::new(my_font_bytes);
    /// let font = BMFont::new(data, OrdinateOrientation::TopToBottom)?;
    /// assert_eq!(font.line_height(), 80);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn new<R>(source: R, ordinate_orientation: OrdinateOrientation) -> Result<BMFont, Error>
    where
        R: Read,
    {
        let sections = Sections::new(source)?;

        let base_height;
        let line_height;
        {
            let mut components = sections.common_section.split_whitespace();
            components.next();
            line_height =
                utils::extract_component_value(components.next(), "common", "lineHeight")?;
            base_height = utils::extract_component_value(components.next(), "common", "base")?;
        }

        let mut pages = Vec::with_capacity(sections.page_sections.len());
        for page_section in &sections.page_sections {
            pages.push(Page::new(page_section)?);
        }

        // Sort the characters while loading them so that lookup can be faster during parse
        let mut characters: Vec<Char> = Vec::with_capacity(sections.char_sections.len());
        for char_section in &sections.char_sections {
            let char = Char::new(char_section)?;
            if let Err(idx) = characters.binary_search_by(|probe| probe.id.cmp(&char.id)) {
                characters.insert(idx, char);
            }
        }

        // Also sort kerning values for the same reason, but we allow duplicates
        let mut kerning_values: Vec<KerningValue> =
            Vec::with_capacity(sections.kerning_sections.len());
        for kerning_section in &sections.kerning_sections {
            let kerning = KerningValue::new(kerning_section)?;

            match kerning_values
                .binary_search_by(|probe| probe.first_char_id.cmp(&kerning.first_char_id))
            {
                Err(idx) | Ok(idx) => kerning_values.insert(idx, kerning),
            }
        }

        Ok(BMFont {
            base_height,
            line_height,
            characters,
            kerning_values,
            pages,
            ordinate_orientation,
        })
    }

    /// Returns the height of a `EM` in pixels.
    pub fn base_height(&self) -> u32 {
        self.base_height
    }

    pub fn line_height(&self) -> u32 {
        self.line_height
    }

    /// Returns an `Iterator` of font page bitmap filenames.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use bmfont::*;
    /// # fn main() -> Result<(), Error> {
    /// let file = std::fs::File::open("font.fnt")?;
    /// let font = BMFont::new(file, OrdinateOrientation::TopToBottom)?;
    /// assert_eq!(font.pages().next(), Some("font.png"));
    /// #     Ok(())
    /// # }
    /// ```
    pub fn pages(&self) -> PageIter {
        PageIter::new(&self.pages)
    }

    pub fn parse(&self, s: &str) -> Parse {
        let lines = self.parse_lines(s);

        #[cfg(feature = "parse-error")]
        let lines = lines?;

        let mut char_positions = Vec::new();
        let mut y: i32 = 0;
        for line in lines {
            let mut x: i32 = 0;
            let mut kerning_values = KerningIter::empty(&self.kerning_values);
            for character in line {
                let kerning_value = kerning_values
                    .into_iter()
                    .find(|k| k.second_char_id == character.id)
                    .map(|k| k.value)
                    .unwrap_or(0);
                let page_rect = Rect {
                    x: character.x as i32,
                    y: character.y as i32,
                    width: character.width,
                    height: character.height,
                };
                let screen_x = x + character.xoffset + kerning_value;
                let screen_y = match self.ordinate_orientation {
                    OrdinateOrientation::BottomToTop => {
                        y + self.base_height as i32 - character.yoffset - character.height as i32
                    }
                    OrdinateOrientation::TopToBottom => y + character.yoffset,
                };
                let screen_rect = Rect {
                    x: screen_x,
                    y: screen_y,
                    width: character.width,
                    height: character.height,
                };
                let char_position = CharPosition {
                    page_rect,
                    screen_rect,
                    page_index: character.page_index,
                };
                char_positions.push(char_position);
                x += character.xadvance + kerning_value;
                kerning_values = self.find_kerning_values(character.id);
            }
            match self.ordinate_orientation {
                OrdinateOrientation::TopToBottom => y += self.line_height as i32,
                OrdinateOrientation::BottomToTop => y -= self.line_height as i32,
            }
        }

        #[cfg(feature = "parse-error")]
        {
            Ok(char_positions)
        }

        #[cfg(not(feature = "parse-error"))]
        {
            char_positions
        }
    }

    fn find_kerning_values(&self, first_char_id: u32) -> KerningIter {
        let needle = (first_char_id << 1) - 1;
        let idx = self
            .kerning_values
            .binary_search_by(|probe| (probe.first_char_id << 1).cmp(&needle))
            .unwrap_err();

        KerningIter {
            first_char_id,
            idx,
            values: &self.kerning_values,
        }
    }

    fn parse_lines(&self, s: &str) -> ParseLines {
        let mut lines = Vec::new();
        let mut line = Vec::new();
        let mut temp = [0u16; 2];

        #[cfg(feature = "parse-error")]
        {
            let mut missing_characters: Option<Vec<char>> = None;
            let mut unsupported_characters: Option<Vec<char>> = None;

            for c in s.chars() {
                if c.len_utf16() != 1 {
                    if let Some(vec) = unsupported_characters.as_mut() {
                        vec.push(c);
                    } else {
                        unsupported_characters = Some(vec![c]);
                    }

                    continue;
                }

                c.encode_utf16(&mut temp);
                let char_id = temp[0] as u32;

                if self
                    .characters
                    .binary_search_by(|probe| probe.id.cmp(&char_id))
                    .is_ok()
                {
                    continue;
                }

                if let Some(vec) = missing_characters.as_mut() {
                    vec.push(c);
                } else {
                    missing_characters = Some(vec![c]);
                }
            }

            if missing_characters.is_some() || unsupported_characters.is_some() {
                return Err(StringParseError {
                    missing_characters: missing_characters.unwrap_or_default(),
                    unsupported_characters: unsupported_characters.unwrap_or_default(),
                });
            }
        }

        for c in s.chars() {
            #[cfg(not(feature = "parse-error"))]
            if c.len_utf16() != 1 {
                continue;
            }

            if c == '\n' {
                lines.push(line);
                line = Vec::new();
                continue;
            }

            c.encode_utf16(&mut temp);
            let char_id = temp[0] as u32;

            let char_idx = self
                .characters
                .binary_search_by(|probe| probe.id.cmp(&char_id));

            #[cfg(not(feature = "parse-error"))]
            if char_idx.is_err() {
                continue;
            }

            let char_idx = char_idx.unwrap();
            line.push(&self.characters[char_idx]);
        }

        if !line.is_empty() {
            lines.push(line);
        }

        #[cfg(feature = "parse-error")]
        {
            Ok(lines)
        }

        #[cfg(not(feature = "parse-error"))]
        lines
    }
}

struct KerningIter<'a> {
    first_char_id: u32,
    idx: usize,
    values: &'a Vec<KerningValue>,
}

impl<'a> KerningIter<'a> {
    fn empty(values: &'a Vec<KerningValue>) -> Self {
        Self {
            first_char_id: 0,
            idx: values.len(),
            values,
        }
    }
}

impl<'a> Iterator for KerningIter<'a> {
    type Item = &'a KerningValue;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.values.get(self.idx) {
            if value.first_char_id == self.first_char_id {
                self.idx += 1;

                return Some(value);
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct PageIter<'a> {
    idx: usize,
    pages: &'a Vec<Page>,
}

impl<'a> PageIter<'a> {
    fn new(pages: &'a Vec<Page>) -> Self {
        Self { idx: 0, pages }
    }
}

impl<'a> Iterator for PageIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(page) = self.pages.get(self.idx) {
            self.idx += 1;
            Some(page.file.as_str())
        } else {
            None
        }
    }
}
