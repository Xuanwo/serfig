use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{FromValue, IntoValue, Value};
use std::fmt::Debug;

pub trait Collector<V: DeserializeOwned + Serialize>: Debug {
    fn collect(&self) -> Result<Value>;
}

impl<V> Collector<V> for Value
where
    V: DeserializeOwned + Serialize,
{
    fn collect(&self) -> Result<Value> {
        let t = V::from_value(self.clone())?;

        Ok(t.into_value()?)
    }
}
