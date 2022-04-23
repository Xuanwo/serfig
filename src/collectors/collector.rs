use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{Value};


pub trait Collector<V: DeserializeOwned + Serialize> {
    fn collect(&mut self) -> Result<Value>;
}

pub trait IntoCollector<V: DeserializeOwned + Serialize> {
    fn into_collector(self) -> Box<dyn Collector<V>>;
}
