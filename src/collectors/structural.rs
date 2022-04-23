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

/// load config from reader with specific format.
///
/// # Examples
///
/// ```no_run
/// use serde::Deserialize;
/// use serde::Serialize;
/// use serfig::Builder;
/// use serfig::collectors::from_reader;
/// use serfig::parsers::Toml;
///
/// #[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
/// #[serde(default)]
/// struct TestConfig {
///     a: String,
///     b: String,
///     c: i64,
/// }
///
/// fn main() -> anyhow::Result<()> {
/// # let r = "Hello, World!".as_bytes();
///     let builder = Builder::default()
///         .collect(from_reader(Toml, r));
///
///     let t: TestConfig = builder.build()?;
///
///     println!("{:?}", t);
///     Ok(())
/// }
/// ```
pub fn from_reader<V, R, P>(parser: P, r: R) -> Structural<V, R, P>
where
    V: DeserializeOwned + Serialize + Debug,
    R: io::Read,
    P: Parser,
{
    Structural {
        phantom: PhantomData::default(),
        reader: r,
        parser,
    }
}

/// load config from file path with specific format.
///
/// # Examples
///
/// ```no_run
/// use serde::Deserialize;
/// use serde::Serialize;
/// use serfig::Builder;
/// use serfig::collectors::from_file;
/// use serfig::parsers::Toml;
///
/// #[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
/// #[serde(default)]
/// struct TestConfig {
///     a: String,
///     b: String,
///     c: i64,
/// }
///
/// fn main() -> anyhow::Result<()> {
///     let builder = Builder::default()
///         .collect(from_file(Toml, "config.toml"));
///
///     let t: TestConfig = builder.build()?;
///
///     println!("{:?}", t);
///     Ok(())
/// }
/// ```
pub fn from_file<V, P>(parser: P, path: &str) -> Structural<V, LazyFileReader, P>
where
    V: DeserializeOwned + Serialize + Debug,
    P: Parser,
{
    Structural {
        phantom: PhantomData::default(),
        reader: LazyFileReader::new(path),
        parser,
    }
}

/// load config from string with specific format.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
/// use serde::Serialize;
/// use serfig::Builder;
/// use serfig::collectors::from_str;
/// use serfig::parsers::Toml;
///
/// #[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
/// #[serde(default)]
/// struct TestConfig {
///     a: String,
///     b: String,
///     c: i64,
/// }
///
/// fn main() -> anyhow::Result<()> {
///     let builder = Builder::default()
///         .collect(from_str(Toml, r#"a = "Hello, World!""#));
///
///     let t: TestConfig = builder.build()?;
///
///     println!("{:?}", t);
///     Ok(())
/// }
/// ```
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

/// Collector that load from a reader and than parsed by specified format.
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

pub struct LazyFileReader {
    path: String,
    r: Option<File>,
}

impl LazyFileReader {
    fn new(path: &str) -> LazyFileReader {
        LazyFileReader {
            path: path.to_string(),
            r: None,
        }
    }
}

impl io::Read for LazyFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.r {
            None => {
                let f = fs::File::open(&self.path)?;
                self.r = Some(f);
                self.read(buf)
            }
            Some(f) => f.read(buf),
        }
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
