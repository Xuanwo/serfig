use anyhow::Result;
use log::debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{IntoValue, Value};
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::Collector;

#[derive(Debug)]
pub struct Environment<V: DeserializeOwned + Serialize + Debug + 'static> {
    phantom: PhantomData<V>,
}

impl<V> Environment<V>
where
    V: DeserializeOwned + Serialize + Debug + 'static,
{
    pub fn create() -> Box<dyn Collector<V>> {
        Box::new(Self {
            phantom: PhantomData::default(),
        })
    }
}

impl<V> Collector<V> for Environment<V>
where
    V: DeserializeOwned + Serialize + Debug,
{
    fn collect(&self) -> Result<Value> {
        let v: V = serde_env::from_env()?;
        debug!("value parsed from env: {:?}", v);
        Ok(v.into_value()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;
    use serde::Deserialize;
    use serde::Serialize;
    use serde_bridge::FromValue;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(rename = "serfig_test_str")]
        test_str: String,
    }

    #[test]
    fn test_env() {
        let _ = env_logger::try_init();

        temp_env::with_vars(vec![("serfig_test_str", Some("test_str"))], || {
            let v = Environment::<TestStruct>::create()
                .collect()
                .expect("must success");

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
