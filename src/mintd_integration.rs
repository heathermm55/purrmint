use std::process::Stdio;
use std::path::PathBuf;
use tokio::process::Child;
use serde_json::Value;
use tracing::{info, error};

pub struct MintdIntegration {
    child: Option<Child>,
    config_path: PathBuf,
    mintd_port: u16,
}

impl MintdIntegration {
    pub fn new(config_dir: PathBuf, port: u16) -> Self {
        Self {
            child: None,
            config_path: config_dir.join("mintd.toml"),
            mintd_port: port,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.child.is_some() {
            return Ok(());
        }

        let mintd_path = self.find_mintd_binary()?;
        info!("Starting mintd from path: {:?}", mintd_path);
        info!("Config path: {:?}", self.config_path);
        
        let mut child = tokio::process::Command::new(&mintd_path)
            .arg("--config")
            .arg(&self.config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        match child.try_wait() {
            Ok(Some(status)) => {
                error!("Mintd process exited with status: {}", status);
                
                // Try to read stderr for more details
                if let Some(stderr) = child.stderr.take() {
                    let mut stderr_content = String::new();
                    if let Ok(_) = tokio::io::AsyncReadExt::read_to_string(&mut tokio::io::BufReader::new(stderr), &mut stderr_content).await {
                        error!("Mintd stderr: {}", stderr_content);
                    }
                }
                
                return Err(Box::<dyn std::error::Error + Send + Sync>::from("Failed to start mintd".to_string()));
            }
            Ok(None) => {
                info!("Mintd process started successfully");
                self.child = Some(child);
                Ok(())
            }
            Err(e) => {
                error!("Error checking mintd process: {}", e);
                Err(Box::<dyn std::error::Error + Send + Sync>::from(e))
            }
        }
    }

    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(mut child) = self.child.take() {
            child.kill().await?;
            info!("Mintd process stopped");
        }
        Ok(())
    }

    pub async fn proxy_request(&self, endpoint: &str, payload: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("http://127.0.0.1:{}{}", self.mintd_port, endpoint);
        
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: Value = response.json().await?;
            Ok(result)
        } else {
            Err(Box::<dyn std::error::Error + Send + Sync>::from(format!("Mintd request failed: {}", response.status())))
        }
    }

    fn find_mintd_binary(&self) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let possible_paths = vec![
            // Android app internal binary path (extracted from assets)
            // Try both possible Android paths
            PathBuf::from("/data/data/com.example.purrmint/files/mintd"),
            PathBuf::from("/data/user/0/com.example.purrmint/files/mintd"),
            // Also try relative to config directory
            self.config_path.parent().unwrap_or(&PathBuf::from(".")).join("mintd"),
            // Standard paths
            PathBuf::from("mintd"),
            PathBuf::from("cdk-mintd"),
            PathBuf::from("/usr/local/bin/mintd"),
            PathBuf::from("/usr/local/bin/cdk-mintd"),
            PathBuf::from("/usr/bin/mintd"),
            PathBuf::from("/usr/bin/cdk-mintd"),
            PathBuf::from("./target/release/mintd"),
            PathBuf::from("./target/release/cdk-mintd"),
            PathBuf::from("../cdk/target/release/cdk-mintd"),
            PathBuf::from("../../cdk/target/release/cdk-mintd"),
        ];

        info!("Searching for mintd binary in possible paths:");
        for path in &possible_paths {
            info!("  Checking: {:?} (exists: {})", path, path.exists());
        }

        for path in possible_paths {
            if path.exists() {
                info!("Found mintd binary at: {:?}", path);
                return Ok(path);
            }
        }

        error!("mintd binary not found in any of the expected locations");
        Err(Box::<dyn std::error::Error + Send + Sync>::from("mintd binary not found. Please install CDK mintd first.".to_string()))
    }

    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }
}

impl Drop for MintdIntegration {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
    }
} 