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
    pub fn pages(&self) -> impl Iterator<Item = &str> {
        self.pages.iter().map(|p| p.file.as_str())
    }

    pub fn parse(&self, s: &str) -> Result<Vec<CharPosition>, StringParseError> {
        let lines = self.parse_lines(s)?;
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
        Ok(char_positions)
    }

    fn find_kerning_values(&self, first_char_id: u32) -> Vec<&KerningValue> {
        self.kerning_values
            .iter()
            .filter(|k| k.first_char_id == first_char_id)
            .collect()
    }

    fn parse_lines(&self, s: &str) -> Result<Vec<Vec<&Char>>, StringParseError> {
        let mut lines = Vec::new();
        let mut line = Vec::new();
        let mut missing_characters = Vec::new();
        let mut unsupported_characters = Vec::new();
        for c in s.chars() {
            if c == '\n' {
                lines.push(line);
                line = Vec::new();
                continue;
            }
            if c.len_utf16() != 1 {
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
                missing_characters.push(c);
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        if missing_characters.is_empty() && unsupported_characters.is_empty() {
            Ok(lines)
        } else {
            Err(StringParseError {
                missing_characters,
                unsupported_characters,
            })
        }
    }
}
