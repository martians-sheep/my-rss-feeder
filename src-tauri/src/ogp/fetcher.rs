use scraper::{Html, Selector};
use url::Url;

#[derive(Debug, Clone)]
pub struct OgpData {
    pub og_image_url: Option<String>,
    pub og_description: Option<String>,
}

/// Extract OGP metadata (og:image, og:description) from HTML.
/// Relative og:image URLs are resolved against `page_url`.
pub fn extract_ogp(html: &str, page_url: &str) -> OgpData {
    let document = Html::parse_document(html);

    let og_image = extract_meta_content(&document, "og:image")
        .and_then(|img_url| resolve_url(page_url, &img_url));

    let og_description = extract_meta_content(&document, "og:description");

    OgpData {
        og_image_url: og_image,
        og_description,
    }
}

fn extract_meta_content(document: &Html, property: &str) -> Option<String> {
    let selector =
        Selector::parse(&format!("meta[property=\"{}\"]", property)).ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn resolve_url(base: &str, target: &str) -> Option<String> {
    if target.starts_with("http://") || target.starts_with("https://") {
        return Some(target.to_string());
    }
    let base_url = Url::parse(base).ok()?;
    base_url.join(target).ok().map(|u| u.to_string())
}

/// Fetch a page and extract its OGP data.
pub async fn fetch_ogp(client: &reqwest::Client, page_url: &str) -> Result<OgpData, crate::error::AppError> {
    let resp = client.get(page_url).send().await?.error_for_status()?;
    let html = resp.text().await?;
    Ok(extract_ogp(&html, page_url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_og_image_and_og_description() {
        let html = r#"
        <html>
        <head>
            <meta property="og:image" content="https://example.com/image.png" />
            <meta property="og:description" content="A test page" />
        </head>
        <body></body>
        </html>
        "#;
        let ogp = extract_ogp(html, "https://example.com/page");
        assert_eq!(
            ogp.og_image_url.as_deref(),
            Some("https://example.com/image.png")
        );
        assert_eq!(ogp.og_description.as_deref(), Some("A test page"));
    }

    #[test]
    fn returns_none_when_og_image_not_set() {
        let html = r#"
        <html>
        <head>
            <meta property="og:description" content="No image here" />
        </head>
        <body></body>
        </html>
        "#;
        let ogp = extract_ogp(html, "https://example.com/page");
        assert!(ogp.og_image_url.is_none());
        assert_eq!(ogp.og_description.as_deref(), Some("No image here"));
    }

    #[test]
    fn converts_relative_url_to_absolute() {
        let html = r#"
        <html>
        <head>
            <meta property="og:image" content="/images/photo.jpg" />
        </head>
        <body></body>
        </html>
        "#;
        let ogp = extract_ogp(html, "https://example.com/blog/post");
        assert_eq!(
            ogp.og_image_url.as_deref(),
            Some("https://example.com/images/photo.jpg")
        );
    }

    #[test]
    fn returns_none_for_empty_content() {
        let html = r#"
        <html>
        <head>
            <meta property="og:image" content="" />
            <meta property="og:description" content="   " />
        </head>
        <body></body>
        </html>
        "#;
        let ogp = extract_ogp(html, "https://example.com/page");
        assert!(ogp.og_image_url.is_none());
        assert!(ogp.og_description.is_none());
    }

    #[test]
    fn returns_none_when_no_meta_tags() {
        let html = "<html><head></head><body></body></html>";
        let ogp = extract_ogp(html, "https://example.com/page");
        assert!(ogp.og_image_url.is_none());
        assert!(ogp.og_description.is_none());
    }
}
