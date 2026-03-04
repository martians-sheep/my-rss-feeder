use scraper::{Html, Selector};
use url::Url;

use crate::error::AppError;

/// 検出されたフィード情報
#[derive(Debug, Clone)]
pub struct DiscoveredFeed {
    pub url: String,
    pub title: Option<String>,
    pub feed_type: String,
}

/// HTML内の <link rel="alternate"> タグからフィードURLを検出する
pub fn discover_feeds_from_html(html: &str, page_url: &str) -> Vec<DiscoveredFeed> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("link[rel=\"alternate\"]").unwrap();

    let mut feeds = Vec::new();

    for element in document.select(&selector) {
        let el = element.value();

        let mime_type = match el.attr("type") {
            Some(t) => t,
            None => continue,
        };

        let feed_type = match mime_type {
            "application/atom+xml" => "atom",
            "application/rss+xml" => "rss",
            "application/feed+json" | "application/json" => "json",
            _ => continue,
        };

        let href = match el.attr("href") {
            Some(h) if !h.is_empty() => h,
            _ => continue,
        };

        // 相対URLを絶対URLに変換
        let absolute_url = match resolve_url(page_url, href) {
            Some(url) => url,
            None => continue,
        };

        let title = el.attr("title").map(|s| s.to_string());

        feeds.push(DiscoveredFeed {
            url: absolute_url,
            title,
            feed_type: feed_type.to_string(),
        });
    }

    feeds
}

/// URLがフィードURLかHTMLページかを判定し、フィードURLでなければHTMLからフィードを検出する
pub async fn discover_or_use_feed_url(
    client: &reqwest::Client,
    url: &str,
) -> Result<String, AppError> {
    // まずフィードとしてパースを試みる
    let response = client
        .get(url)
        .send()
        .await?
        .error_for_status()?;

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let body = response.bytes().await?.to_vec();

    // Content-Typeがフィード系、またはXML/JSONなら直接フィードとして利用
    if content_type.contains("xml")
        || content_type.contains("atom")
        || content_type.contains("rss")
        || content_type.contains("feed+json")
    {
        return Ok(url.to_string());
    }

    // feed-rsでパースを試みる
    if feed_rs::parser::parse(&body).is_ok() {
        return Ok(url.to_string());
    }

    // HTMLとしてフィードを検出する
    let html = String::from_utf8_lossy(&body);
    let discovered = discover_feeds_from_html(&html, url);

    if let Some(feed) = discovered.first() {
        Ok(feed.url.clone())
    } else {
        Err(AppError::FeedParse(
            "フィードが見つかりませんでした。フィードURLを直接指定してください。".to_string(),
        ))
    }
}

fn resolve_url(base: &str, target: &str) -> Option<String> {
    if target.starts_with("http://") || target.starts_with("https://") {
        return Some(target.to_string());
    }
    let base_url = Url::parse(base).ok()?;
    base_url.join(target).ok().map(|u| u.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_atom_feed_from_html() {
        let html = r#"
        <html>
        <head>
            <link rel="alternate" type="application/atom+xml" title="Atom Feed" href="/feed/atom" />
        </head>
        <body></body>
        </html>
        "#;
        let feeds = discover_feeds_from_html(html, "https://example.com");
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].url, "https://example.com/feed/atom");
        assert_eq!(feeds[0].feed_type, "atom");
        assert_eq!(feeds[0].title, Some("Atom Feed".to_string()));
    }

    #[test]
    fn discover_rss_feed_from_html() {
        let html = r#"
        <html>
        <head>
            <link rel="alternate" type="application/rss+xml" title="RSS Feed" href="https://example.com/rss.xml" />
        </head>
        <body></body>
        </html>
        "#;
        let feeds = discover_feeds_from_html(html, "https://example.com");
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].url, "https://example.com/rss.xml");
        assert_eq!(feeds[0].feed_type, "rss");
    }

    #[test]
    fn discover_multiple_feeds() {
        let html = r#"
        <html>
        <head>
            <link rel="alternate" type="application/atom+xml" title="Atom" href="/atom.xml" />
            <link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml" />
        </head>
        <body></body>
        </html>
        "#;
        let feeds = discover_feeds_from_html(html, "https://example.com");
        assert_eq!(feeds.len(), 2);
    }

    #[test]
    fn no_feeds_in_html() {
        let html = r#"
        <html>
        <head>
            <link rel="stylesheet" href="/style.css" />
        </head>
        <body></body>
        </html>
        "#;
        let feeds = discover_feeds_from_html(html, "https://example.com");
        assert!(feeds.is_empty());
    }

    #[test]
    fn ignores_link_without_href() {
        let html = r#"
        <html>
        <head>
            <link rel="alternate" type="application/atom+xml" title="No Href" />
        </head>
        <body></body>
        </html>
        "#;
        let feeds = discover_feeds_from_html(html, "https://example.com");
        assert!(feeds.is_empty());
    }

    #[test]
    fn resolves_relative_urls() {
        let html = r#"
        <html>
        <head>
            <link rel="alternate" type="application/atom+xml" href="../feed.xml" />
        </head>
        <body></body>
        </html>
        "#;
        let feeds = discover_feeds_from_html(html, "https://example.com/blog/page");
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].url, "https://example.com/feed.xml");
    }
}
