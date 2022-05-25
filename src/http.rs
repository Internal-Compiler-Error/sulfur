use std::{error::{Error},
          fmt::{Display, Formatter, Debug},
          path::{Path, PathBuf},
          sync::{Arc},
          num::{ParseIntError},
          future::Future,
          ops::DerefMut,
};
use derive_more::{Display, Error};
use diesel::{ExpressionMethods,
             QueryDsl,
             r2d2, r2d2::{ConnectionManager, Pool},
             RunQueryDsl,
             SqliteConnection};
use tokio::{task::{JoinHandle},
            io::{AsyncWriteExt, AsyncSeekExt, SeekFrom},
            sync::Mutex,
            fs::OpenOptions,
            select};
use hyper_rustls::HttpsConnectorBuilder;
use futures::future::join_all;
use ::url::Url as WgUrl;
use hyper::{Body,
            Client,
            Request,
            client::{connect::Connect, HttpConnector},
            body::HttpBody};
use crate::{schema::*, request::http::{HttpRequestSource, HttpRequest}, util::{last_insert_rowid, EnableForeignKeys}};

#[derive(Debug, Display, Error)]
enum ParseError {
    UnsupportedSchema
}

#[derive(Debug)]
pub struct SubDownload {
    // having an id here directly is not great, but without it, we don't know which row to update
    // multiple subdownloads will share the same file path and url; offset and total will be changed
    // by then so we can't do a look up with them. Having an id is the easiest way to solve this
    // issue.
    pub id: i32,
    pub url: Arc<WgUrl>,
    pub offset: usize,
    pub total: usize,
    pub file_path: Arc<Path>,
}


#[derive(Debug, Queryable, Insertable, Identifiable, Associations)]
#[table_name = "file_path"]
#[primary_key(path)]
struct PathTable {
    pub path: String,
}

#[derive(Debug)]
struct PathConversionError {
    invalid_path: bool,
    not_file: bool,
}

impl Display for PathConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path_err = if self.invalid_path { "Invalid path" } else { " " };
        let file_err = if self.not_file { "Not a file" } else { " " };
        let err_msg = [path_err, file_err].join(" and ");
        write!(f, "{}", err_msg.trim())
    }
}

impl Error for PathConversionError {}

impl TryInto<PathBuf> for PathTable {
    type Error = PathConversionError;

    fn try_into(self) -> Result<PathBuf, Self::Error> {
        let mut path = PathBuf::new();
        path.push(self.path);

        let conversion_err = PathConversionError {
            invalid_path: !path.exists(),
            not_file: !path.is_file(),
        };

        if !conversion_err.not_file && !conversion_err.invalid_path {
            Ok(path)
        } else {
            Err(conversion_err)
        }
    }
}

impl From<&PathBuf> for PathTable {
    fn from(path: &PathBuf) -> Self {
        PathTable {
            path: path.to_str().unwrap().to_string(),
        }
    }
}


#[derive(Debug, Queryable, Insertable, Identifiable, Associations)]
#[table_name = "url"]
#[primary_key(full_text)]
struct UrlTable {
    pub full_text: String,
}

impl From<&WgUrl> for UrlTable {
    fn from(url: &WgUrl) -> Self {
        UrlTable {
            full_text: url.to_string(),
        }
    }
}

impl TryInto<WgUrl> for UrlTable {
    type Error = ::url::ParseError;

    fn try_into(self) -> Result<WgUrl, Self::Error> {
        WgUrl::parse(&self.full_text)
    }
}

#[derive(Debug, Queryable, Insertable, Identifiable, Associations, AsChangeset)]
#[table_name = "http_subdownload"]
#[belongs_to(UrlTable, foreign_key = "url")]
struct SubDownloadTable {
    pub id: i32,
    pub url: String,
    pub offset: i32,
    pub total: i32,
    pub file_path: String,
}


#[derive(Debug, Display, Error)]
enum StoreError {
    NotFound,
}

pub trait DownloadStore: Debug {
    fn add_download(&self, download: &SubDownload) -> Result<i32, diesel::result::Error>;
    // todo: perhaps it should take an id?
    fn update_download(&self, download: &SubDownload) -> Result<(), diesel::result::Error>;
    fn downloads_by_url(&self, wg_url: &WgUrl) -> Result<Vec<SubDownload>, diesel::result::Error>;
    fn remove_by_url(&self, wg_url: &WgUrl) -> Result<(), diesel::result::Error>;
    fn remove_by_id(&self, id: i32) -> Result<(), diesel::result::Error>;
}


