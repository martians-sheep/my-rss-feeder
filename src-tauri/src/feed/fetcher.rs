use crate::error::AppError;

pub struct FetchResult {
    pub body: Vec<u8>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

pub async fn fetch_feed(
    client: &reqwest::Client,
    url: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<Option<FetchResult>, AppError> {
    let mut request = client.get(url);

    if let Some(etag) = etag {
        request = request.header("If-None-Match", etag);
    }
    if let Some(last_modified) = last_modified {
        request = request.header("If-Modified-Since", last_modified);
    }

    let response = request.send().await?;

    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(None);
    }

    let response = response.error_for_status()?;

    let resp_etag = response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let resp_last_modified = response
        .headers()
        .get("last-modified")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let body = response.bytes().await?.to_vec();

    Ok(Some(FetchResult {
        body,
        etag: resp_etag,
        last_modified: resp_last_modified,
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn client_builds_with_timeout() {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();
        assert!(client.get("https://example.com").build().is_ok());
    }
}
