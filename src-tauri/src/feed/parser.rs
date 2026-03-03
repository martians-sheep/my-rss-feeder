use sha2::{Digest, Sha256};

use crate::error::AppError;

pub struct ParsedFeed {
    pub title: String,
    pub site_url: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub entries: Vec<ParsedEntry>,
}

pub struct ParsedEntry {
    pub entry_id: String,
    pub title: String,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<String>,
}

pub fn parse_feed(data: &[u8], feed_url: &str) -> Result<ParsedFeed, AppError> {
    let feed =
        feed_rs::parser::parse(data).map_err(|e| AppError::FeedParse(format!("{}", e)))?;

    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| feed_url.to_string());

    let site_url = feed.links.first().map(|l| l.href.clone());

    let description = feed.description.map(|d| d.content);

    let icon_url = feed
        .icon
        .map(|i| i.uri)
        .or_else(|| feed.logo.map(|l| l.uri));

    let entries = feed
        .entries
        .into_iter()
        .map(|entry| {
            let entry_id = if entry.id.is_empty() {
                let fallback = entry
                    .links
                    .first()
                    .map(|l| l.href.clone())
                    .or_else(|| entry.title.as_ref().map(|t| t.content.clone()))
                    .unwrap_or_default();
                let mut hasher = Sha256::new();
                hasher.update(fallback.as_bytes());
                format!("{:x}", hasher.finalize())
            } else {
                entry.id.clone()
            };

            let title = entry
                .title
                .map(|t| t.content)
                .unwrap_or_else(|| "(no title)".to_string());

            let url = entry.links.first().map(|l| l.href.clone());

            let summary = entry.summary.map(|s| s.content);

            let content = entry.content.and_then(|c| c.body);

            let author = entry.authors.first().map(|a| a.name.clone());

            let published_at = entry
                .published
                .or(entry.updated)
                .map(|dt| dt.to_rfc3339());

            ParsedEntry {
                entry_id,
                title,
                url,
                summary,
                content,
                author,
                published_at,
            }
        })
        .collect();

    Ok(ParsedFeed {
        title,
        site_url,
        description,
        icon_url,
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RSS2: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test RSS Feed</title>
    <link>https://example.com</link>
    <description>A test RSS feed</description>
    <item>
      <title>First Article</title>
      <link>https://example.com/article1</link>
      <guid>article-1</guid>
      <pubDate>Mon, 01 Jan 2024 00:00:00 GMT</pubDate>
      <description>Summary of first article</description>
    </item>
    <item>
      <title>Second Article</title>
      <link>https://example.com/article2</link>
      <guid>article-2</guid>
    </item>
  </channel>
</rss>"#;

    const SAMPLE_ATOM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Test Atom Feed</title>
  <link href="https://example.com"/>
  <id>urn:uuid:test-feed</id>
  <entry>
    <title>Atom Entry</title>
    <link href="https://example.com/atom-entry"/>
    <id>urn:uuid:atom-entry-1</id>
    <updated>2024-01-01T00:00:00Z</updated>
    <summary>Summary of atom entry</summary>
    <author><name>Author Name</name></author>
  </entry>
</feed>"#;

    #[test]
    fn parse_rss2_feed() {
        let result =
            parse_feed(SAMPLE_RSS2.as_bytes(), "https://example.com/feed.xml").unwrap();
        assert_eq!(result.title, "Test RSS Feed");
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].title, "First Article");
        assert!(result.entries[0].url.is_some());
        assert!(result.entries[0].summary.is_some());
    }

    #[test]
    fn parse_atom_feed() {
        let result =
            parse_feed(SAMPLE_ATOM.as_bytes(), "https://example.com/atom.xml").unwrap();
        assert_eq!(result.title, "Test Atom Feed");
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].title, "Atom Entry");
        assert_eq!(
            result.entries[0].author,
            Some("Author Name".to_string())
        );
    }

    #[test]
    fn title_falls_back_to_url_when_missing() {
        let no_title_rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <link>https://example.com</link>
    <item>
      <title>An Article</title>
      <guid>a-1</guid>
    </item>
  </channel>
</rss>"#;
        let result = parse_feed(
            no_title_rss.as_bytes(),
            "https://example.com/feed.xml",
        )
        .unwrap();
        assert_eq!(result.title, "https://example.com/feed.xml");
    }

    #[test]
    fn entries_always_have_non_empty_entry_id() {
        let result =
            parse_feed(SAMPLE_RSS2.as_bytes(), "https://example.com/feed.xml").unwrap();
        for entry in &result.entries {
            assert!(!entry.entry_id.is_empty());
        }
    }

    #[test]
    fn invalid_xml_returns_error() {
        let result = parse_feed(b"not xml at all", "https://example.com/feed.xml");
        assert!(result.is_err());
    }

    #[test]
    fn parse_rss2_extracts_description() {
        let result =
            parse_feed(SAMPLE_RSS2.as_bytes(), "https://example.com/feed.xml").unwrap();
        assert_eq!(result.description, Some("A test RSS feed".to_string()));
    }

    #[test]
    fn parse_atom_extracts_site_url() {
        let result =
            parse_feed(SAMPLE_ATOM.as_bytes(), "https://example.com/atom.xml").unwrap();
        assert!(result.site_url.is_some());
    }
}
