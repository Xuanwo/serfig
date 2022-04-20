use anyhow::Result;
use serde::de::DeserializeOwned;

pub trait Parser: 'static {
    fn parse<T: DeserializeOwned>(&self, bs: &[u8]) -> Result<T>;
}
