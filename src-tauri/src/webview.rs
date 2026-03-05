use std::sync::Mutex;

use tauri::webview::PageLoadEvent;
use tauri::{AppHandle, Manager, WebviewBuilder, WebviewUrl};

use crate::error::AppError;

const WEBVIEW_LABEL: &str = "article-webview";
const SIDEBAR_WIDTH: f64 = 256.0;
const DEFAULT_ARTICLE_LIST_WIDTH: f64 = 320.0;
const RESIZE_HANDLE_WIDTH: f64 = 4.0;

static LEFT_OFFSET: Mutex<f64> =
    Mutex::new(SIDEBAR_WIDTH + DEFAULT_ARTICLE_LIST_WIDTH + RESIZE_HANDLE_WIDTH);

static PENDING_HIGHLIGHT_TITLE: Mutex<Option<String>> = Mutex::new(None);

const HIGHLIGHT_INIT_SCRIPT: &str = r#"
window.__rssHighlightTitle = function(title, retryCount) {
    window.__rssRemoveHighlight();
    if (!document.body) {
        if ((retryCount || 0) < 10) {
            setTimeout(function() { window.__rssHighlightTitle(title, (retryCount || 0) + 1); }, 500);
        }
        return;
    }

    var searchText = title;
    var found = false;
    var minLen = Math.max(2, Math.min(title.length, 2));

    while (searchText.length >= minLen && !found) {
        var walker = document.createTreeWalker(
            document.body,
            NodeFilter.SHOW_TEXT,
            null,
            false
        );

        var node;
        while (node = walker.nextNode()) {
            var idx = node.textContent.indexOf(searchText);
            if (idx !== -1) {
                var range = document.createRange();
                range.setStart(node, idx);
                range.setEnd(node, idx + searchText.length);
                var mark = document.createElement('mark');
                mark.className = '__rss-highlight';
                mark.style.backgroundColor = '#fef08a';
                mark.style.padding = '2px';
                range.surroundContents(mark);
                mark.scrollIntoView({ behavior: 'smooth', block: 'center' });
                found = true;
                break;
            }
        }

        if (!found) {
            searchText = searchText.substring(0, searchText.length - 1);
        }
    }

    if (!found && typeof window.find === 'function') {
        var findText = title;
        while (findText.length >= 2 && !found) {
            if (window.find(findText, false, false, false, false, true, false)) {
                found = true;
            } else {
                findText = findText.substring(0, findText.length - 1);
            }
        }
    }

    if (!found && (retryCount || 0) < 5) {
        setTimeout(function() { window.__rssHighlightTitle(title, (retryCount || 0) + 1); }, 1000);
    }
};

window.__rssRemoveHighlight = function() {
    var marks = document.querySelectorAll('mark.__rss-highlight');
    marks.forEach(function(mark) {
        var parent = mark.parentNode;
        while (mark.firstChild) {
            parent.insertBefore(mark.firstChild, mark);
        }
        parent.removeChild(mark);
        parent.normalize();
    });
    if (window.getSelection) {
        window.getSelection().removeAllRanges();
    }
};
"#;

fn escape_for_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

pub fn build_highlight_eval(title: &str) -> String {
    let escaped = escape_for_js(title);

    format!(
        r#"(function() {{
            function tryHighlight() {{
                if (typeof window.__rssHighlightTitle === 'function') {{
                    window.__rssHighlightTitle('{}');
                }} else {{
                    setTimeout(tryHighlight, 200);
                }}
            }}
            setTimeout(tryHighlight, 500);
        }})();"#,
        escaped
    )
}

pub fn open_article_webview(
    app: &AppHandle,
    url: &str,
    title: Option<&str>,
) -> Result<(), AppError> {
    // Store the pending highlight title before navigation
    *PENDING_HIGHLIGHT_TITLE.lock().unwrap() = title.map(|t| t.to_string());

    // If webview already exists, just navigate to the new URL
    // The on_page_load callback (set at creation) will handle highlighting
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
    )
    .initialization_script(HIGHLIGHT_INIT_SCRIPT)
    .on_page_load(|webview, payload| {
        if matches!(payload.event(), PageLoadEvent::Finished) {
            let title = PENDING_HIGHLIGHT_TITLE.lock().unwrap().take();
            if let Some(t) = title {
                let js = build_highlight_eval(&t);
                let _ = webview.eval(&js);
            }
        }
    });

    window
        .add_child(
            webview_builder,
            tauri::LogicalPosition::new(left_offset, 0.0),
            tauri::LogicalSize::new(webview_width, logical_height),
        )
        .map_err(|e| AppError::Other(format!("Failed to create webview: {}", e)))?;

    Ok(())
}

pub fn highlight_in_webview(app: &AppHandle, title: &str) -> Result<(), AppError> {
    let webview = app
        .get_webview(WEBVIEW_LABEL)
        .ok_or_else(|| AppError::Other("Article webview not found".to_string()))?;

    let escaped = escape_for_js(title);
    let js = format!(
        "if (typeof window.__rssHighlightTitle === 'function') {{ window.__rssHighlightTitle('{}'); }}",
        escaped
    );
    webview
        .eval(&js)
        .map_err(|e| AppError::Other(format!("Failed to eval highlight: {}", e)))?;
    Ok(())
}

pub fn remove_highlight_in_webview(app: &AppHandle) -> Result<(), AppError> {
    let webview = app
        .get_webview(WEBVIEW_LABEL)
        .ok_or_else(|| AppError::Other("Article webview not found".to_string()))?;

    webview
        .eval("if (typeof window.__rssRemoveHighlight === 'function') { window.__rssRemoveHighlight(); }")
        .map_err(|e| AppError::Other(format!("Failed to eval remove highlight: {}", e)))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_highlight_eval_escapes_single_quotes() {
        let js = build_highlight_eval("it's a test");
        assert!(js.contains(r"it\'s a test"));
        assert!(!js.contains("it's a test"));
    }

    #[test]
    fn build_highlight_eval_escapes_backslashes() {
        let js = build_highlight_eval(r"path\to\file");
        assert!(js.contains(r"path\\to\\file"));
    }

    #[test]
    fn build_highlight_eval_escapes_newlines() {
        let js = build_highlight_eval("line1\nline2\rline3");
        assert!(js.contains(r"line1\nline2\rline3"));
        assert!(!js.contains('\n') || !js.contains("line1\nline2"));
    }

    #[test]
    fn build_highlight_eval_handles_japanese() {
        let js = build_highlight_eval("官報の記事タイトル");
        assert!(js.contains("官報の記事タイトル"));
        assert!(js.contains("__rssHighlightTitle"));
    }

    #[test]
    fn build_highlight_eval_contains_required_functions() {
        let js = build_highlight_eval("test title");
        assert!(js.contains("__rssHighlightTitle"));
        assert!(js.contains("tryHighlight"));
        assert!(js.contains("setTimeout"));
    }

    #[test]
    fn escape_for_js_handles_combined() {
        let result = escape_for_js("it's a\\path\nwith\rstuff");
        assert_eq!(result, r"it\'s a\\path\nwith\rstuff");
    }
}
