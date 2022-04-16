use crate::model::{Download, DownloadId, SubDownloadId, SubDownload};

pub mod in_memory;

pub trait DownloadStore {
    // todo: consider using a static result type
    fn download_by_id(&self, id: DownloadId) -> color_eyre::Result<Option<Download>>;
    fn download_by_uri(&self, uri: hyper::Uri) -> color_eyre::Result<Option<Vec<Download>>>;
    fn store_download(&self, download: &Download) -> color_eyre::Result<()>;
}

pub trait SubDownloadStore {
    fn sub_download_by_parent(&self, parent: DownloadId) -> color_eyre::Result<Option<Vec<SubDownload>>>;
    fn sub_download_by_id(&self, id: SubDownloadId) -> color_eyre::Result<Option<SubDownload>>;
    fn sub_download_by_uri(&self, uri: hyper::Uri) -> color_eyre::Result<Option<Vec<SubDownload>>>;
    fn store_sub_download(&self, sub_download: &SubDownload) -> color_eyre::Result<()>;
}
