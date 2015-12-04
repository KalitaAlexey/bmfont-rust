use std::fmt::Debug;
use std::str::FromStr;
use super::{ConfigParseError, utils};

const SECTION_NAME: &'static str = "char";

pub struct Char {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub xoffset: i32,
    pub yoffset: i32,
    pub xadvance: i32,
    pub page_index: u32,
}

impl Char {
    pub fn new(s: &str) -> Result<Char, ConfigParseError> {
        let mut components = s.split_whitespace();
        let section_name = components.next();
        assert_eq!(section_name.expect("Char cannot be created from empty string"),
                   SECTION_NAME);
        let id: u32 = try!(extract_component_value(components.next(), "id"));
        let x: u32 = try!(extract_component_value(components.next(), "x"));
        let y: u32 = try!(extract_component_value(components.next(), "y"));
        let width: u32 = try!(extract_component_value(components.next(), "width"));
        let height: u32 = try!(extract_component_value(components.next(), "height"));
        let xoffset: i32 = try!(extract_component_value(components.next(), "xoffset"));
        let yoffset: i32 = try!(extract_component_value(components.next(), "yoffset"));
        let xadvance: i32 = try!(extract_component_value(components.next(), "xadvance"));
        let page_index: u32 = try!(extract_component_value(components.next(), "page"));
        Ok(Char {
            id: id,
            x: x,
            y: y,
            width: width,
            height: height,
            xoffset: xoffset,
            yoffset: yoffset,
            xadvance: xadvance,
            page_index: page_index,
        })
    }
}

fn extract_component_value<T>(s: Option<&str>, component: &str) -> Result<T, ConfigParseError>
    where T: FromStr,
            T::Err: Debug
{
    utils::extract_component_value(s, SECTION_NAME, component)
}