use anyhow::Result;
use serde_bridge::Value;

pub trait Collector {
    fn collect(&self) -> Result<Value>;
}
