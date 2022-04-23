//! serfig is a layered configuration system built upon serde.
//!
//! [`Builder`] will load configurations via different [collectors][crate::collectors].
//!
//! # Examples
//!
//! ```
//! use serde::{Deserialize, Serialize};
//! use serfig::collectors::{from_env, from_file, from_self};
//! use serfig::parsers::Toml;
//! use serfig::Builder;
//!
//! #[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
//! #[serde(default)]
//! struct TestConfig {
//!     a: String,
//!     b: String,
//!     c: i64,
//! }
//!
//! fn main() -> anyhow::Result<()> {
//!     let builder = Builder::default()
//!         .collect(from_env())
//!         .collect(from_file(Toml, "config.toml"))
//!         .collect(from_self(TestConfig::default()));
//!     let t: TestConfig = builder.build()?;
//!
//!     println!("{:?}", t);
//!     Ok(())
//! }
//! ```

mod builder;
pub use builder::Builder;

pub mod collectors;
pub use collectors::Collector;

pub mod parsers;
pub use parsers::Parser;

mod value;
