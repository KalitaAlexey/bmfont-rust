//! Parser for bitmap fonts

#![feature(unicode)]

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

use std::io::Read;
use self::char::Char;
use self::kerning_value::KerningValue;
use self::page::Page;
use self::sections::Sections;

pub struct CharPosition {
    pub page_rect: Rect,
    pub screen_rect: Rect,
    pub page_index: u32,
}

pub enum OrdinateOrientation {
    BottomToTop,
    TopToBottom,
}

pub struct BMFont {
    line_height: u32,
    characters: Vec<Char>,
    kerning_values: Vec<KerningValue>,
    pages: Vec<Page>,
    ordinate_orientation: OrdinateOrientation,
}

impl BMFont {
    pub fn new<R>(source: R, ordinate_orientation: OrdinateOrientation) -> Result<BMFont, Error>
        where R: Read
    {
        let sections = try!(Sections::new(source));
        let mut pages = Vec::new();
        for page_section in &sections.page_sections {
            pages.push(try!(Page::new(page_section)));
        }
        let mut characters = Vec::new();
        for char_section in &sections.char_sections {
            characters.push(try!(Char::new(char_section)));
        }
        let mut kerning_values = Vec::new();
        for kerning_section in &sections.kerning_sections {
            kerning_values.push(try!(KerningValue::new(kerning_section)));
        }
        Ok(BMFont {
            line_height: 80,
            characters: characters,
            kerning_values: kerning_values,
            pages: pages,
            ordinate_orientation: ordinate_orientation,
        })
    }

    pub fn line_height(&self) -> u32 {
        self.line_height
    }

    pub fn pages(&self) -> Vec<String> {
        self.pages.iter().map(|p| p.file.clone()).collect()
    }

    pub fn parse(&self, s: &str) -> Result<Vec<CharPosition>, StringParseError> {
        let lines = try!(self.parse_lines(s));
        let mut char_positions = Vec::new();
        let mut y: i32 = 0;
        for line in lines {
            let mut x: i32 = 0;
            let mut kerning_values: Vec<&KerningValue> = Vec::new();
            for character in line {
                let kerning_value = kerning_values.into_iter()
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
                        y + self.line_height as i32 - character.yoffset - character.height as i32
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
                    page_rect: page_rect,
                    screen_rect: screen_rect,
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
        self.kerning_values.iter().filter(|k| k.first_char_id == first_char_id).collect()
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
            let mut buffer: [u16; 4] = [0, 0, 0, 0];
            c.encode_utf16(&mut buffer).unwrap();
            let char_id = buffer[0] as u32;
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
                missing_characters: missing_characters,
                unsupported_characters: unsupported_characters,
            })
        }
    }
}
