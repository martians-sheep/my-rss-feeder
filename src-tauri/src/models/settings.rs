use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub enabled: bool,
    pub time: String,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            time: "10:00".to_string(),
        }
    }
}
