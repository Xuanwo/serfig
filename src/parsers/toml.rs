use anyhow::Result;
use serde::de::DeserializeOwned;

use crate::Parser;

/// Toml format support
#[derive(Debug)]
pub struct Toml;

impl Parser for Toml {
    fn parse<T: DeserializeOwned>(&mut self, bs: &[u8]) -> Result<T> {
        Ok(toml::from_slice(bs)?)
    }
}
