use super::{ConfigParseError, Error};
use std::io::Read;

#[derive(Clone, Debug)]
pub struct Sections {
    pub common_section: String,
    pub page_sections: Vec<String>,
    pub char_sections: Vec<String>,
    pub kerning_sections: Vec<String>,
}

impl Sections {
    pub fn new<R>(mut source: R) -> Result<Sections, Error>
    where
        R: Read,
    {
        // Load the entire file into a String.
        let mut content = String::new();
        source.read_to_string(&mut content)?;

        // Expect the "info" section.
        let mut lines = content.lines();
        if !lines.next().map(|l| l.starts_with("info")).unwrap_or(false) {
            return Err(Error::from(ConfigParseError::MissingSection(String::from(
                "info",
            ))));
        }

        // Expect the "common" section.
        let common_section = match lines.next() {
            Some(line) if line.starts_with("common") => line.to_owned(),
            _ => {
                return Err(Error::from(ConfigParseError::MissingSection(String::from(
                    "common",
                ))))
            }
        };

        // Expect the "page" sections.
        let lines = lines
            .skip_while(|l| !l.starts_with("page"))
            .collect::<Vec<_>>();
        let lines = lines.iter();
        let page_sections = lines
            .clone()
            .take_while(|l| l.starts_with("page"))
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        if page_sections.is_empty() {
            return Err(Error::from(ConfigParseError::MissingSection(String::from(
                "page",
            ))));
        }
        let mut lines = lines.skip(page_sections.len());

        // Expect the "char" sections.
        let _ = lines.next().unwrap(); // char_count_section
        let char_sections = lines
            .clone()
            .take_while(|l| l.starts_with("char"))
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        if char_sections.is_empty() {
            return Err(Error::from(ConfigParseError::MissingSection(String::from(
                "char",
            ))));
        }
        let mut lines = lines.skip(char_sections.len());

        // Expect the "kerning" sections.
        let kerning_sections = if lines.next().is_some() {
            lines
                .clone()
                .take_while(|l| l.starts_with("kerning"))
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        Ok(Sections {
            common_section,
            page_sections,
            char_sections,
            kerning_sections,
        })
    }
}
