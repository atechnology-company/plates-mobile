/**
 * Battery module for monitoring device battery status
 */
import { invoke } from "@tauri-apps/api/core";
import { batteries } from "tauri-plugin-system-info-api";

/**
 * Get current battery level percentage
 * Uses Tauri's native capabilities to access system information
 * @returns Promise with battery level percentage
 */
export async function getBatteryLevel(): Promise<number> {
  try {
    const batteryInfo = await batteries();
    return batteryInfo[0]?.state_of_charge || 100;
  } catch (error) {
    console.error("Error fetching battery level:", error);
    return 100; // Default to 100% if unable to get actual level
  }
}

/**
 * Setup a battery level update interval
 * @param callback Function to call with updated battery level
 * @param interval Interval in milliseconds (default: 60000ms - 1 minute)
 * @returns Function to clear the interval
 */
export function setupBatteryInterval(
  callback: (batteryLevel: number) => void,
  interval: number = 60000
): () => void {
  // Get initial battery level
  getBatteryLevel().then(callback);
  
  // Set up interval for updates
  const batteryInterval = setInterval(async () => {
    const level = await getBatteryLevel();
    callback(level);
  }, interval);

  return () => clearInterval(batteryInterval);
}