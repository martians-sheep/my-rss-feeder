use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub id: String,
    pub feed_id: String,
    pub entry_id: String,
    pub title: String,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<String>,
    pub is_read: bool,
    pub read_at: Option<String>,
    pub og_image_url: Option<String>,
    pub og_image_local: Option<String>,
    pub og_description: Option<String>,
    pub og_fetched: bool,
    pub created_at: String,
    pub feed_title: Option<String>,
    pub categories: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_article() -> Article {
        Article {
            id: "article-1".to_string(),
            feed_id: "feed-1".to_string(),
            entry_id: "entry-1".to_string(),
            title: "Test Article".to_string(),
            url: Some("https://example.com/article".to_string()),
            summary: None,
            content: None,
            author: None,
            published_at: None,
            is_read: false,
            read_at: None,
            og_image_url: None,
            og_image_local: None,
            og_description: None,
            og_fetched: false,
            created_at: "2024-01-01T00:00:00+00:00".to_string(),
            feed_title: None,
            categories: None,
        }
    }

    #[test]
    fn article_default_state_is_unread() {
        let article = sample_article();
        assert!(!article.is_read);
        assert!(article.read_at.is_none());
    }

    #[test]
    fn article_default_og_is_unfetched() {
        let article = sample_article();
        assert!(!article.og_fetched);
        assert!(article.og_image_url.is_none());
    }

    #[test]
    fn article_serializes_to_camel_case() {
        let article = sample_article();
        let json = serde_json::to_value(&article).unwrap();
        assert!(json.get("feedId").is_some());
        assert!(json.get("entryId").is_some());
        assert!(json.get("isRead").is_some());
        assert!(json.get("ogFetched").is_some());
        assert!(json.get("feedTitle").is_some());
        assert!(json.get("feed_id").is_none());
        assert!(json.get("is_read").is_none());
    }

    #[test]
    fn article_is_read_serializes_as_boolean() {
        let article = sample_article();
        let json = serde_json::to_value(&article).unwrap();
        assert_eq!(json["isRead"], false);
    }
}
