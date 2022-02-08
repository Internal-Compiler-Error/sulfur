use axum::extract::Query;
use reqwest::get;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs::File;
use url::Url;

/// Maps to /api/v1/download
pub async fn new_download(req: Query<DownloadReq>) -> String {
    format!("you passed in {}", req.url)
}

#[derive(Deserialize)]
pub struct DownloadReq {
    url: Url,
}
