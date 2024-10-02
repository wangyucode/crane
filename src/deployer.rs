use anyhow::Result;
use libflate::gzip;
use tokio::sync::mpsc::Sender;

pub async fn deploy(tx: &Sender<String>, url: String) -> Result<()> {
    tx.send(format!("start deploy {url}\r\n")).await?;
    let res = ureq::get(url.as_str()).call()?;
    let gzip_decoder = gzip::Decoder::new(res.into_reader())?;
    let mut tar_archive = tar::Archive::new(gzip_decoder);
    for entry in tar_archive.entries()? {
        let mut file = entry?;
        let path = format!(
            "/dist/{}",
            file.header().path()?.as_os_str().to_str().unwrap()
        );
        let msg = format!("deploying: {:?}, size: {}\r\n", path, file.header().size()?);
        println!("{msg}");
        tx.send(msg.clone()).await?;
        if let Some(parent) = file.path().unwrap().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut outfile = std::fs::File::create(&path)?;
        std::io::copy(&mut file, &mut outfile)?;
    }
    tx.send("deploy success\r\n".to_string()).await?;

    Ok(())
}
