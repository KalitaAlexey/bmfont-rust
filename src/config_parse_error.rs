use std::fmt::{Display, Error, Formatter};

#[derive(Debug)]
pub enum ConfigParseError {
    MissingSection(String),
    MissingComponent {
        section: String,
        component: String,
    },
    InvalidComponent {
        section: String,
        expected_component: String,
        actual_component: String,
    },
    InvalidComponentValue {
        section: String,
        component: String,
        value: String,
    },
}

impl Display for ConfigParseError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        write!(formatter, "Parse error. ")?;
        match *self {
            ConfigParseError::MissingSection(ref section) => {
                write!(formatter, "Missing section = {}", section)
            }
            ConfigParseError::MissingComponent {
                ref section,
                ref component,
            } => write!(
                formatter,
                "Missing component = {} in section = {}",
                component, section
            ),
            ConfigParseError::InvalidComponent {
                ref section,
                ref expected_component,
                ref actual_component,
            } => write!(
                formatter,
                "Invalid component = {} in section = {}. Expected = {}",
                actual_component, section, expected_component
            ),
            ConfigParseError::InvalidComponentValue {
                ref section,
                ref component,
                ref value,
            } => write!(
                formatter,
                "Invalid component value = {} for component = {} in section = {}",
                value, component, section
            ),
        }
    }
}
