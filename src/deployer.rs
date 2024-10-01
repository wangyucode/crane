use std::error::Error;

use tokio::sync::mpsc::Sender;

pub struct Deployer {
    sender: Sender<String>,
}

// Deployer 的构造器
impl Deployer {
    pub fn new(sender: Sender<String>) -> Self {
        Self { sender }
    }

    // 模拟的异步下载方法，发送进度到 tx
    pub async fn download(&self, url: &str) -> Result<(), Box<dyn Error>> {
        // 模拟下载过程中的进度更新
        for i in 0..=100 {
            self.sender.send(format!("{}%\r\n", i)).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await; // 模拟延迟
            if i == 100 {
                break;
            }
        }
        self.sender.send("Download complete".to_string()).await?;
        Ok(())
    }
}
