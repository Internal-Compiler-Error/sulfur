use std::ops::DerefMut;
use std::sync::{Arc, RwLock};
use axum::http::Uri;
use diesel::prelude::*;
use crate::model::*;
use std::ops::Deref;
use diesel::r2d2::{ConnectionManager, Pool};
use crate::model::{Download, DownloadId};
use crate::persistence::DownloadStore;

struct SqlitePersistence {
    pub(crate) pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl SqlitePersistence {
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder().build(manager).expect("Failed to create pool.");

        SqlitePersistence { pool }
    }
}

struct SqliteDownloadStore {
    conn: SqlitePersistence,
}

impl SqliteDownloadStore {
    pub fn new(conn: SqlitePersistence) -> Self {
        SqliteDownloadStore { conn }
    }
}

// impl DownloadStore for SqliteDownloadStore {
//     fn download_by_id(&self, id: DownloadId) -> color_eyre::Result<Option<Download>> {
//         let conn = &self.conn;
//         let mut conn = &mut conn.pool.get().unwrap();
//
//         use crate::schema::http_download::dsl::*;
//         let mut conn = conn.deref_mut();
//         http_download.filter(id.eq(id))
//             .first::<DownloadSqlForm>(conn)
//             .map(|download| download.into())
//             .optional()
//             .map_err(|e| e.into())
//     }
//
//     fn download_by_uri(&self, uri: Uri) -> color_eyre::Result<Option<Vec<Download>>> {
//         todo!()
//     }
//
//     fn store_download(&self, download: &Download) -> color_eyre::Result<()> {
//         todo!()
//     }
// }
