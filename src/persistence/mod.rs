use std::error::Error;
use crate::model::{Download, DownloadId, SubDownloadId, SubDownload};
use crate::schema::sub_http_download::parent_id;

pub mod in_memory;
pub mod sqlite;


pub trait DownloadStore<E>
    where E: Error
{
    fn download_by_id(&self, id: DownloadId) -> Result<Option<Download>, E>;
    fn download_by_uri(&self, uri: hyper::Uri) -> Result<Option<Vec<Download>>, E>;
    fn store_download(&mut self, download: &Download) -> Result<(), E>;
    fn remove_download(&mut self, id: DownloadId) -> Result<(), E>;
}

pub trait SubDownloadStore<E>
    where E: Error
{
    fn sub_download_by_parent(&self, parent: DownloadId) -> Result<Option<Vec<SubDownload>>, E>;
    fn sub_download_by_id(&self, id: SubDownloadId) -> Result<Option<SubDownload>, E>;
    fn sub_download_by_uri(&self, uri: hyper::Uri) -> Result<Option<Vec<SubDownload>>, E>;
    fn store_sub_download(&mut self, sub_download: &SubDownload, download_id: DownloadId) -> Result<(), E>;
    fn remove_sub_download(&mut self, id: SubDownloadId) -> Result<(), E>;
}
