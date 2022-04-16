#![allow(dead_code)]

#[macro_use]
extern crate diesel;


pub mod schema;
pub mod api;
pub mod http;
pub mod model;
pub mod persistence;


use axum::{routing::get, Router};

use crate::api::v1;



use std::net::SocketAddr;
use tower_http::{trace::TraceLayer};
use tracing::{Level};


#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

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

    Ok(())
}

async fn root() -> &'static str {
    "This is the root, you shouldn't be here"
}

async fn hello_world() -> &'static str {
    "Hello World"
}
