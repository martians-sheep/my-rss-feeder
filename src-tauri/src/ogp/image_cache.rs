use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::error::AppError;

const MAX_IMAGE_SIZE: u64 = 2 * 1024 * 1024; // 2MB

/// Cache an image from `image_url` into `cache_dir`.
/// Returns the local file path on success, or None if the image was skipped (e.g., too large).
pub async fn cache_image(
    client: &reqwest::Client,
    image_url: &str,
    cache_dir: &Path,
) -> Result<Option<PathBuf>, AppError> {
    let filename = url_to_filename(image_url);
    let local_path = cache_dir.join(&filename);

    // Skip download if already cached
    if local_path.exists() {
        return Ok(Some(local_path));
    }

    std::fs::create_dir_all(cache_dir)?;

    let resp = client.get(image_url).send().await?.error_for_status()?;

    // Check Content-Length header first for early rejection
    if let Some(content_length) = resp.content_length() {
        if content_length > MAX_IMAGE_SIZE {
            return Ok(None);
        }
    }

    let bytes = resp.bytes().await?;

    // Double-check actual size
    if bytes.len() as u64 > MAX_IMAGE_SIZE {
        return Ok(None);
    }

    std::fs::write(&local_path, &bytes)?;
    Ok(Some(local_path))
}

/// Generate a deterministic filename from a URL: SHA256(url) + extension.
fn url_to_filename(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let ext = extract_extension(url).unwrap_or("jpg");
    format!("{}.{}", hash, ext)
}

fn extract_extension(url: &str) -> Option<&str> {
    let path = url.split('?').next()?;
    let filename = path.rsplit('/').next()?;
    let ext = filename.rsplit('.').next()?;
    let ext = ext.to_lowercase();
    // Only allow common image extensions
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "avif" => {
            // Return the original slice from the URL for valid extensions
            Some(filename.rsplit('.').next()?)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    use tokio::net::TcpListener;

    /// Start a tiny HTTP server that serves `body` with the given Content-Type.
    async fn start_server(body: Vec<u8>, content_type: &str) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr: SocketAddr = listener.local_addr().unwrap();
        let ct = content_type.to_string();

        let handle = tokio::spawn(async move {
            // Serve one request
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await.unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                ct,
                body.len()
            );
            tokio::io::AsyncWriteExt::write_all(&mut stream, response.as_bytes())
                .await
                .unwrap();
            tokio::io::AsyncWriteExt::write_all(&mut stream, &body)
                .await
                .unwrap();
        });

        (format!("http://{}/image.png", addr), handle)
    }

    #[tokio::test]
    async fn caches_image_locally() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("image_cache");
        let image_data = vec![0xFFu8; 1024]; // 1KB fake image

        let (url, server) = start_server(image_data.clone(), "image/png").await;
        let client = reqwest::Client::new();

        let result = cache_image(&client, &url, &cache_dir).await.unwrap();
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.exists());
        assert_eq!(std::fs::read(&path).unwrap(), image_data);

        server.abort();
    }

    #[tokio::test]
    async fn skips_download_for_cached_image() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("image_cache");
        std::fs::create_dir_all(&cache_dir).unwrap();

        let image_url = "https://example.com/image.png";
        let filename = url_to_filename(image_url);
        let cached_path = cache_dir.join(&filename);
        std::fs::write(&cached_path, b"already cached").unwrap();

        let client = reqwest::Client::new();
        let result = cache_image(&client, image_url, &cache_dir).await.unwrap();

        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "already cached");
    }

    #[tokio::test]
    async fn skips_images_over_2mb() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("image_cache");
        let big_image = vec![0xFFu8; (2 * 1024 * 1024) + 1]; // 2MB + 1 byte

        let (url, server) = start_server(big_image, "image/png").await;
        let client = reqwest::Client::new();

        let result = cache_image(&client, &url, &cache_dir).await.unwrap();
        assert!(result.is_none());

        server.abort();
    }

    #[test]
    fn url_to_filename_uses_sha256_and_extension() {
        let filename = url_to_filename("https://example.com/photo.png");
        assert!(filename.ends_with(".png"));
        assert!(filename.len() > 64); // SHA256 hex = 64 chars + dot + ext
    }

    #[test]
    fn url_to_filename_defaults_to_jpg() {
        let filename = url_to_filename("https://example.com/image");
        assert!(filename.ends_with(".jpg"));
    }
}
