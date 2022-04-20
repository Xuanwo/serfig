use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_bridge::{FromValue, Value};

use crate::value::merge;
use crate::Collector;

#[derive(Default)]
pub struct Builder {
    collectors: Vec<Box<dyn Collector>>,
}

impl Builder {
    pub fn new() -> Builder {
        Self::default()
    }

    pub fn collect(mut self, c: impl Collector) -> Self {
        self.collectors.push(Box::new(c));
        Self {
            collectors: self.collectors,
        }
    }

    pub fn build<T: DeserializeOwned>(self) -> Result<T> {
        let mut result = None;
        let mut value = Value::Unit;
        for c in self.collectors {
            // Merge value if we collect successfully.
            let pre_value = value.clone();
            value = match c.collect() {
                Ok(v) => merge(value, v),
                Err(_) => continue,
            };
            // Re-deserialize the value if we from_value correctly.
            result = match T::from_value(value.clone()) {
                Ok(v) => Some(v),
                Err(_) => {
                    // Reset to previous value if we from_value failed.
                    value = pre_value;
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
    use crate::impls::Environment;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestConfig {
        test_a: String,
        test_b: String,
    }

    #[test]
    fn test_build() -> Result<()> {
        temp_env::with_vars(
            vec![("test_a", Some("test_a")), ("test_b", Some("test_b"))],
            || {
                let cfg = Builder::default().collect(Environment);
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
}
