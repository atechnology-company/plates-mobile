use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use dotenv::dotenv;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
    pub image_url: Option<String>,
}

// Tauri command to fetch search results
#[tauri::command]
pub async fn fetch_search_results(query: String) -> Result<Vec<SearchResult>, String> {
    dotenv().ok();
    
    // Try to get API keys from environment variables
    let api_key = env::var("GOOGLE_API_KEY");
    let search_engine_id = env::var("GOOGLE_SEARCH_ENGINE_ID");
    
    // If API keys are not available, return mock data
    if api_key.is_err() || search_engine_id.is_err() {
        println!("Warning: Using mock data because API keys are not set");
        return fetch_mock_search_results(&query).await;
    }
    
    let api_key = api_key.unwrap();
    let search_engine_id = search_engine_id.unwrap();
    
    // Build the Google Custom Search API URL
    let url = format!(
        "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}&searchType=image",
        api_key, search_engine_id, urlencoding::encode(&query)
    );
    
    // Create HTTP client and send request
    let client = Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to Google Search API: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await
            .unwrap_or_else(|_| "Failed to get error response".to_string());
        println!("Google Search API error: {}", error_text);
        return fetch_mock_search_results(&query).await;
    }
    
    // Parse the response
    let search_response: GoogleSearchResponse = response.json().await
        .map_err(|e| format!("Failed to parse Google Search API response: {}", e))?;
    
    // Extract search results
    let items = match search_response.items {
        Some(items) => items,
        None => return Ok(vec![]), // No results found
    };
    
    // Convert API response to our SearchResult structure
    let results: Vec<SearchResult> = items.into_iter().map(|item| {
        // Extract image URL if available
        let pagemap_clone = item.pagemap.clone();
        let image_url = pagemap_clone
            .and_then(|pagemap| pagemap.cse_image)
            .and_then(|images| images.first().map(|img| img.src.clone()))
            .or_else(|| {
                item.pagemap
                    .and_then(|pagemap| pagemap.cse_thumbnail)
                    .and_then(|thumbnails| thumbnails.first().map(|thumb| thumb.src.clone()))
            });
        
        SearchResult {
            title: item.title,
            link: item.link,
            snippet: item.snippet,
            image_url,
        }
    }).collect();
    
    Ok(results)
}

// Function to generate mock search results when API is not available
pub async fn fetch_mock_search_results(query: &str) -> Result<Vec<SearchResult>, String> {
    let mut results = Vec::new();
    
    // Add some mock results
    results.push(SearchResult {
        title: format!("Search result 1 for {}", query),
        link: "https://example.com/result1".to_string(),
        snippet: "This is a description of the first search result. It provides a brief overview of what the page contains.".to_string(),
        image_url: Some("https://via.placeholder.com/120x90".to_string()),
    });
    
    results.push(SearchResult {
        title: format!("Search result 2 for {}", query),
        link: "https://example.com/result2".to_string(),
        snippet: "Another search result with different information. This one might be more relevant to your query.".to_string(),
        image_url: Some("https://via.placeholder.com/120x90".to_string()),
    });
    
    results.push(SearchResult {
        title: format!("Search result 3 for {}", query),
        link: "https://example.com/result3".to_string(),
        snippet: "A third search result with additional information about the topic you searched for.".to_string(),
        image_url: None,
    });
    
    results.push(SearchResult {
        title: format!("Search result 4 for {}", query),
        link: "https://example.com/result4".to_string(),
        snippet: "This result contains more detailed information about your search query and related topics.".to_string(),
        image_url: Some("https://via.placeholder.com/120x90".to_string()),
    });
    
    Ok(results)
}

// Google Search API response structures
#[derive(Deserialize, Debug)]
struct GoogleSearchResponse {
    items: Option<Vec<GoogleSearchItem>>,
}

#[derive(Deserialize, Debug, Clone)]
struct GoogleSearchItem {
    title: String,
    link: String,
    snippet: String,
    pagemap: Option<PageMap>,
}

#[derive(Deserialize, Debug, Clone)]
struct PageMap {
    cse_image: Option<Vec<CseImage>>,
    cse_thumbnail: Option<Vec<CseThumbnail>>,
}

#[derive(Deserialize, Debug, Clone)]
struct CseImage {
    src: String,
}

#[derive(Deserialize, Debug, Clone)]
struct CseThumbnail {
    src: String,
}

// Tauri command to open a link in the default browser
#[tauri::command]
pub async fn open_link(url: String) -> Result<(), String> {
    // Use tauri-plugin-opener to open the URL in the default browser
    tauri_plugin_opener::open(&url)
        .map_err(|e| format!("Failed to open URL: {}", e))
}