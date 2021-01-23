use super::{utils, ConfigParseError};
use std::fmt::Debug;
use std::str::FromStr;

const SECTION_NAME: &str = "kerning";

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_json", derive(Eq, PartialEq))]
pub struct KerningValue {
    pub first_char_id: u32,
    pub second_char_id: u32,
    pub value: i32,
}

impl KerningValue {
    pub fn new(s: &str) -> Result<KerningValue, ConfigParseError> {
        let mut components = s.split_whitespace();
        let section_name = components.next();
        assert_eq!(
            section_name.expect("Kerning value cannot be created from empty string"),
            SECTION_NAME
        );
        let first_char_id: u32 = extract_component_value(components.next(), "first")?;
        let second_char_id: u32 = extract_component_value(components.next(), "second")?;
        let value: i32 = extract_component_value(components.next(), "amount")?;
        Ok(KerningValue {
            first_char_id,
            second_char_id,
            value,
        })
    }
}

fn extract_component_value<T>(s: Option<&str>, component: &str) -> Result<T, ConfigParseError>
where
    T: FromStr,
    T::Err: Debug,
{
    utils::extract_component_value(s, SECTION_NAME, component)
}
