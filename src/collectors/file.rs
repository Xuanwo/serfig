use crate::{Collector, Parser};
use anyhow::Result;
use serde_bridge::Value;
use std::fs;

pub struct File<P: Parser> {
    path: String,
    parser: P,
}

impl<P> File<P>
where
    P: Parser,
{
    pub fn new(path: &str, parser: P) -> Self {
        Self {
            path: path.to_string(),
            parser,
        }
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
