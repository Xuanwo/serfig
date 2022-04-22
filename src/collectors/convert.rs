use crate::Collector;
use crate::Parser;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{IntoValue, Value};
use std::fmt::Debug;

#[derive(Debug)]
pub struct IntoCollect<T: Sized + Debug + AsRef<[u8]>, P: Parser> {
    value: T,
    parser: P,
}

pub trait IntoCollector<V: DeserializeOwned + Serialize, P: Parser>:
    Sized + Debug + AsRef<[u8]>
{
    fn into_collector(self, parser: P) -> Box<dyn Collector<V>>;
}

impl<V, P> IntoCollector<V, P> for String
where
    V: DeserializeOwned + Serialize,
    P: Parser,
{
    fn into_collector(self, parser: P) -> Box<dyn Collector<V>> {
        Box::new(IntoCollect {
            value: self,
            parser,
        })
    }
}

impl<V, P> IntoCollector<V, P> for &'static str
where
    V: DeserializeOwned + Serialize,
    P: Parser,
{
    fn into_collector(self, parser: P) -> Box<dyn Collector<V>> {
        Box::new(IntoCollect {
            value: self,
            parser,
        })
    }
}

impl<V, T, P> Collector<V> for IntoCollect<T, P>
where
    V: DeserializeOwned + Serialize,
    T: Sized + Debug + AsRef<[u8]>,
    P: Parser,
{
    fn collect(&self) -> Result<Value> {
        let bs = self.value.as_ref();
        let v: V = self.parser.parse(bs)?;
        Ok(v.into_value()?)
    }
}
