


use tauri::Manager;
use serde::{Serialize, Deserialize};
use tauri_plugin_system_info::{commands::battery, model::{Battery, BatteryState}};
use reqwest;
use dotenv::dotenv;
use std::env;

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

// Battery level command
#[tauri::command]
fn get_battery_level(state: tauri::State<'_, tauri_plugin_system_info::SysInfoState>) -> Result<u8, String> {
    let battery_info = battery::batteries(state).map_err(|e| e.to_string())?;
    let first_battery = battery_info.get(0).ok_or("No battery found".to_string())?;
    // Get the state of charge from the battery
    let state_of_charge = first_battery.state_of_charge;
    Ok(state_of_charge)
}

#[tauri::command]
fn get_battery_state(state: tauri::State<'_, tauri_plugin_system_info::SysInfoState>) -> Result<BatteryState, String> {
    let battery_info = battery::batteries(state).map_err(|e| e.to_string())?;
    let first_battery = battery_info.get(0).ok_or("No battery found".to_string())?;
    // Get the actual battery state
    let battery_state = first_battery.state.clone();
    Ok(battery_state)
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
        // Add location and microphone permissions plugins
        .setup(|_app| {
            #[cfg(mobile)]
            {
                // Request permissions on mobile
                // This is a placeholder - actual implementation would use platform-specific APIs
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            is_first_run,
            complete_tutorial,
            set_as_launcher,
            get_battery_level,
            get_battery_state,
            get_weather
        ])
        .plugin(tauri_plugin_geolocation::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
