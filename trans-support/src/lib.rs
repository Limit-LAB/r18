pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub mod source;
pub mod translation;