pub type SharedDownloadStore = Arc<dyn DownloadStore + Send + Sync>;

struct SqliteStore {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

#[allow(unused_variables)]
impl Debug for SqliteStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!("maybe just print the entire state of the database?")
    }
}

impl SqliteStore {
    pub fn new(database_url: &str) -> Result<Self, r2d2::PoolError> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(32)
            .connection_customizer(Box::new(EnableForeignKeys::new()))
            .build(manager)?;

        Ok(Self {
            pool,
        })
    }
}

impl DownloadStore for SqliteStore {
    fn add_download(&self, download: &SubDownload) -> Result<i32, diesel::result::Error> {
        // find if the url already exists
        let conn = self.pool.get().expect("Failed to get connection");

        use crate::schema::url::dsl::*;
        let url_query = url.filter(full_text.eq(download.url.as_str())).limit(1);

        // find if the file path already exists
        use crate::schema::file_path::dsl::*;
        let file_path_query = file_path.filter(path.eq((*download.file_path).as_os_str().to_string_lossy())).limit(1);


        conn.exclusive_transaction(|| {
            let url_res = url_query.first::<UrlTable>(&conn);
            let file_path_res = file_path_query.first::<PathTable>(&conn);


            match (url_res, file_path_res) {
                // they both exist, then we can just add the download
                (Ok(_), Ok(_)) => {}
                // both of them don't exist, we need to crate them first to fulfill the relationship
                (Err(diesel::result::Error::NotFound), Err(diesel::result::Error::NotFound)) => {
                    let url_table = UrlTable { full_text: download.url.to_string() };
                    let file_path_table = PathTable { path: (*download.file_path).to_string_lossy().to_string() };

                    // I don't know why all the sudden I need to use the full path here, and I hate it
                    diesel::insert_into(crate::schema::url::dsl::url).values(url_table).execute(&conn)?;
                    diesel::insert_into(crate::schema::file_path::dsl::file_path).values(file_path_table).execute(&conn)?;
                }

                // if somehow only one of them exist, then the program is already broken
                (a, b) => {
                    eprintln!("{:?}, {:?}", a, b);
                    unreachable!()
                }
            };

            use crate::schema::http_subdownload::dsl::*;
            let insert = diesel::insert_into(http_subdownload)
                .values((url.eq(download.url.as_str()),
                         offset.eq(download.offset as i32),
                         total.eq(download.total as i32),
                         file_path.eq(download.file_path.to_str().expect("file path is not utf8????"))));


            insert.execute(&conn)?;

            let row_id = diesel::select(last_insert_rowid).first(&conn)?;

            Ok(row_id)
        })
    }

    fn update_download(&self, download: &SubDownload) -> Result<(), diesel::result::Error> {
        let conn = self.pool.get().expect("Failed to get connection");

        // construct the table equivalent of download (the subdownload struct itself cannot be
        // directly used with diesel, and update it


        let sql_form = SubDownloadTable {
            id: download.id,
            url: download.url.to_string(),
            offset: download.offset as i32,
            total: download.total as i32,
            file_path: download.file_path.to_str().expect("file path is not utf8????").to_string(),
        };


        conn.exclusive_transaction(|| {
            use crate::schema::http_subdownload::dsl::*;
            diesel::update(http_subdownload).set(&sql_form).execute(&conn)?;

            Ok(())
        })
    }

    fn downloads_by_url(&self, wg_url: &WgUrl) -> Result<Vec<SubDownload>, diesel::result::Error> {
        let conn = self.pool.get().expect("Failed to get connection");

        conn.exclusive_transaction(|| {
            use crate::schema::http_subdownload::dsl::*;

            let result = http_subdownload.filter(url.eq(wg_url.as_str())).load::<SubDownloadTable>(&conn)?;
            // assume result is not empty, if it is, then it should be an error and already returned

            let url_p = Arc::new(WgUrl::parse(&result.first().unwrap().url).expect("database corrupted, url should be valid"));


            let mut path = PathBuf::new();
            path.push(&result.first().unwrap().file_path);


            let path = Arc::from(path);


            Ok(
                result.iter().map(|x| {
                    SubDownload {
                        id: x.id,
                        url: Arc::clone(&url_p),
                        offset: x.offset as usize,
                        total: x.total as usize,
                        file_path: Arc::clone(&path),
                    }
                }).collect::<Vec<_>>()
            )
        })
    }

