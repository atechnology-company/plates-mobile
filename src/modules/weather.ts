import { invoke } from "@tauri-apps/api/core";
import { getCurrentPosition } from '@tauri-apps/plugin-geolocation';

// Weather data interface
export interface WeatherData {
  temperature: string;
  icon: string;
}

// Default weather data
const defaultWeather: WeatherData = {
  temperature: "--°F",
  icon: "🌤️"
};

/**
 * Get current weather based on device location
 * Uses Tauri's native capabilities to access location and weather APIs
 * @returns Promise with weather data
 */
export async function getCurrentWeather(): Promise<WeatherData> {
  try {
    // Get current position
    const position = await getCurrentPosition();
    
    // Call Tauri backend to get weather data with coordinates
    const weatherData = await invoke<WeatherData>("get_weather", {
      lat: Math.round(position.coords.latitude),
      lon: Math.round(position.coords.longitude)
    });
    return weatherData;
  } catch (error) {
    console.error("Error fetching weather data:", error);
    return defaultWeather;
  }
}

/**
 * Setup a weather update interval
 * @param callback Function to call with updated weather data
 * @param interval Interval in milliseconds (default: 600000ms - 10 minutes)
 * @returns Function to clear the interval
 */
export function setupWeatherInterval(
  callback: (weatherData: WeatherData) => void,
  interval: number = 600000
): () => void {
  // Get initial weather
  getCurrentWeather().then(callback);
  
  // Set up interval for updates
  const weatherInterval = setInterval(async () => {
    const weatherData = await getCurrentWeather();
    callback(weatherData);
  }, interval);

  return () => clearInterval(weatherInterval);
}