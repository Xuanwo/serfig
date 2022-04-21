use anyhow::{anyhow, Result};
use log::{debug, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{FromValue, Value};

use crate::collectors::Collector;
use crate::value::merge;

#[derive(Default)]
pub struct Builder<V: DeserializeOwned + Serialize> {
    collectors: Vec<Box<dyn Collector<V>>>,
}

impl<V> Builder<V>
where
    V: DeserializeOwned + Serialize,
{
    pub fn new() -> Builder<V> {
        Self {
            collectors: Vec::new(),
        }
    }

    pub fn collect(mut self, c: Box<dyn Collector<V>>) -> Self {
        self.collectors.push(c);
        Self {
            collectors: self.collectors,
        }
    }

    pub fn build(self) -> Result<V> {
        let mut result = None;
        let mut value = Value::Unit;
        for c in self.collectors {
            // Merge value if we collect successfully.
            value = match c.collect() {
                Ok(v) => merge(value, v),
                Err(e) => {
                    warn!("collect from {:?}: {:?}", c, e);
                    continue;
                }
            };
            debug!("we got value: {:?}", value);
            // Re-deserialize the value if we from_value correctly.
            result = match V::from_value(value.clone()) {
                Ok(v) => Some(v),
                Err(e) => {
                    warn!("deserialize value {:?}: {:?}", value, e);
                    continue;
                }
            }
        }

        result.ok_or_else(|| anyhow!("no valid value to deserialize",))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::{Environment, IntoCollector};
    use crate::parsers::Toml;
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
    #[serde(default)]
    struct TestConfig {
        test_a: String,
        test_b: String,
    }

    #[test]
    fn test_build() -> Result<()> {
        temp_env::with_vars(
            vec![("test_a", Some("test_a")), ("test_b", Some("test_b"))],
            || {
                let cfg = Builder::default().collect(Environment::create());
                let t: TestConfig = cfg.build().expect("must success");

                assert_eq!(
                    t,
                    TestConfig {
                        test_a: "test_a".to_string(),
                        test_b: "test_b".to_string(),
                    }
                )
            },
        );

        Ok(())
    }

    #[test]
    fn test_layered_build() -> Result<()> {
        let _ = env_logger::try_init();

        temp_env::with_vars(vec![("test_a", Some("test_a"))], || {
            let cfg = Builder::default()
                .collect(Environment::create())
                .collect(r#"test_b = "test_b""#.into_collector(Toml));
            let t: TestConfig = cfg.build().expect("must success");

            assert_eq!(
                t,
                TestConfig {
                    test_a: "test_a".to_string(),
                    test_b: "test_b".to_string(),
                }
            )
        });

        Ok(())
    }
}
