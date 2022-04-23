use anyhow::Result;
use serde::de::DeserializeOwned;

/// Parse input bytes into specified type `T`.
pub trait Parser {
    fn parse<T: DeserializeOwned>(&mut self, bs: &[u8]) -> Result<T>;
}