    fn remove_by_url(&self, wg_url: &WgUrl) -> Result<(), diesel::result::Error> {
        use crate::schema::http_subdownload::dsl::*;

        // delete cascade will delete all subdownloads
        let matching_rows = http_subdownload.filter(url.eq(wg_url.as_str()));
        let conn = self.pool.get().expect("Failed to get connection");
        (*conn).exclusive_transaction(|| {
            diesel::delete(matching_rows).execute(&conn)?;
            Ok(())
        })
    }

    fn remove_by_id(&self, id: i32) -> Result<(), diesel::result::Error> {
        let identification = id;

        {
            use crate::schema::http_subdownload::dsl::*;

            // delete the subdownload by id
            let matching_rows = http_subdownload.filter(id.eq(identification));
            let conn = self.pool.get().expect("Failed to get connection");
            (*conn).exclusive_transaction(|| {
                diesel::delete(matching_rows).execute(&conn)?;
                Ok(())
            })
        }
    }
}


/// Given a size, split into [begin, end) intervals suitable for parallel downloads by the number of
/// CPU cores
fn split_range(size: usize) -> Vec<(usize, usize)> {
    let mut ranges = vec![];
    let cores = num_cpus::get();
    let each_size = size / cores;
    let remainder = size % cores;

    // each cores gets the total size / core
    for i in 0..cores - 1 {
        ranges.push((each_size * i, each_size * (i + 1)))
    }

    // except the last core, which need to handle the remainder
    ranges.push((each_size * cores, each_size * cores + remainder));

    ranges
}

#[derive(Debug)]
pub struct HttpDownloader<R>
    where R: HttpRequestSource + Send + Sync
{
    request_source: Mutex<R>,
    download_store: SharedDownloadStore,
    current_downloads: Mutex<Vec<JoinHandle<()>>>,
}

/// The errors that could occur when we try to download a file in parallel
#[derive(Debug)]
pub enum HttpDownloaderError {
    ContentLengthNotSupported,
    BadServer,
    Other(String),
}


impl Display for HttpDownloaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            HttpDownloaderError::ContentLengthNotSupported => { write!(f, "Content-Length header not supported") }
            HttpDownloaderError::BadServer => { write!(f, "The server returned something we can't continue with") }
            HttpDownloaderError::Other(reason) => { write!(f, "generic error: {}", reason) }
        }
    }
}

impl Error for HttpDownloaderError {}

impl From<hyper::Error> for HttpDownloaderError
{
    fn from(e: hyper::Error) -> Self {
        HttpDownloaderError::Other(format!("{}", e))
    }
}

impl From<hyper::header::ToStrError> for HttpDownloaderError
{
    fn from(e: hyper::header::ToStrError) -> Self {
        HttpDownloaderError::Other(format!("{}", e))
    }
}

impl From<ParseIntError> for HttpDownloaderError
{
    fn from(e: ParseIntError) -> Self {
        HttpDownloaderError::Other(format!("{}", e))
    }
}

impl From<std::io::Error> for HttpDownloaderError
{
    fn from(e: std::io::Error) -> Self {
        HttpDownloaderError::Other(format!("{}", e))
    }
}

impl From<diesel::result::Error> for HttpDownloaderError
{
    fn from(e: diesel::result::Error) -> Self {
        HttpDownloaderError::Other(format!("{}", e))
    }
}

