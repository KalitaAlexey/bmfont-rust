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

        let mut pages = Vec::new();
        for page_section in &sections.page_sections {
            pages.push(Page::new(page_section)?);
        }
        let mut characters = Vec::new();
        for char_section in &sections.char_sections {
            characters.push(Char::new(char_section)?);
        }
        let mut kerning_values = Vec::new();
        for kerning_section in &sections.kerning_sections {
            kerning_values.push(KerningValue::new(kerning_section)?);
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
            let mut kerning_values: Vec<&KerningValue> = Vec::new();
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

    fn find_kerning_values(&self, first_char_id: u32) -> Vec<&KerningValue> {
        self.kerning_values
            .iter()
            .filter(|k| k.first_char_id == first_char_id)
            .collect()
    }

    fn parse_lines(&self, s: &str) -> ParseLines {
        let mut lines = Vec::new();
        let mut line = Vec::new();

        #[cfg(feature = "parse-error")]
        let mut missing_characters = Vec::new();

        #[cfg(feature = "parse-error")]
        let mut unsupported_characters = Vec::new();

        for c in s.chars() {
            if c == '\n' {
                lines.push(line);
                line = Vec::new();
                continue;
            }
            if c.len_utf16() != 1 {
                #[cfg(feature = "parse-error")]
                unsupported_characters.push(c);

                continue;
            }
            let tmp_str = {
                let mut t = String::new();
                t.push(c);
                t
            };
            let char_id = tmp_str.encode_utf16().next().unwrap() as u32;
            if let Some(c) = self.characters.iter().find(|c| c.id == char_id) {
                line.push(c);
            } else {
                #[cfg(feature = "parse-error")]
                {
                    missing_characters.push(c);
                }
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }

        #[cfg(feature = "parse-error")]
        if missing_characters.is_empty() && unsupported_characters.is_empty() {
            Ok(lines)
        } else {
            Err(StringParseError {
                missing_characters,
                unsupported_characters,
            })
        }

        #[cfg(not(feature = "parse-error"))]
        lines
    }
}

#[derive(Clone, Debug)]
pub struct PageIter<'a> {
    idx: usize,
    pages: &'a Vec<Page>,
}

impl<'a> PageIter<'a> {
    fn new(pages: &'a Vec<Page>) -> Self {
        Self {
            idx: 0,
            pages: &*pages,
        }
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
