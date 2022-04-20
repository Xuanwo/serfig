use crate::Parser;
use anyhow::Result;
use serde::de::DeserializeOwned;

pub struct Toml;

impl Parser for Toml {
    fn parse<T: DeserializeOwned>(&self, bs: &[u8]) -> Result<T> {
        Ok(toml::from_slice(bs)?)
    }
}
