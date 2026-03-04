use std::sync::Mutex;

use tauri::{AppHandle, Manager, WebviewBuilder, WebviewUrl};

use crate::error::AppError;

const WEBVIEW_LABEL: &str = "article-webview";
const SIDEBAR_WIDTH: f64 = 256.0;
const DEFAULT_ARTICLE_LIST_WIDTH: f64 = 320.0;
const RESIZE_HANDLE_WIDTH: f64 = 4.0;

static LEFT_OFFSET: Mutex<f64> =
    Mutex::new(SIDEBAR_WIDTH + DEFAULT_ARTICLE_LIST_WIDTH + RESIZE_HANDLE_WIDTH);

pub fn open_article_webview(app: &AppHandle, url: &str) -> Result<(), AppError> {
    // If webview already exists, just navigate to the new URL
    if let Some(webview) = app.get_webview(WEBVIEW_LABEL) {
        webview
            .navigate(
                url.parse()
                    .map_err(|e| AppError::Other(format!("Invalid URL: {}", e)))?,
            )
            .map_err(|e| AppError::Other(format!("Failed to navigate: {}", e)))?;
        return Ok(());
    }

    let window = app
        .get_window("main")
        .ok_or_else(|| AppError::Other("Main window not found".to_string()))?;

    let scale = window
        .scale_factor()
        .map_err(|e| AppError::Other(format!("Failed to get scale factor: {}", e)))?;

    let window_size = window
        .inner_size()
        .map_err(|e| AppError::Other(format!("Failed to get window size: {}", e)))?;

    let logical_width = window_size.width as f64 / scale;
    let logical_height = window_size.height as f64 / scale;

    let left_offset = *LEFT_OFFSET.lock().unwrap();
    let webview_width = (logical_width - left_offset).max(100.0);

    let webview_builder = WebviewBuilder::new(
        WEBVIEW_LABEL,
        WebviewUrl::External(
            url.parse()
                .map_err(|e| AppError::Other(format!("Invalid URL: {}", e)))?,
        ),
    );

    window
        .add_child(
            webview_builder,
            tauri::LogicalPosition::new(left_offset, 0.0),
            tauri::LogicalSize::new(webview_width, logical_height),
        )
        .map_err(|e| AppError::Other(format!("Failed to create webview: {}", e)))?;

    Ok(())
}

pub fn close_article_webview(app: &AppHandle) -> Result<(), AppError> {
    if let Some(webview) = app.get_webview(WEBVIEW_LABEL) {
        webview
            .close()
            .map_err(|e| AppError::Other(format!("Failed to close webview: {}", e)))?;
    }
    Ok(())
}

pub fn update_article_webview_bounds(app: &AppHandle, left_offset: f64) -> Result<(), AppError> {
    *LEFT_OFFSET.lock().unwrap() = left_offset;
    resize_article_webview(app)
}

pub fn resize_article_webview(app: &AppHandle) -> Result<(), AppError> {
    let webview = match app.get_webview(WEBVIEW_LABEL) {
        Some(w) => w,
        None => return Ok(()),
    };

    let window = app
        .get_window("main")
        .ok_or_else(|| AppError::Other("Main window not found".to_string()))?;

    let scale = window
        .scale_factor()
        .map_err(|e| AppError::Other(format!("Failed to get scale factor: {}", e)))?;

    let window_size = window
        .inner_size()
        .map_err(|e| AppError::Other(format!("Failed to get window size: {}", e)))?;

    let logical_width = window_size.width as f64 / scale;
    let logical_height = window_size.height as f64 / scale;

    let left_offset = *LEFT_OFFSET.lock().unwrap();
    let webview_width = (logical_width - left_offset).max(100.0);

    webview
        .set_position(tauri::LogicalPosition::new(left_offset, 0.0))
        .map_err(|e| AppError::Other(format!("Failed to set position: {}", e)))?;

    webview
        .set_size(tauri::LogicalSize::new(webview_width, logical_height))
        .map_err(|e| AppError::Other(format!("Failed to set size: {}", e)))?;

    Ok(())
}

pub fn hide_article_webview(app: &AppHandle) -> Result<(), AppError> {
    if let Some(webview) = app.get_webview(WEBVIEW_LABEL) {
        webview
            .hide()
            .map_err(|e| AppError::Other(format!("Failed to hide webview: {}", e)))?;
    }
    Ok(())
}

pub fn show_article_webview(app: &AppHandle) -> Result<(), AppError> {
    if let Some(webview) = app.get_webview(WEBVIEW_LABEL) {
        webview
            .show()
            .map_err(|e| AppError::Other(format!("Failed to show webview: {}", e)))?;
    }
    Ok(())
}
