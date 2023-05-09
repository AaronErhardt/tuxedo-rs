use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg(feature = "test-utilities")]
pub fn default_json_roundtrip<T: Serialize + for<'a> Deserialize<'a> + Default + Debug + Eq>() -> ()
{
    let orig = T::default();
    let serialized = serde_json::to_string_pretty(&orig).unwrap();
    let deserialized: T = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, orig)
}
