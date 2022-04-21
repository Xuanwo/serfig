use anyhow::Result;
use serde_bridge::Value;
use std::fmt::Debug;

pub trait Collector: Debug {
    fn collect(&self) -> Result<Value>;
}

impl Collector for Value {
    fn collect(&self) -> Result<Value> {
        Ok(self.clone())
    }
}
