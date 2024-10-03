use std::path::Path;

use anyhow::Error;
use async_compression::tokio::bufread::GzipDecoder;
use futures_util::TryStreamExt;
use tokio::sync::mpsc::Sender;
use tokio_stream::StreamExt;
use tokio_tar::Archive;
use tokio_util::io::StreamReader;

pub async fn deploy(tx: &Sender<String>, url: String) -> Result<(), Error> {
    tx.send(format!("start deploy {url}\r\n")).await?;
    let res = reqwest::get(url.as_str()).await?;

    let stream = res.bytes_stream();
    let stream_reader =
        StreamReader::new(stream.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));
    let gzip_decoder = GzipDecoder::new(stream_reader);
    // let async_stream = ReaderStream::new(gzip_decoder);
    let mut tar_archive = Archive::new(gzip_decoder);
    let mut entries = tar_archive.entries()?;

    while let Some(entry) = entries.next().await {
        let mut entry = entry?;
        let path = format!("/dist/{}", entry.header().path()?.to_str().unwrap());
        let path = Path::new(&path);
        if entry.header().entry_type().is_dir() {
            tx.send(format!("create dir: {path:?}\r\n")).await?;
            tokio::fs::create_dir_all(path).await?;
        } else {
            tx.send(format!("write file: {path:?}\r\n")).await?;
            let mut file = tokio::fs::File::create(path).await?;
            tokio::io::copy(&mut entry, &mut file).await?;
        }
    }
    tx.send("deploy success\r\n".to_string()).await?;

    Ok(())
}
