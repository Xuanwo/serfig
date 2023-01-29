use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;

use crate::Parser;

/// Toml format support
#[derive(Debug)]
pub struct Toml;

impl Parser for Toml {
    fn parse<T: DeserializeOwned>(&mut self, bs: &[u8]) -> Result<T> {
        let s = std::str::from_utf8(bs)
            .map_err(|err| anyhow!("input value is not valid utf-8: {err:?}"))?;
        Ok(toml::from_str(s)?)
    }
}
