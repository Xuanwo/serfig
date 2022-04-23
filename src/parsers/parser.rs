use anyhow::Result;
use serde::de::DeserializeOwned;

pub trait Parser {
    fn parse<T: DeserializeOwned>(&mut self, bs: &[u8]) -> Result<T>;
}
