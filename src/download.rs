#![allow(dead_code)]

use crate::download::Error::{IoErr, ReqwestErr};
use futures::StreamExt;
use reqwest::Client;
use std::io;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinHandle;
use url::Url;
use serde::Deserialize;


pub enum Error {
    ReqwestErr(reqwest::Error),
    IoErr(io::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        ReqwestErr(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        IoErr(err)
    }
}

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

pub fn download(
    link: reqwest::Url,
    range: (usize, usize),
    path: PathBuf,
) -> Result<JoinHandle<Result<(), Error>>, Error> {
    let client = Client::new();

    let req = client
        .get(link)
        .header("range", format!("{}-{}", range.0, range.1))
        .build()?;

    let handle = tokio::spawn(async move {
        let mut file = File::create(path).await?;

        let res = client.execute(req).await?;
        let mut stream = res.bytes_stream();

        while let Some(bytes) = stream.next().await {
            file.write(&bytes?).await?;
        }

        Ok::<(), Error>(())
    });

    Ok(handle)
}
