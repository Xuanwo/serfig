use anyhow::Result;
use serde_bridge::Value;

pub trait Collector: 'static {
    fn collect(&self) -> Result<Value>;
}
