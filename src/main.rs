#![allow(dead_code, unused_imports)]
mod api;
mod download;

use axum::routing::Route;
use axum::{http, routing::get, Json, Router};

use crate::api::v1;
use async_trait::async_trait;
use axum::extract::{FromRequest, Query, RequestParts};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{info, Level};
use url::Url;

#[tokio::main]
async fn main() {
    let collector = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(collector).expect("unable to install log handler");

    let app = Router::new()
        .route("/", get(root))
        .layer(TraceLayer::new_for_http())
        .route("/api/v1/hello-world", get(hello_world))
        .route("/api/v1/download", get(v1::new_download));

    let addr = SocketAddr::from(([127, 0, 0, 1], 6969));

    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "This is the root, you shouldn't be here"
}

async fn hello_world() -> &'static str {
    "Hello World"
}
