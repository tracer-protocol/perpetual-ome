use std::path::Path;

use ethereum_types::U256;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer, Serializer};

/// Helper to convert from hexadecimal strings to decimal strings
///
/// This is necessary to override serde's defaults for the underlying field
/// types we're using.
pub fn from_hex_se<S>(x: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    /* try to convert to an unsigned 128-bit integer, otherwise strip high bits */
    let casted_val: u128 = match *x {
        x if x <= U256::from(u128::MAX) => x.as_u128(),
        _ => x.low_u128(),
    };

    serializer.serialize_u128(casted_val)
}

/// Helper to convert from hexadecimal strings to decimal strings
///
/// This is necessary to override serde's defaults for the underlying field
/// types we're using.
pub fn from_hex_de<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    U256::from_dec_str(&s).map_err(|_e| {
        D::Error::invalid_type(
            Unexpected::Other("non-decimal string"),
            &"decimal string",
        )
    })
}

pub fn is_existing_state(path: &Path) -> bool {
    path.exists()
}