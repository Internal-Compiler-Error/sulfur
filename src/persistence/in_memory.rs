use std::fmt::Display;
use multimap::MultiMap;
use super::*;

#[derive(Debug)]
struct HttpInMemoryStore {
    data: Vec<Download>,
}

#[derive(Debug)]
/// Error type for in memory database, it never fails.
struct Never {}

impl Display for Never {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

impl Error for Never {}

#[derive(Debug)]
struct SubHttpInMemoryStore {
    data: Vec<SubDownload>,
    parent_id_to_sub_download: MultiMap<DownloadId, SubDownloadId>,
}

impl HttpInMemoryStore {
    pub fn new() -> Self {
        HttpInMemoryStore {
            data: Vec::new(),
        }
    }
}

impl DownloadStore<Never> for HttpInMemoryStore {
    fn download_by_id(&self, id: DownloadId) -> Result<Option<Download>, Never> {
        self.data
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .map(|d| Some(d))
            .ok_or_else(|| {
                unreachable!()
            })
    }

    fn download_by_uri(&self, uri: hyper::Uri) -> Result<Option<Vec<Download>>, Never> {
        let downloads = self.data
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
    fn store_download(&mut self, download: &Download) -> Result<(), Never> {
        let found = self.data
            .iter_mut()
            .find(|d| d.id == download.id);

        match found {
            None => {
                self.data.push(download.clone());
                Ok(())
            }
            Some(target) => {
                *target = download.clone();
                Ok(())
            }
        }
    }

    fn remove_download(&mut self, id: DownloadId) -> Result<(), Never> {
        let index = self.data
            .iter()
            .position(|d| d.id == id);

        match index {
            None => Ok(()),
            Some(index) => {
                self.data.remove(index);
                Ok(())
            }
        }
    }
}

impl SubDownloadStore<Never> for SubHttpInMemoryStore {
    fn sub_download_by_parent(&self, parent: DownloadId) -> Result<Option<Vec<SubDownload>>, Never> {
        // find all sub downloads id with the given parent id
        let sub_download_ids = self.parent_id_to_sub_download.get_vec(&parent);


        match sub_download_ids {
            None => {
                Ok(None)
            }
            Some(ids) => {
                // find all sub downloads with the given sub download id
                let id = ids.first().unwrap().clone();
                let sub_downloads: Vec<_> = self.data
                    .iter()
                    .filter(|d| d.id == id)
                    .cloned()
                    .collect();

                Ok(Some(sub_downloads))
            }
        }
    }

    fn sub_download_by_id(&self, id: SubDownloadId) -> Result<Option<SubDownload>, Never> {
        self.data
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .map(|d| Some(d))
            .ok_or_else(|| {
                unreachable!()
            })
    }

    fn sub_download_by_uri(&self, uri: hyper::Uri) -> Result<Option<Vec<SubDownload>>, Never> {
        let downloads = self.data
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

    fn store_sub_download(&mut self, sub_download: &SubDownload, download_id: DownloadId) -> Result<(), Never> {
        let target = self.data
            .iter_mut()
            .find(|d| d.id == sub_download.id);

        match target {
            None => {
                self.data.push(sub_download.clone());
                self.parent_id_to_sub_download.insert(download_id, sub_download.id);
                Ok(())
            }
            Some(target) => {
                *target = sub_download.clone();
                Ok(())
            }
        }
    }


    fn remove_sub_download(&mut self, id: SubDownloadId) -> Result<(), Never> {
        let index = self.data
            .iter()
            .position(|d| d.id == id);

        match index {
            None => Ok(()),
            Some(index) => {
                self.data.remove(index);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;
    use crate::model::*;
    use std::env::current_dir;

    #[test]
    fn http_in_memory_basic_crud() {
        let mut store = HttpInMemoryStore::new();

        let domains = vec!["google.ca".to_string(),
                           "yahoo.com".to_string(),
                           "https://releases.ubuntu.com/21.10/ubuntu-21.10-desktop-amd64.iso".to_string(),
                           "https://diesel.rs/guides/getting-started.html".to_string(),
                           "https://download.mozilla.org/?product=firefox-latest-ssl&os=linux64&lang=en-CA".to_string()];

        let mut domains: Vec<_> = domains
            .iter()
            .map(|domain| hyper::Uri::from_str(domain).unwrap())
            .collect();

        for (index, domain) in domains.drain(..).enumerate() {
            let download = Download::new(index as DownloadId, domain, 0f64, current_dir().unwrap());
            store.store_download(&download).unwrap();


            assert_eq!(store.download_by_id(download.id).unwrap(), Some(download.clone()));
            assert_eq!(store.download_by_uri(domain.clone()).unwrap(), Some(vec![download.clone()]));
        }
    }
}
