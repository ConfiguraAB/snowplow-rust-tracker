struct TsMillisecondsStringVisitor;

pub mod ts_milliseconds_string {
    use std::fmt;

    use chrono::{DateTime, Utc};
    use serde::{de, ser};

    use super::TsMillisecondsStringVisitor;

    impl<'de> de::Visitor<'de> for TsMillisecondsStringVisitor {
        type Value = DateTime<Utc>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a unix timestamp in milliseconds as string")
        }

        fn visit_str<E>(self, value: &str) -> Result<DateTime<Utc>, E>
        where
            E: de::Error,
        {
            value
                .parse()
                .ok()
                .and_then(DateTime::from_timestamp_millis)
                .ok_or_else(|| de::Error::invalid_value(de::Unexpected::Str(value), &self))
        }
    }

    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&dt.timestamp_millis().to_string())
    }

    pub fn deserialize<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_string(TsMillisecondsStringVisitor)
    }
}

pub mod ts_milliseconds_string_option {
    use std::fmt;

    use chrono::{DateTime, Utc};
    use serde::{de, ser};

    use super::TsMillisecondsStringVisitor;

    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Option<DateTime<Utc>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a unix timestamp in milliseconds as string or none")
        }

        /// Deserialize a timestamp in milliseconds since the epoch
        fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            d.deserialize_string(TsMillisecondsStringVisitor).map(Some)
        }

        /// Deserialize a timestamp in milliseconds since the epoch
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        /// Deserialize a timestamp in milliseconds since the epoch
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    pub fn serialize<S>(opt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *opt {
            Some(ref dt) => serializer.serialize_some(&dt.timestamp_millis().to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_option(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use chrono::{DateTime, SubsecRound, Utc};
    use serde::{Deserialize, Serialize};

    use super::*;

    #[test]
    fn test_serialize_timestamp() {
        #[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
        struct CustomTimestamp(#[serde(with = "ts_milliseconds_string")] DateTime<Utc>);

        let now = Utc::now().trunc_subsecs(0);
        let ts = CustomTimestamp(now);
        let unix = now
            .signed_duration_since(DateTime::<Utc>::from(UNIX_EPOCH))
            .num_milliseconds()
            .to_string();
        assert_eq!(serde_json::to_value(ts).unwrap(), serde_json::json!(unix));
        assert_eq!(
            serde_json::from_str::<CustomTimestamp>(&format!("\"{unix}\"")).unwrap(),
            ts
        );
    }

    #[test]
    fn test_serialize_timestamp_option() {
        #[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
        struct OptionalCustomTimestamp(
            #[serde(with = "ts_milliseconds_string_option")] Option<DateTime<Utc>>,
        );

        let now = Utc::now().trunc_subsecs(0);
        let ts = OptionalCustomTimestamp(Some(now));
        let unix = now
            .signed_duration_since(DateTime::<Utc>::from(UNIX_EPOCH))
            .num_milliseconds()
            .to_string();
        assert_eq!(serde_json::to_value(ts).unwrap(), serde_json::json!(unix));
        assert_eq!(
            serde_json::from_str::<OptionalCustomTimestamp>(&format!("\"{unix}\"")).unwrap(),
            ts
        );

        let ts = OptionalCustomTimestamp(None);
        assert_eq!(serde_json::to_value(ts).unwrap(), serde_json::json!(null));
        assert_eq!(
            serde_json::from_str::<OptionalCustomTimestamp>("null").unwrap(),
            ts
        );
    }
}
