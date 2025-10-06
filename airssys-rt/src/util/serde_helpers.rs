//! Serialization helpers for common types.

use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serde serialization module for Duration as seconds.
pub mod duration_serde {
    use super::*;

    /// Serializes Duration as seconds (u64).
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    /// Deserializes Duration from seconds (u64).
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStruct {
        #[serde(with = "duration_serde")]
        duration: Duration,
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_duration_serde_roundtrip() {
        let original = TestStruct {
            duration: Duration::from_secs(60),
        };

        let json = serde_json::to_string(&original).expect("Serialization should succeed");
        assert!(json.contains("60"));

        let deserialized: TestStruct =
            serde_json::from_str(&json).expect("Deserialization should succeed");
        assert_eq!(original, deserialized);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_duration_serde_zero() {
        let test = TestStruct {
            duration: Duration::from_secs(0),
        };

        let json = serde_json::to_string(&test).expect("Serialization should succeed");
        let deserialized: TestStruct =
            serde_json::from_str(&json).expect("Deserialization should succeed");
        assert_eq!(test.duration, deserialized.duration);
    }
}
