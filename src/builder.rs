use anyhow::{anyhow, Result};
use log::{debug, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{into_value, FromValue};

use crate::collectors::{Collector, IntoCollector};
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

    pub fn collect(mut self, c: impl IntoCollector<V>) -> Self {
        self.collectors.push(c.into_collector());
        Self {
            collectors: self.collectors,
        }
    }

    pub fn build_with(self, value: V) -> Result<V> {
        let mut result = None;
        let default = into_value(value)?;
        let mut value = default.clone();
        for mut c in self.collectors {
            let collected_value = match c.collect() {
                // Merge will default to make sure every value here is from
                // user input.
                Ok(v) => merge_with_default(default.clone(), v),
                Err(e) => {
                    warn!("collect: {:?}", e);
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
    use crate::collectors::*;
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
                let cfg = Builder::default().collect(from_env());
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
                .collect(from_env())
                .collect(from_str(Toml, r#"test_b = "test_b""#));
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
                .collect(from_env())
                .collect(from_str(Toml, r#"test_b = "test_b""#));
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

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(default)]
    struct TestConfigBool {
        test_bool: bool,
    }

    impl Default for TestConfigBool {
        fn default() -> Self {
            Self { test_bool: true }
        }
    }

    #[test]
    fn test_config_bool_enabled() -> Result<()> {
        let _ = env_logger::try_init();

        temp_env::with_vars(vec![("test_bool", Some("false"))], || {
            let cfg = Builder::default().collect(from_env());
            let t: TestConfigBool = cfg.build().expect("must success");

            assert_eq!(t, TestConfigBool { test_bool: false })
        });

        Ok(())
    }
}
