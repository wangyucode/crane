use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Error;
use http_body_util::{BodyExt, Empty};
use hyper::{body::Bytes, header::HeaderValue, Uri};
use hyper_tls::HttpsConnector;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use libflate::gzip::Decoder as GzipDecoder;
use tar::Archive as TarArchive;
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncWriteExt, BufWriter},
    sync::mpsc::Sender,
};

pub struct Deployer<'a> {
    pub sender: &'a Sender<String>,
    timestamp: u128,
}

impl<'a> Deployer<'a> {
    pub fn new(sender: &'a Sender<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        Self { sender, timestamp }
    }

    /// Download the given url and save it to /tmp/filename.tar.gz
    pub async fn download(&self, url: &str) -> Result<(), Error> {
        let https = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new()).build::<_, Empty<Bytes>>(https);

        let uri = Uri::from_str(url)?;
        let mut res = client.get(uri).await?;

        if !res.status().is_success() {
            return Err(Error::msg(res.status()));
        }

        // total size of the body
        let total_size = res
            .headers()
            .get("content-length")
            .unwrap_or(&HeaderValue::from_static("0"))
            .to_str()?
            .parse::<f64>()?;
        self.sender
            .send(format!(
                "Download started, total size: {:.0}\r\n",
                total_size
            ))
            .await?;

        let mut read_so_far = 0;
        // create the file
        let filename = format!("/tmp/{}.tar.gz", self.timestamp);
        //make sure the /tmp/ directory exists
        create_dir_all("/tmp/").await?;

        let file = File::create(filename).await?;
        let mut file_stream = BufWriter::new(file);
        // read the body and write it to file
        while let Some(next) = res.body_mut().frame().await {
            let frame = next?;
            if let Some(chunk) = frame.data_ref() {
                file_stream.write_all(chunk).await?;

                // update the progress
                read_so_far += chunk.len();
                let progress = format!(
                    "Download progress: {:.2}%\r\n",
                    (read_so_far as f64 / total_size as f64) * 100.0
                );
                self.sender.send(progress).await?;
            }
        }
        // flush and close the stream
        file_stream.shutdown().await?;
        // send success message
        self.sender.send("Download success!\r\n".to_string()).await?;
        Ok(())
    }

    /// extract the downloaded file to target `path`
    pub async fn deploy(&self, path: &str) -> Result<(), Error> {
        // open tar.gz file
        let filename = format!("/tmp/{}.tar.gz", self.timestamp);
        let file = std::fs::File::open(filename.as_str())?;
        let file_stream = std::io::BufReader::new(file);
        let gzip_decoder = GzipDecoder::new(file_stream)?;
        let mut tar_archive = TarArchive::new(gzip_decoder);
        self.sender.send(format!("unpack {}\r\n", filename)).await?;
        tar_archive.unpack(path)?;
        self.sender.send("unpack success!\r\n".to_string()).await?;
        Ok(())
    }
}
