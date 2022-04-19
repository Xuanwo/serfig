use crate::Collector;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Deserialize;

#[derive(Default)]
struct Config {
    collectors: Vec<Box<dyn Collector>>,
}

impl Config {
    pub fn new() -> Config {
        Self::default()
    }

    pub fn with_collector(mut self, c: Box<dyn Collector>) -> Self {
        self.collectors.push(c);
        Self {
            collectors: self.collectors,
        }
    }

    pub fn build<T: DeserializeOwned>(self) -> Result<T> {
        todo!()
    }
}
