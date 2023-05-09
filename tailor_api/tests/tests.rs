#[cfg(feature = "test-utilities")]
use tailor_api::{default_json_roundtrip, ColorProfile, ProfileInfo};

#[test]
#[cfg(feature = "test-utilities")]
fn default_serde_roundrips() {
    default_json_roundtrip::<ProfileInfo>();
    default_json_roundtrip::<ColorProfile>();
}
