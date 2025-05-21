


use tauri::Manager;
use serde::{Serialize, Deserialize};
use tauri_plugin_system_info::{commands::battery, model::{Battery, BatteryState}};
use reqwest;
use dotenv::dotenv;
use std::env;

// Import modules
mod talk;
mod engine;
mod network;
mod speech;
mod search;

// Import battery command from tauri_plugin_system_info
#[tauri::command]
async fn get_battery_info() -> Result<tauri_plugin_system_info::model::Battery, String> {
    tauri_plugin_system_info::commands::battery::get_battery()
        .map_err(|e| e.to_string())
}

// Define the greet command that was referenced but not implemented
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Command to check if this is the first run
#[tauri::command]
fn is_first_run(app_handle: tauri::AppHandle) -> bool {
    let path = app_handle.path().app_data_dir().unwrap();
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
    
    let first_run_file = path.join("first_run.txt");
    !first_run_file.exists()
}

// Command to mark tutorial as completed
#[tauri::command]
fn complete_tutorial(app_handle: tauri::AppHandle) -> Result<(), String> {
    let path = app_handle.path().app_data_dir().unwrap();
    let first_run_file = path.join("first_run.txt");
    
    std::fs::write(first_run_file, "Tutorial completed").map_err(|e| e.to_string())
}

// Command to set app as launcher
#[tauri::command]
fn set_as_launcher() -> Result<(), String> {
    // This would typically involve platform-specific code
    // For now, we'll just return success as a placeholder
    Ok(())
}

// Weather data structures
#[derive(Deserialize)]
struct OpenWeatherResponse {
    main: MainWeather,
    weather: Vec<Weather>,
}

#[derive(Deserialize)]
struct MainWeather {
    temp: f64,
}

#[derive(Deserialize)]
struct Weather {
    icon: String,
}

#[derive(Serialize)]
struct WeatherData {
    temperature: String,
    icon: String,
}

// Weather command
#[tauri::command]
async fn get_weather(lat: i8, lon: i8) -> Result<WeatherData, String> {
    dotenv().ok();
    let api_key = env::var("OPENWEATHER_API_KEY").map_err(|_| "API key not found".to_string())?;
    
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}&units=imperial",
        lat, lon, api_key
    );
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    let weather_data: OpenWeatherResponse = response
        .json()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(WeatherData {
        temperature: format!("{:.0}°F", weather_data.main.temp),
        icon: format!("https://openweathermap.org/img/wn/{}@2x.png", weather_data.weather[0].icon),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_system_info::init())
        .plugin(tauri_plugin_geolocation::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            is_first_run,
            complete_tutorial,
            set_as_launcher,
            get_weather,
            get_battery_info,
            network::check_network_status,
            speech::initialize_stt,
            speech::set_stt_mode,
            speech::get_stt_mode,
            speech::start_recording,
            speech::stop_recording,
            speech::transcribe_audio,
            engine::process_text_input,
            search::fetch_search_results,
            search::open_link
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
