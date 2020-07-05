use super::ConfigParseError;
use std::fmt::Debug;
use std::str::FromStr;

pub fn extract_component_value<T>(
    s: Option<&str>,
    section: &str,
    component: &str,
) -> Result<T, ConfigParseError>
where
    T: FromStr,
    T::Err: Debug,
{
    let s = unwrap_string(s, section, component)?;
    let string_parts = s.splitn(2, '=').collect::<Vec<_>>();
    check_string_parts(&string_parts, section, component)?;
    let value = extract_value(&string_parts, section, component)?;
    if let Ok(value) = value.parse() {
        Ok(value)
    } else {
        Err(ConfigParseError::InvalidComponentValue {
            section: section.to_string(),
            component: component.to_string(),
            value: value.to_string(),
        })
    }
}

fn unwrap_string<'a>(
    s: Option<&'a str>,
    section: &'a str,
    component: &'a str,
) -> Result<&'a str, ConfigParseError> {
    if let Some(s) = s {
        Ok(s)
    } else {
        Err(ConfigParseError::MissingComponent {
            section: String::from(section),
            component: String::from(component),
        })
    }
}

fn check_string_parts(
    string_parts: &[&str],
    section: &str,
    component: &str,
) -> Result<(), ConfigParseError> {
    if string_parts.len() == 2 {
        return Ok(());
    }
    assert_eq!(string_parts.len(), 1);
    Err(ConfigParseError::InvalidComponentValue {
        section: section.to_string(),
        component: component.to_string(),
        value: String::new(),
    })
}

fn extract_value<'a>(
    string_parts: &'a [&'a str],
    section: &str,
    component: &str,
) -> Result<&'a str, ConfigParseError> {
    let actual_component = string_parts[0];
    if actual_component == component {
        return Ok(string_parts[1]);
    }
    Err(ConfigParseError::InvalidComponent {
        section: String::from(section),
        expected_component: String::from(component),
        actual_component: String::from(actual_component),
    })
}
