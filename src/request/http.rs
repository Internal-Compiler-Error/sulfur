use std::fmt::Debug;
use derive_more::{Display, Error};
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;
use url::Url;
use std::path::PathBuf;

#[derive(Debug)]
pub struct HttpRequest {
    pub url: Url,
    pub path: PathBuf,
}


#[async_trait]
pub trait HttpRequestSource: Debug {
    type Error: Debug;

    async fn get_request(&mut self) -> Result<HttpRequest, Self::Error>;
}


// see https://github.com/rust-lang/rust/issues/63063 for why impl is not allowed
pub type SharedHttpRequestSource<E> = Arc<dyn HttpRequestSource<Error=E> + 'static + Send + Sync>;

#[derive(Debug)]
pub struct ChannelHttpRequestSource {
    requests: Receiver<HttpRequest>,
}


impl ChannelHttpRequestSource {
    pub fn new(requests: Receiver<HttpRequest>) -> Self {
        Self {
            requests
        }
    }
}


#[derive(Debug, Display, Error)]
pub enum ChannelHttpRequestSourceError {
    ChannelClosed,
}


#[async_trait]
impl HttpRequestSource for ChannelHttpRequestSource {
    type Error = ChannelHttpRequestSourceError;

    async fn get_request(&mut self) -> Result<HttpRequest, ChannelHttpRequestSourceError> {
        if let Some(request) = self.requests.recv().await {
            Ok(request)
        } else {
            Err(ChannelHttpRequestSourceError::ChannelClosed)
        }
    }
}
