use std::fmt::Debug;
use std::marker::PhantomData;

use anyhow::Result;
use log::debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{IntoValue, Value};

use crate::collectors::collector::IntoCollector;
use crate::Collector;

/// load config from env.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
/// use serde::Serialize;
/// use serfig::Builder;
/// use serfig::collectors::from_env;
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
///         .collect(from_env());
///     let t: TestConfig = builder.build()?;
///
///     println!("{:?}", t);
///     Ok(())
/// }
/// ```
pub fn from_env<V>() -> Environment<V>
where
    V: DeserializeOwned + Serialize + Debug,
{
    Environment {
        phantom: PhantomData::default(),
    }
}

/// Collector that can load config from env.
///
/// Created by [`from_env`].
#[derive(Debug)]
pub struct Environment<V: DeserializeOwned + Serialize + Debug> {
    phantom: PhantomData<V>,
}

impl<V> Collector<V> for Environment<V>
where
    V: DeserializeOwned + Serialize + Debug,
{
    fn collect(&mut self) -> Result<Value> {
        let v: V = serde_env::from_env()?;
        debug!("value parsed from env: {:?}", v);
        Ok(v.into_value()?)
    }
}

impl<V> IntoCollector<V> for Environment<V>
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
    fn test_env() {
        let _ = env_logger::try_init();

        temp_env::with_vars(vec![("serfig_test_str", Some("test_str"))], || {
            let mut c: Environment<TestStruct> = from_env();

            let v = c.collect().expect("must success");

            debug!("value: {:?}", v);
            let t = TestStruct::from_value(v).expect("must success");

            assert_eq!(
                t,
                TestStruct {
                    test_str: "test_str".to_string()
                }
            )
        })
    }
}
