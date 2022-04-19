use anyhow::Result;
use serde_bridge::Value;

use crate::Collector;

struct Environment;

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
            println!("{:?}", v);

            let t = TestStruct::from_value(v).expect("must success");
            println!("{:?}", t);

            assert_eq!(
                t,
                TestStruct {
                    test_str: "test_str".to_string()
                }
            )
        })
    }
}
