use super::{utils, ConfigParseError};
use std::fmt::Debug;
use std::str::FromStr;

const SECTION_NAME: &str = "page";

#[derive(Clone, Debug)]
pub struct Page {
    pub id: u32,
    pub file: String,
}

impl Page {
    pub fn new(s: &str) -> Result<Page, ConfigParseError> {
        let mut components = s.split_whitespace();
        let section_name = components.next();
        assert_eq!(
            section_name.expect("Page cannot be created from empty string"),
            SECTION_NAME
        );
        let id: u32 = extract_component_value(components.next(), "id")?;
        let file: String = extract_component_value(components.next(), "file")?;
        let file = file.trim_matches('"').to_string();
        Ok(Page { id, file })
    }
}

fn extract_component_value<T>(s: Option<&str>, component: &str) -> Result<T, ConfigParseError>
where
    T: FromStr,
    T::Err: Debug,
{
    utils::extract_component_value(s, SECTION_NAME, component)
}
