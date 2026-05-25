#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(untagged)]
pub enum StatusUpdate {
    Component {
        component_update: serde_json::Value,
        component: serde_json::Value,
    },
    Incident {
        incident: serde_json::Value,
    },
}

pub enum StatusUpdateV2 {
    Incident {
        body: String,
        created_at: chrono::DateTime<chrono::Utc>,
        display_at: chrono::DateTime<chrono::Utc>,
        status: String,
        updated_ad: chrono::DateTime<chrono::Utc>,
        id: String,
        incident_id: String,
    },
}

impl<'de> serde::Deserialize<'de> for StatusUpdateV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::endpoints::WebhookMessage;

    #[test]
    fn incident_from_file() {
        let message = serde_json::from_str::<WebhookMessage>(include_str!(
            "../../inputs/github/incident_update.json"
        ))
        .unwrap();
        println!("{message:#?}");
    }

    #[test]
    fn component_from_file() {
        let message = serde_json::from_str::<WebhookMessage>(include_str!(
            "../../inputs/github/component_update.json"
        ))
        .unwrap();
        println!("{message:#?}");
    }
}
