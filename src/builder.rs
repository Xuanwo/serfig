use anyhow::{anyhow, Result};
use log::{debug, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{into_value, FromValue};

use crate::collectors::{Collector, IntoCollector};
use crate::value::{merge, merge_with_default};

/// Builder will collect values from different collectors and merge into the final value.
#[derive(Default)]
pub struct Builder<V: DeserializeOwned + Serialize> {
    collectors: Vec<Box<dyn Collector<V>>>,
}

impl<V> Builder<V>
where
    V: DeserializeOwned + Serialize,
{
    /// Create new builders.
    pub fn new() -> Builder<V> {
        Self {
            collectors: Vec::new(),
        }
    }

    /// Add collectors into builder.
    ///
    /// This is a lazy operation that no real IO happens.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use serfig::collectors::{from_env, from_file, from_self};
    /// use serfig::parsers::Toml;
    /// use serfig::Builder;
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
    ///         .collect(from_env())
    ///         .collect(from_file(Toml, "config.toml"))
    ///         .collect(from_self(TestConfig::default()));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn collect(mut self, c: impl IntoCollector<V>) -> Self {
        self.collectors.push(c.into_collector());
        Self {
            collectors: self.collectors,
        }
    }

    /// Use input `default` as the default value to build.
    ///
    /// # Behavior
    ///
    /// Builder will ignore any errors happened during build, and only returns
    /// errors if no valid value collected.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use serfig::collectors::{from_env, from_file, from_self};
    /// use serfig::parsers::Toml;
    /// use serfig::Builder;
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
    ///
    ///     let t = builder.build_with(TestConfig::default())?;
    ///     Ok(())
    /// }
    /// ```
    pub fn build_with(self, default: V) -> Result<V> {
        let mut result = None;
        let default = into_value(default)?;
        let mut value = default.clone();
        for mut c in self.collectors {
            // Merge will default to make sure every value here is from
            // user input.
            let collected_value = merge_with_default(default.clone(), c.collect()?);

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
    /// If input value implements `Default`, we can use `build` instead.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use serde::{Deserialize, Serialize};
    /// use serfig::collectors::{from_env, from_file, from_self};
    /// use serfig::parsers::Toml;
    /// use serfig::Builder;
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
    ///         .collect(from_env())
    ///         .collect(from_file(Toml, "config.toml"));
    ///
    ///     let t = builder.build()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn build(self) -> Result<V> {
        self.build_with(V::default())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::collectors::*;
    use crate::parsers::Toml;

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

    #[test]
    fn test_layered_overwrite() -> Result<()> {
        let _ = env_logger::try_init();

        temp_env::with_vars(
            vec![("test_a", Some("test_a")), ("test_b", Some("test_b"))],
            || {
                let cfg = Builder::default()
                    .collect(from_env())
                    .collect(from_str(Toml, r#"test_b = "test_b_overwrite""#));
                let t: TestConfig = cfg.build().expect("must success");

                assert_eq!(
                    t,
                    TestConfig {
                        test_a: "test_a".to_string(),
                        test_b: "test_b_overwrite".to_string(),
                    }
                )
            },
        );

        temp_env::with_vars(
            vec![("test_a", Some("test_a")), ("test_b", Some("test_b"))],
            || {
                let cfg = Builder::default()
                    .collect(from_str(Toml, r#"test_b = "test_b_overwrite""#))
                    .collect(from_env());
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

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(default)]
    struct TestConfigDefault {
        test_a: String,
        test_b: String,
        test_c: String,
        test_d: String,
    }

    impl Default for TestConfigDefault {
        fn default() -> Self {
            Self {
                test_a: String::new(),
                test_b: "Hello, World!".to_string(),
                test_c: "Default".to_string(),
                test_d: "".to_string(),
            }
        }
    }

    #[test]
    fn test_layered_build_default() -> Result<()> {
        let _ = env_logger::try_init();

        temp_env::with_vars(
            vec![
                ("test_a", Some("test_a")),
                ("test_b", Some("test_b_from_env")),
            ],
            || {
                let cfg = Builder::default()
                    .collect(from_env())
                    .collect(from_str(Toml, r#"test_b = "test_b""#))
                    .collect(from_str(Toml, r#"test_b = "Hello, World!""#))
                    .collect(from_self(TestConfigDefault {
                        test_d: "override".to_string(),
                        ..Default::default()
                    }));
                let t: TestConfigDefault = cfg.build().expect("must success");

                assert_eq!(
                    t,
                    TestConfigDefault {
                        test_a: "test_a".to_string(),
                        test_b: "test_b".to_string(),
                        test_c: "Default".to_string(),
                        test_d: "override".to_string(),
                    }
                )
            },
        );

        Ok(())
    }

    #[derive(Debug, Serialize, Default, Deserialize, PartialEq)]
    #[serde(default)]
    struct TestConfigVec {
        test_a: Vec<String>,
    }

    #[test]
    fn test_layered_build_vec() -> Result<()> {
        let _ = env_logger::try_init();

        temp_env::with_vars(vec![("test_a", Some(""))], || {
            let cfg = Builder::default()
                .collect(from_env())
                .collect(from_str(Toml, r#"test_a = ["test_b"]"#))
                .collect(from_self(TestConfigVec::default()));
            let t: TestConfigVec = cfg.build().expect("must success");

            assert_eq!(
                t,
                TestConfigVec {
                    test_a: vec!["test_b".to_string()],
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
