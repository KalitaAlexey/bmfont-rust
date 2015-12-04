#[derive(Debug)]
pub struct StringParseError {
    pub missing_characters: Vec<char>,
    pub unsupported_characters: Vec<char>,
}
