use std::fmt::Debug;

use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{IntoValue, Value};

use crate::collectors::collector::IntoCollector;
use crate::Collector;

/// load config from `Self`.
///
/// # Examples
///
/// ```no_run
/// use serde::Deserialize;
/// use serde::Serialize;
/// use serfig::Builder;
/// use serfig::collectors::from_self;
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
///         .collect(from_self(TestConfig::default()));
///
///     let t: TestConfig = builder.build()?;
///
///     println!("{:?}", t);
///     Ok(())
/// }
/// ```
pub fn from_self<V>(v: V) -> FromSelf<V>
where
    V: DeserializeOwned + Serialize + Debug,
{
    FromSelf(Some(v))
}

/// Collectors that can load configs from self.
pub struct FromSelf<V: DeserializeOwned + Serialize + Debug>(Option<V>);

impl<V> Collector<V> for FromSelf<V>
where
    V: DeserializeOwned + Serialize + Debug,
{
    fn collect(&mut self) -> Result<Value> {
        Ok(self.0.take().expect("contains valid value").into_value()?)
    }
}

impl<V> IntoCollector<V> for FromSelf<V>
where
    V: DeserializeOwned + Serialize + Debug + 'static,
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

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(rename = "serfig_test_str")]
        test_str: String,
    }

    #[test]
    fn test_from_self() {
        let _ = env_logger::try_init();

        let raw = TestStruct {
            test_str: "Hello, World!".to_string(),
        };

        let mut c = from_self(raw);

        let v = c.collect().expect("collect");
        debug!("value: {:?}", v);
        let t = TestStruct::from_value(v).expect("from value");

        assert_eq!(
            t,
            TestStruct {
                test_str: "Hello, World!".to_string()
            }
        )
    }
}
