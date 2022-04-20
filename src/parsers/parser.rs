use anyhow::Result;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub trait Parser: Debug + 'static {
    fn parse<T: DeserializeOwned>(&self, bs: &[u8]) -> Result<T>;
}
