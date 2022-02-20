use axum::{
    body::{self, Bytes},
    http::StatusCode,
    response::IntoResponse,
    Router,
};

use reqwest::Client;
use reqwest::{get, Response};

use axum::extract::Query;
use std::path::PathBuf;

use tokio::fs::File;

use serde::Deserialize;

use url::{ParseError, Url};

pub struct Error(reqwest::Error);

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self { 0: err }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let builder = axum::response::Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR);

        let res = builder.body(body::boxed(body::Full::from("something went in reqwest")));

        res.unwrap()
    }
}

/// Maps to /api/v1/download
pub async fn new_download(req: Query<DownloadReq>) -> Result<String, Error> {
    let client = Client::new();
    let url = req.0.url;

    let url_clone = url.clone();
    let request = client.get(url).build()?;

    client.execute(request).await?;

    Ok(format!("you passed in {}", url_clone))
}

pub struct DownloadResponse {
    message: String,
}

#[derive(Deserialize)]
pub struct DownloadReq {
    url: Url,
}
