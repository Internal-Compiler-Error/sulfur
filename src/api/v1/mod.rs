



use axum::extract::Query;

use serde::Deserialize;

use url::{Url};







#[allow(unused)]/// Maps to /api/v1/download
pub async fn new_download(req: Query<DownloadReq>) -> color_eyre::Result<String> {
    // let client = Client::new();
    // let Url = req.0.Url;
    //
    // let url_clone = Url.clone();
    // let request = client.get(Url).build()?;
    //
    // client.execute(request).await?;
    //
    // Ok(format!("you passed in {}", url_clone))
    todo!()
}

pub struct DownloadResponse {
    message: String,
}

#[derive(Deserialize)]
pub struct DownloadReq {
    url: Url,
}
