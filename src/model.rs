use std::path::PathBuf;

pub type DownloadId = u64;
pub type SubDownloadId = u64;

#[derive(Debug, Clone, Queryable)]
pub struct Download {
    pub id: DownloadId,
    pub uri: hyper::Uri,
    pub progress: f64,
    pub path: PathBuf,
    // TODO: figure out if having a list of subdownloads here is a good idea
}

#[derive(Debug, Clone, Queryable)]
pub struct SubDownload {
    pub id: SubDownloadId,
    pub parent_id: DownloadId,

    /// offset denotes where should the file be seeked to before writing new data
    pub offset: u32,

    pub uri: hyper::Uri,
    pub progress: f64,
}
