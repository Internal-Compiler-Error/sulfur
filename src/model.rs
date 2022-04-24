use std::path::PathBuf;

pub type DownloadId = u64;
pub type DownloadIdSqlForm = i32;
pub type SubDownloadId = u64;
pub type SubDownloadIdSqlForm = i32;


/// A model of a download.
#[derive(Debug, Clone, PartialEq)]
pub struct Download {
    /// The ID of the download.
    pub id: DownloadId,

    /// uri of the http request, schema should be HTTP(S) but not checked
    pub uri: hyper::Uri,

    // the total progress of the download
    pub progress: f64,

    /// The path to the file.
    pub path: PathBuf,
    // TODO: figure out if having a list of subdownloads here is a good idea
}

impl Download {
    /// Create a new download.
    pub fn new(id: DownloadId, uri: hyper::Uri, progress: f64, path: PathBuf) -> Download {
        Download {
            id,
            uri,
            progress,
            path,
        }
    }
}


/// The model of download stored in the database, it doesn't use any "exotic" types such as `PathBuf`.
/// The type should not be used directly in most cases, but rather through the `Download` type.
#[derive(Debug, Clone, Queryable)]
pub struct DownloadSqlForm {
    pub id: DownloadIdSqlForm,
    pub uri: String,
    pub progress: f64,
    pub path: String,
}

impl DownloadSqlForm {
    /// Create a new download from the database form.
    pub fn new(id: DownloadIdSqlForm, uri: String, progress: f64, path: String) -> DownloadSqlForm {
        DownloadSqlForm {
            id,
            uri,
            progress,
            path,
        }
    }
}

impl Into<Download> for DownloadSqlForm {
    fn into(self) -> Download {
        Download {
            /// safety: always positive
            id: self.id as DownloadId,
            uri: self.uri.parse().unwrap(),
            progress: self.progress,
            path: PathBuf::from(self.path),
        }
    }
}


/// A model of a sub segment of a download.
#[derive(Debug, Clone, PartialEq)]
pub struct SubDownload {
    /// The ID of the sub download.
    pub id: SubDownloadId,

    /// The ID of the associated parent that represents the entire download .
    pub parent_id: DownloadId,

    /// offset denotes where should the file be seeked to before writing new data, this is intended
    /// to be modified after each new write
    pub offset: usize,

    /// the uri of the sub request, in practice this means it contains the range header
    pub uri: hyper::Uri,

    /// the process of the **sub** download
    pub progress: f64,
}

impl SubDownload {
    /// Create a new sub download.
    pub fn new(id: SubDownloadId, parent_id: DownloadId, offset: usize, uri: hyper::Uri, progress: f64) -> SubDownload {
        SubDownload {
            id,
            parent_id,
            offset,
            uri,
            progress,
        }
    }
}

/// A model of a sub download stored in the database, it doesn't use any "exotic" types such as `PathBuf`.
/// The type should not be used directly in most cases, but rather through the `SubDownload` type.
#[derive(Debug, Clone, Queryable)]
pub struct SubDownloadSqlForm {
    pub id: SubDownloadIdSqlForm,
    pub parent_id: DownloadId,
    pub offset: i64,
    pub uri: String,
    pub progress: f64,
}

impl SubDownloadSqlForm {
    /// Create a new sub download from the database form.
    pub fn new(id: SubDownloadIdSqlForm, parent_id: DownloadId, offset: i64, uri: String, progress: f64) -> SubDownloadSqlForm {
        SubDownloadSqlForm {
            id,
            parent_id,
            offset,
            uri,
            progress,
        }
    }
}

impl Into<SubDownload> for SubDownloadSqlForm {
    fn into(self) -> SubDownload {
        SubDownload {
            /// safety: always positive
            id: self.id as SubDownloadId,
            parent_id: self.parent_id,
            /// safety: offset is always positive
            offset: self.offset as usize,
            uri: self.uri.parse().unwrap(),
            progress: self.progress,
        }
    }
}
