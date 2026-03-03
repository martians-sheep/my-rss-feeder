pub mod fetcher;
pub mod image_cache;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgpResult {
    pub article_id: String,
    pub success: bool,
    pub data: Option<OgpResultData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgpResultData {
    pub og_image_url: Option<String>,
    pub og_image_local: Option<String>,
    pub og_description: Option<String>,
}