impl<R> HttpDownloader<R>
    where R: HttpRequestSource + Send + Sync + 'static
{
    fn new(request_source: R, shared_store: SharedDownloadStore) -> Self {
        HttpDownloader {
            request_source: Mutex::new(request_source),
            download_store: shared_store,
            current_downloads: Mutex::new(vec![]),
        }
    }

    async fn lifetime_loop(self: &Arc<Self>, stop_token: tokio::sync::oneshot::Receiver<bool>) {
        let http_connector = HttpConnector::new();
        let https_connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .enable_http2()
            .build();

        loop {
            let _ = select! {
                _stop = async { stop_token.await } => {
                    return;
                },
                _huh = async {
                    let request = self.request_source.lock().await.get_request().await.unwrap();
                    let file_sink = OpenOptions::new()
                                                .write(true)
                                                .create(true)
                                                .open(&*request.path)
                                                .await
                    .unwrap();

                    let file_sink = Arc::new(Mutex::new(file_sink));

                    if request.url.scheme() == "https" {
                        let task = self.spawn_downloads(request, file_sink, https_connector.clone()).await.unwrap();

                        self
                        .current_downloads
                        .lock()
                        .await
                        .push (task);
                    } else if request.url.scheme() == "http" {
                        let task = self.spawn_downloads(request, file_sink, http_connector.clone()).await.unwrap();

                        self
                        .current_downloads
                        .lock()
                        .await
                        .push (task);
                    } else {
                        panic!("unsupported schema");
                    }
                } => { return; },
            };
        }
    }


    /// Given a http download request and a sink, split the download into multiple subdownloads and
    /// start
    pub async fn spawn_downloads<S, C>(self: &Arc<Self>, download_request: HttpRequest, sink: Arc<Mutex<S>>, connector: C) -> Result<JoinHandle<()>, HttpDownloaderError>
        where S: AsyncWriteExt + AsyncSeekExt + Send + Unpin + 'static,
              C: Connect + Clone + Send + Sync + 'static
    {
        // get the file size
        let size = {
            let client = Client::builder()
                .build::<_, Body>(connector.clone());

            // the HTTP head method will return just the head of the HTTP response, we hope it will
            // return the Content-Length header to allow us to split the download into multiple
            let request = Request::head(download_request.url.as_str())
                .body(Body::empty())
                .unwrap();

            client.request(request)
                .await?
                .headers()
                .get("Content-Length")
                .ok_or(HttpDownloaderError::ContentLengthNotSupported)?
                .to_str()?
                .parse::<usize>()?
        };

        // split the download into multiple subdownloads by the number of cores
        let ranges = split_range(size);

        let url = Arc::new(download_request.url.clone());
        let file_path: Arc<Path> = Arc::from(download_request.path.clone());

        // convert the ranges to sub_downloads that we can store in the database
        let downloads: Vec<SubDownload> =
            ranges.into_iter().map(|range| {
                SubDownload {
                    id: -1,
                    url: url.clone(),
                    offset: range.0,
                    total: range.1 - range.0,
                    file_path: file_path.clone(),
                }
            }).map(|mut sub_download| {
                // todo: allow storing to fail, the challenge here is that `?` inside the map method
                // doesn't work since the return type is not Option<T> or Result<T, E>
                let id = self.
                    download_store
                    .add_download(&sub_download)
                    .unwrap();

                sub_download.id = id;
                sub_download
            }).collect();


        // download the file in parallel
        let mut download_tasks = vec![];

        for download in downloads {
            download_tasks.push(tokio::spawn(Self::chunked_download(self.clone(), download, sink.clone(), connector.clone())));
        }


        Ok(tokio::spawn(async { join_all(download_tasks).await; }))
    }


    fn chunked_download<S, C>(self: Arc<Self>, mut download: SubDownload, sink: Arc<Mutex<S>>, connector: C) -> impl Future<Output=Result<(), HttpDownloaderError>>
        where S: AsyncSeekExt + AsyncWriteExt + Send + Unpin + 'static,
              C: Connect + Clone + Send + Sync + 'static
    {
        let this = self.clone();
        async move {
            let request = Request::builder()
                .method("GET")
                .uri(download.url.as_str())
                .header("Range", format!("bytes={}-{}", download.offset, download.offset + download.total - 1))
                .body(Body::empty())
                .unwrap();


            let client = Client::builder()
                .build::<_, Body>(connector.clone());

            let mut body = client.request(request)
                .await?
                .into_body();


            // start downloading chunk by chunk
            while let Some(chunk) = body.data().await {
                let chunk = chunk?;
                {
                    let mut guard = sink
                        .lock()
                        .await;

                    let sink = guard.deref_mut();


                    // make sure to seek to the correct position
                    sink.seek(SeekFrom::Start(download.offset as u64)).await?;

                    // then we can write
                    sink.write_all(&chunk).await?;
                }
                // update the offset
                download.offset += chunk.len();

                // update the database so query knows the most recent truth
                this
                    .download_store
                    .update_download(&download)?
            }


            // we're done our chunk, remove ourselves from the store
            self.download_store.remove_by_id(download.id)?;

            Ok(())
        }
    }
}


#[cfg(test)]
mod tests {
    use std::env::current_dir;
    use std::thread::spawn;
    use std::time::Duration;
    use super::*;
    use tokio::sync::{mpsc, oneshot};
    use tokio::{join, time};
    use crate::request::http::ChannelHttpRequestSource;

    embed_migrations!("migrations");


    fn init_db() -> color_eyre::Result<SqliteStore> {
        let store = SqliteStore::new(":memory:")?;
        embedded_migrations::run(&store.pool.get()?)?;

        Ok(store)
    }

