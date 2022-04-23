use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::Value;

/// Collector will collect a value which take `V` as template.
///
/// Implementor SHOULD deserialize into `V` directly and then serialize
/// into a [`serde_bridge::Value`] to make value merge possible.
///
/// Take `serde-env` as an example:
///
/// ```ignore
/// #[derive(Debug)]
/// pub struct Environment<V: DeserializeOwned + Serialize + Debug> {
///     phantom: PhantomData<V>,
/// }
///
/// impl<V> Collector<V> for Environment<V>
/// where
///     V: DeserializeOwned + Serialize + Debug,
/// {
///     fn collect(&mut self) -> Result<Value> {
///         let v: V = serde_env::from_env()?;
///         Ok(v.into_value()?)
///     }
/// }
/// ```
pub trait Collector<V: DeserializeOwned + Serialize> {
    fn collect(&mut self) -> Result<Value>;
}

/// It's recommended to implement `IntoCollector` so that it can be used
/// in [`Builder::collect()`][`crate::Builder::collect()`] directly.
pub trait IntoCollector<V: DeserializeOwned + Serialize> {
    fn into_collector(self) -> Box<dyn Collector<V>>;
}
