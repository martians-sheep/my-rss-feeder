use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub id: String,
    pub title: String,
    pub url: String,
    pub site_url: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_fetched_at: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_feed() -> Feed {
        Feed {
            id: "test-id".to_string(),
            title: "Test Feed".to_string(),
            url: "https://example.com/feed.xml".to_string(),
            site_url: Some("https://example.com".to_string()),
            description: Some("A test feed".to_string()),
            icon_url: None,
            created_at: "2024-01-01T00:00:00+00:00".to_string(),
            updated_at: "2024-01-01T00:00:00+00:00".to_string(),
            last_fetched_at: None,
            etag: None,
            last_modified: None,
        }
    }

    #[test]
    fn feed_serializes_to_camel_case_json() {
        let feed = sample_feed();
        let json = serde_json::to_value(&feed).unwrap();
        assert!(json.get("siteUrl").is_some());
        assert!(json.get("createdAt").is_some());
        assert!(json.get("updatedAt").is_some());
        assert!(json.get("lastFetchedAt").is_some());
        assert!(json.get("iconUrl").is_some());
        assert!(json.get("site_url").is_none());
        assert!(json.get("created_at").is_none());
    }

    #[test]
    fn feed_deserializes_from_camel_case_json() {
        let json = r#"{
            "id": "test-id",
            "title": "Test Feed",
            "url": "https://example.com/feed.xml",
            "siteUrl": "https://example.com",
            "description": null,
            "iconUrl": null,
            "createdAt": "2024-01-01T00:00:00+00:00",
            "updatedAt": "2024-01-01T00:00:00+00:00",
            "lastFetchedAt": null,
            "etag": null,
            "lastModified": null
        }"#;
        let feed: Feed = serde_json::from_str(json).unwrap();
        assert_eq!(feed.id, "test-id");
        assert_eq!(feed.title, "Test Feed");
        assert!(feed.site_url.is_some());
    }

    #[test]
    fn feed_roundtrip_serialization() {
        let feed = sample_feed();
        let json = serde_json::to_string(&feed).unwrap();
        let deserialized: Feed = serde_json::from_str(&json).unwrap();
        assert_eq!(feed.id, deserialized.id);
        assert_eq!(feed.url, deserialized.url);
        assert_eq!(feed.site_url, deserialized.site_url);
    }
}
