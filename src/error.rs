use std::error::Error;
#[derive(Debug, Clone, Copy)]
pub struct ElementNotFound;

impl std::fmt::Display for ElementNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Not Found Element")
    }
}

impl Error for ElementNotFound {}
