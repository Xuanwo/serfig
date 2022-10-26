//! Collectors will provide abstractions for different collectors.
//!
//! Every collectors implements
//!
//! ```ignore
//! pub trait Collector<V: DeserializeOwned + Serialize> {
//!     fn collect(&mut self) -> Result<Value>;
//! }
//! ```
//!
//! We are supports the following collectors:
//!
//! - [`from_env`]: Load from current environment.
//! - [`from_file`]: Load from file with specific format like toml.
//! - [`from_reader`]: Load from [`std::io::Read`] with specific format like toml.
//! - [`from_str`]: Load from string with specific format like toml.
//! - [`from_self`]: Load the config value itself.
//!
//! Collectors often been used by [`Builder`][`crate::Builder`]:
//!
//! For Examples:
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
//!         .collect(from_self(TestConfig::default()));
//!     let t: TestConfig = builder.build()?;
//!
//!     println!("{:?}", t);
//!     Ok(())
//! }
//! ```

mod collector;
pub use collector::{Collector, IntoCollector};

mod env;
pub use env::from_env;

mod structural;
pub use structural::{from_file, from_reader, from_str};

mod value;
pub use value::from_self;
