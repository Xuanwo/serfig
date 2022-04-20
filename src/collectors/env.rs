use anyhow::Result;
use serde_bridge::Value;

use crate::Collector;

#[derive(Debug)]
pub struct Environment;

impl Environment {
    pub fn create() -> Box<dyn Collector> {
        Box::new(Self)
    }
}

impl Collector for Environment {
    fn collect(&self) -> Result<Value> {
        Ok(envy::from_env()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_bridge::FromValue;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(rename = "serfig_test_str")]
        test_str: String,
    }

    #[test]
    fn test_env() {
        temp_env::with_vars(vec![("serfig_test_str", Some("test_str"))], || {
            let v = Environment.collect().expect("must success");

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
