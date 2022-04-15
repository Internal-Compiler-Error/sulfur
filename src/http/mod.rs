use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use axum::handler::Handler;
use color_eyre::Report;
use hyper::{Body, client::Client, http::uri::Scheme, Request};
use hyper::client::connect::Connect;
use futures::future::join_all;


#[derive(Debug)]
enum Error {
    UnsupportedSchema
}


impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnsupportedSchema => {
                f.write_str(&format!("{self:?}"))
            }
        }
    }
}

impl std::error::Error for Error {}


#[derive(Debug, Default)]
struct HttpDownloader {
    current_downloads: Vec<color_eyre::Result<()>>,
}


/// Given a size, split into [begin, end) intervals suitable for parallel downloads by the number of
/// CPU cores
pub fn split_range(size: usize) -> Vec<(usize, usize)> {
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

pub async fn par_download<C>(
    link: hyper::Uri,
    path: PathBuf,
    connector: C,
) -> color_eyre::Result<()>
    where C: Connect + Clone + Send + Sync + 'static,
{
    let scheme = link.scheme().unwrap();

    if *scheme != Scheme::HTTP || *scheme != Scheme::HTTPS {
        return Err(Report::new(Error::UnsupportedSchema));
    }

    // get the file size
    let size = {
        let client = Client::builder()
            .build::<_, Body>(connector.clone());
        let res_body = Body::empty();
        let request = Request::head(link.clone())
            .body(res_body)
            .unwrap();

        client.request(request)
            .await?
            .headers()
            .get("Content-Length")
            .ok_or(Report::new(Error::UnsupportedSchema))?
            .to_str()
            .unwrap().parse::<usize>()?
    };

    let ranges = split_range(size);


    // download the file in parallel
    let mut download_tasks = vec![];
    for range in ranges {
        let client = Client::builder()
            .build::<_, Body>(connector.clone());

        let request = Request::builder()
            .method("GET")
            .uri(link.clone())
            .header("Range", format!("bytes={}-{}", range.0, range.1))
            .body(Body::empty())
            .unwrap();


        let task = tokio::spawn(async move {
            let body = client.request(request)
                .await
                .unwrap()
                .into_body();

            println!("{body:?}");
        });

        download_tasks.push(task);
    }


    join_all(download_tasks).await;
    Ok(())
}
