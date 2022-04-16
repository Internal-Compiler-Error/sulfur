use std::cell::RefCell;
use color_eyre::eyre::eyre;
use std::collections::HashMap;
use super::*;

#[derive(Debug)]
struct HttpInMemoryStore {
    data: RefCell<Vec<Download>>,
}

#[derive(Debug)]
struct SubHttpInMemoryStore {
    data: RefCell<Vec<SubDownload>>,
    parent_id_to_sub_download: RefCell<HashMap<DownloadId, SubDownloadId>>,
}

impl HttpInMemoryStore {
    pub fn new() -> Self {
        HttpInMemoryStore {
            data: RefCell::new(Vec::new()),
        }
    }
}

impl DownloadStore for HttpInMemoryStore {
    fn download_by_id(&self, id: DownloadId) -> color_eyre::Result<Option<Download>> {
        self.data.borrow()
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .map(|d| Some(d))
            .ok_or_else(|| {
                eyre!("Download not found")
            })
    }
    fn download_by_uri(&self, uri: hyper::Uri) -> color_eyre::Result<Option<Vec<Download>>> {
        let downloads = self.data.borrow()
            .iter()
            .filter(|d| d.uri == uri)
            .cloned()
            .collect::<Vec<_>>();
        if downloads.is_empty() {
            Ok(None)
        } else {
            Ok(Some(downloads))
        }
    }
    fn store_download(&self, download: &Download) -> color_eyre::Result<()> {
        self.data.borrow_mut().push(download.clone());
        Ok(())
    }
}

impl SubDownloadStore for SubHttpInMemoryStore {
    fn sub_download_by_parent(&self, parent: DownloadId) -> color_eyre::Result<Option<Vec<SubDownload>>> {
        let sub_downloads = self.parent_id_to_sub_download.borrow()
            .entry(parent)
            .cloned()
            .collect::<Vec<_>>();
        if sub_downloads.is_empty() {
            Ok(None)
        } else {
            Ok(Some(sub_downloads))
        }
    }

    fn sub_download_by_id(&self, id: SubDownloadId) -> color_eyre::Result<Option<SubDownload>> {
        self.data.borrow()
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .map(|d| Some(d))
            .ok_or_else(|| {
                eyre!("SubDownload not found")
            })
    }
    fn sub_download_by_uri(&self, uri: hyper::Uri) -> color_eyre::Result<Option<Vec<SubDownload>>> {
        let downloads = self.data.borrow()
            .iter()
            .filter(|d| d.uri == uri)
            .cloned()
            .collect::<Vec<_>>();
        if downloads.is_empty() {
            Ok(None)
        } else {
            Ok(Some(downloads))
        }
    }
    fn store_sub_download(&self, sub_download: &SubDownload) -> color_eyre::Result<()> {
        self.data.borrow_mut().push(sub_download.clone());
        Ok(())
    }
}
