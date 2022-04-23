use std::fmt::Debug;
use std::fs::File;
use std::marker::PhantomData;
use std::{fs, io};

use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{IntoValue, Value};

use crate::collectors::collector::IntoCollector;
use crate::{Collector, Parser};

pub fn from_reader<V, R, P>(parser: P, r: R) -> Result<Structural<V, R, P>>
where
    V: DeserializeOwned + Serialize + Debug,
    R: io::Read,
    P: Parser,
{
    Ok(Structural {
        phantom: PhantomData::default(),
        reader: r,
        parser,
    })
}

pub fn from_file<V, P>(parser: P, path: &str) -> Result<Structural<V, File, P>>
where
    V: DeserializeOwned + Serialize + Debug,
    P: Parser,
{
    let f = fs::File::open(path)?;

    Ok(Structural {
        phantom: PhantomData::default(),
        reader: f,
        parser,
    })
}

pub fn from_str<V, P>(parser: P, s: &str) -> Structural<V, &[u8], P>
where
    V: DeserializeOwned + Serialize + Debug,
    P: Parser,
{
    Structural {
        phantom: PhantomData::default(),
        reader: s.as_bytes(),
        parser,
    }
}

pub struct Structural<V: DeserializeOwned + Serialize + Debug, R: io::Read, P: Parser> {
    phantom: PhantomData<V>,
    reader: R,
    parser: P,
}

impl<V, R, P> Collector<V> for Structural<V, R, P>
where
    V: DeserializeOwned + Serialize + Debug,
    R: io::Read,
    P: Parser,
{
    fn collect(&mut self) -> Result<Value> {
        let mut bs = Vec::new();
        let _ = self.reader.read_to_end(&mut bs)?;

        let v: V = self.parser.parse(&bs)?;
        Ok(v.into_value()?)
    }
}

impl<V, R, P> IntoCollector<V> for Structural<V, R, P>
where
    V: DeserializeOwned + Serialize + Debug + 'static,
    R: io::Read + 'static,
    P: Parser + 'static,
{
    fn into_collector(self) -> Box<dyn Collector<V>> {
        Box::new(self)
    }
}

#[cfg(test)]
mod tests {
    use log::debug;
    use serde::{Deserialize, Serialize};
    use serde_bridge::FromValue;

    use super::*;
    use crate::parsers::Toml;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(rename = "serfig_test_str")]
        test_str: String,
    }

    #[test]
    fn test_from_str() {
        let _ = env_logger::try_init();

        let mut c: Structural<TestStruct, &[u8], Toml> =
            from_str(Toml, r#"serfig_test_str = "test_str""#);

        let v = c.collect().expect("must success");
        debug!("value: {:?}", v);

        let t = TestStruct::from_value(v).expect("from value");

        assert_eq!(
            t,
            TestStruct {
                test_str: "test_str".to_string()
            }
        )
    }
}
