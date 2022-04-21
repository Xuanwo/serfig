use crate::{Collector, Parser};
use anyhow::Result;
use log::debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{IntoValue, Value};
use std::fmt::Debug;
use std::fs;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct File<V: DeserializeOwned + Serialize + Debug + 'static, P: Parser> {
    path: String,
    parser: P,
    phantom: PhantomData<V>,
}

impl<V, P> File<V, P>
where
    V: DeserializeOwned + Serialize + Debug + 'static,
    P: Parser,
{
    pub fn create(path: &str, parser: P) -> Box<dyn Collector<V>> {
        Box::new(Self {
            path: path.to_string(),
            parser,
            phantom: PhantomData::default(),
        })
    }
}

impl<V, P> Collector<V> for File<V, P>
where
    V: DeserializeOwned + Serialize + Debug,
    P: Parser,
{
    fn collect(&self) -> Result<Value> {
        let bs = fs::read(&self.path)?;

        let v: V = self.parser.parse(&bs)?;
        debug!("parsed value: {:?}", v);
        Ok(v.into_value()?)
    }
}
