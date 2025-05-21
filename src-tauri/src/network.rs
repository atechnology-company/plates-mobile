use reqwest::Client;
use std::time::Duration;

pub struct NetworkDetector {
    client: Client,
}

impl NetworkDetector {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn is_online(&self) -> bool {
        // Try to connect to Google's DNS server
        match self.client
            .get("https://8.8.8.8")
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

#[tauri::command]
pub async fn check_network_status() -> Result<bool, String> {
    let detector = NetworkDetector::new();
    Ok(detector.is_online().await)
}