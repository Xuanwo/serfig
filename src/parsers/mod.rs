//! Parsers will provide abstractions for parsing structural data like toml and json.

mod parser;
pub use parser::Parser;

mod toml;
pub use self::toml::Toml;
pub use self::toml::TomlIgnored;
