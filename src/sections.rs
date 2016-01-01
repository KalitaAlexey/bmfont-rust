use std::io::Read;
use super::{Error, ConfigParseError};

pub struct Sections {
    pub common_section: String,
    pub page_sections: Vec<String>,
    pub char_sections: Vec<String>,
    pub kerning_sections: Vec<String>,
}

impl Sections {
    pub fn new<R>(mut source: R) -> Result<Sections, Error>
        where R: Read
    {
        let mut content = String::new();
        try!(source.read_to_string(&mut content));
        let mut lines = content.lines();
        if !lines.next().map(|l| l.starts_with("info")).unwrap_or(false) {
            return Err(Error::from(ConfigParseError::MissingSection(String::from("info"))));
        }

        let common_section = match lines.next() {
            Some(line) if line.starts_with("common") => line.to_owned(),
            _ => return Err(Error::from(ConfigParseError::MissingSection(String::from("common"))))
        };

        let lines = lines.skip_while(|l| !l.starts_with("page"))
                         .collect::<Vec<_>>();
        let lines = lines.iter();
        let page_sections = lines.clone()
                                 .take_while(|l| l.starts_with("page"))
                                 .map(|s| s.to_string())
                                 .collect::<Vec<_>>();
        if page_sections.is_empty() {
            return Err(Error::from(ConfigParseError::MissingSection(String::from("page"))));
        }
        let mut lines = lines.skip(page_sections.len());
        let _ = lines.next().unwrap(); // char_count_section
        let char_sections = lines.clone()
                                 .take_while(|l| l.starts_with("char"))
                                 .map(|s| s.to_string())
                                 .collect::<Vec<_>>();
        if char_sections.is_empty() {
            return Err(Error::from(ConfigParseError::MissingSection(String::from("char"))));
        }
        let mut lines = lines.skip(char_sections.len());
        let _ = lines.next().unwrap(); // kerning_count_section
        let kerning_sections = lines.clone()
                                    .take_while(|l| l.starts_with("kerning"))
                                    .map(|s| s.to_string())
                                    .collect::<Vec<_>>();
        Ok(Sections {
            common_section: common_section,
            page_sections: page_sections,
            char_sections: char_sections,
            kerning_sections: kerning_sections,
        })
    }
}
