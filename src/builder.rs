use anyhow::{anyhow, Result};
use log::{debug, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{into_value, FromValue, Value};

use crate::collectors::Collector;
use crate::value::{merge, merge_with_default};

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

    pub fn build_with(self, value: V) -> Result<V> {
        let mut result = None;
        let default = into_value(value)?;
        let mut value = Value::Unit;
        for c in self.collectors {
            let collected_value = match c.collect() {
                // Merge will default to make sure every value here is from
                // user input.
                Ok(v) => merge_with_default(default.clone(), v),
                Err(e) => {
                    warn!("collect from {:?}: {:?}", c, e);
                    continue;
                }
            };
            // Three way merge here to make sure we take the last non-default
            // value.
            value = merge(default.clone(), value, collected_value);

            debug!("got value: {:?}", value);
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

impl<V> Builder<V>
where
    V: DeserializeOwned + Serialize + Default,
{
    pub fn build(self) -> Result<V> {
        self.build_with(V::default())
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

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(default)]
    struct TestConfigDefault {
        test_a: String,
        test_b: String,
        test_c: String,
    }

    impl Default for TestConfigDefault {
        fn default() -> Self {
            Self {
                test_a: String::new(),
                test_b: "Hello, World!".to_string(),
                test_c: "Default".to_string(),
            }
        }
    }

    #[test]
    fn test_layered_build_default() -> Result<()> {
        let _ = env_logger::try_init();

        temp_env::with_vars(vec![("test_a", Some("test_a"))], || {
            let cfg = Builder::default()
                .collect(Environment::create())
                .collect(r#"test_b = "test_b""#.into_collector(Toml));
            let t: TestConfigDefault = cfg.build().expect("must success");

            assert_eq!(
                t,
                TestConfigDefault {
                    test_a: "test_a".to_string(),
                    test_b: "test_b".to_string(),
                    test_c: "Default".to_string(),
                }
            )
        });

        Ok(())
    }
}
