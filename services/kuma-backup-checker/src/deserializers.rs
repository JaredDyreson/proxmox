use serde::Deserialize;

pub fn deserialize_unix_timestamp<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Local>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(
        chrono::DateTime::<chrono::Utc>::from_timestamp(i64::deserialize(deserializer)?, 0)
            .unwrap()
            .with_timezone(&chrono::Local),
    )
}
