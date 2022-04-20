use crate::Collector;
use crate::Parser;
use anyhow::Result;
use serde_bridge::Value;
use std::fmt::Debug;

#[derive(Debug)]
pub struct IntoCollect<T: Sized + Debug + AsRef<[u8]>, P: Parser> {
    value: T,
    parser: P,
}

pub trait IntoCollector<P: Parser>: Sized + Debug + AsRef<[u8]> {
    fn into_collector(self, parser: P) -> Box<dyn Collector>;
}

impl<P> IntoCollector<P> for String
where
    P: Parser,
{
    fn into_collector(self, parser: P) -> Box<dyn Collector> {
        Box::new(IntoCollect {
            value: self,
            parser,
        })
    }
}

impl<P> IntoCollector<P> for &'static str
where
    P: Parser,
{
    fn into_collector(self, parser: P) -> Box<dyn Collector> {
        Box::new(IntoCollect {
            value: self,
            parser,
        })
    }
}

impl<T, P> Collector for IntoCollect<T, P>
where
    T: Sized + Debug + AsRef<[u8]>,
    P: Parser,
{
    fn collect(&self) -> Result<Value> {
        let bs = self.value.as_ref();
        self.parser.parse(bs)
    }
}