    #[test]
    fn sqlite_store_can_add() -> color_eyre::Result<()> {
        use pretty_assertions::assert_ne;

        let store = init_db()?;
        color_eyre::install()?;

        let url = Arc::from(WgUrl::parse("https://www.google.com")?);
        let file_path: Arc<Path> = Arc::from(PathBuf::from("/tmp/test.txt"));

        let download = SubDownload {
            id: -90999,
            url: Arc::clone(&url),
            offset: 0,
            total: 5000,
            file_path: Arc::clone(&file_path),
        };

        let res1 = store.add_download(&download)?;

        let download = SubDownload {
            // we do a bit of trolling
            id: -0,
            url: Arc::clone(&url),
            offset: 0,
            total: 5000,
            file_path: Arc::clone(&file_path),
        };

        let res2 = store.add_download(&download)?;

        assert_ne!(res1, res2);

        Ok(())
    }

    #[test]
    fn sqlite_store_remove_by_url_cleans_itself() -> color_eyre::Result<()> {
        color_eyre::install()?;

        let store = init_db()?;

        let url = Arc::from(WgUrl::parse("https://www.google.com")?);
        let file_path: Arc<Path> = Arc::from(PathBuf::from("/tmp/test.txt"));

        let download = SubDownload {
            id: -90999,
            url: Arc::clone(&url),
            offset: 0,
            total: 5000,
            file_path: Arc::clone(&file_path),
        };

        store.add_download(&download)?;

        let download = SubDownload {
            // we do a bit of trolling
            id: -0,
            url: Arc::clone(&url),
            offset: 0,
            total: 5000,
            file_path: Arc::clone(&file_path),
        };

        store.add_download(&download)?;
        store.remove_by_url(&url)?;
        {
            use crate::schema::http_subdownload::dsl::*;

            let result = http_subdownload.load::<SubDownloadTable>(&store.pool.get()?);
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());

            let result = crate::schema::url::dsl::url.load::<UrlTable>(&store.pool.get()?);
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }


        {
            use crate::schema::file_path::dsl::*;

            let result = file_path.load::<PathTable>(&store.pool.get()?);
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }


        Ok(())
    }

    #[test]
    fn sqlite_store_removes_by_id() -> color_eyre::Result<()> {
        use pretty_assertions::assert_eq;

        color_eyre::install()?;

        let store = init_db()?;

        let url = Arc::from(WgUrl::parse("https://www.google.com")?);
        let file_path: Arc<Path> = Arc::from(PathBuf::from("/tmp/test.txt"));

        let mut download1 = SubDownload {
            id: -90999,
            url: Arc::clone(&url),
            offset: 0,
            total: 5000,
            file_path: Arc::clone(&file_path),
        };

        download1.id = store.add_download(&download1)?;


        let mut download2 = SubDownload {
            // we do a bit of trolling
            id: -0,
            url: Arc::clone(&url),
            offset: 0,
            total: 5000,
            file_path: Arc::clone(&file_path),
        };

        download2.id = store.add_download(&download2)?;
        store.remove_by_id(download1.id)?;

        {
            use crate::schema::http_subdownload::dsl::*;

            let result = http_subdownload.load::<SubDownloadTable>(&store.pool.get()?)?;

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].id, download2.id);
        }


        Ok(())
    }

    #[tokio::test]
    async fn download_ubuntu_22_04() -> color_eyre::Result<()> {
        let (req_tx, req_rx) = mpsc::channel(100);
        let request_source = ChannelHttpRequestSource::new(req_rx);

        req_tx.send(HttpRequest {
            url: WgUrl::parse("https://releases.ubuntu.com/22.04/ubuntu-22.04-desktop-amd64.iso")?,
            path: current_dir().unwrap().join("ubuntu-22.04-desktop-amd64.iso"),
        })
            .await
            .unwrap();


        let store: SharedDownloadStore = Arc::new(init_db()?);

        let downloader = HttpDownloader::new(request_source, store);

        let (tx, rx) = oneshot::channel();

        let download_task = tokio::spawn(async move {
            let downloader = Arc::new(downloader);
            downloader.lifetime_loop(rx).await;
        });

        // limit the task to 10 minutes
        let time_limit = tokio::spawn(async move {
            time::sleep(Duration::from_secs(60 * 10)).await;
            tx.send(true)
        });

        join!(download_task, time_limit);

        Ok(())
    }
}
