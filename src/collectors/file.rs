use crate::{Collector, Parser};
use anyhow::Result;
use serde_bridge::Value;
use std::fs;

#[derive(Debug)]
pub struct File<P: Parser> {
    path: String,
    parser: P,
}

impl<P> File<P>
where
    P: Parser,
{
    pub fn create(path: &str, parser: P) -> Box<dyn Collector> {
        Box::new(Self {
            path: path.to_string(),
            parser,
        })
    }
}

impl<P> Collector for File<P>
where
    P: Parser,
{
    fn collect(&self) -> Result<Value> {
        let bs = fs::read(&self.path)?;

        self.parser.parse(&bs)
    }
}
