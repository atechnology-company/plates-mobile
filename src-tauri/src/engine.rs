use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::env;
use reqwest::Client;
use dotenv::dotenv;

// Gemini API response structures
#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize, Debug)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Deserialize, Debug)]
struct Part {
    text: String,
}

// Request structures for Gemini API
#[derive(Serialize, Debug)]
struct GeminiRequest {
    contents: Vec<RequestContent>,
}

#[derive(Serialize, Debug)]
struct RequestContent {
    parts: Vec<RequestPart>,
}

#[derive(Serialize, Debug)]
struct RequestPart {
    text: String,
}

// Initialize the Gemini API client
pub struct GeminiClient {
    api_key: String,
    client: Client,
}

impl GeminiClient {
    pub fn new() -> Result<Self, String> {
        dotenv().ok();
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| "GEMINI_API_KEY not found in environment variables".to_string())?;
        
        Ok(Self {
            api_key,
            client: Client::new(),
        })
    }
    
    pub async fn generate_response(&self, prompt: &str) -> Result<String, String> {
        let request = GeminiRequest {
            contents: vec![RequestContent {
                parts: vec![RequestPart {
                    text: prompt.to_string(),
                }],
            }],
        };
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}",
            self.api_key
        );
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Gemini API: {}", e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Failed to get error response".to_string());
            return Err(format!("Gemini API error: {}", error_text));
        }
        
        let gemini_response: GeminiResponse = response.json().await
            .map_err(|e| format!("Failed to parse Gemini API response: {}", e))?;
        
        // Extract the text from the response
        if let Some(candidate) = gemini_response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }
        
        Err("No response text found in Gemini API response".to_string())
    }
}

#[tauri::command]
pub async fn process_text_input(text: String) -> Result<String, String> {
    println!("Received text input: {}", text);
    
    // Initialize the Gemini client
    let gemini_client = GeminiClient::new()?;
    
    // Send the text to the Gemini API and get the response
    let response = gemini_client.generate_response(&text).await?;
    
    Ok(response)
}

// Make sure to register this command in your main.rs file:
// .invoke_handler(tauri::generate_handler![engine::process_text_input])