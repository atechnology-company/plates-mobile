/**
 * Battery module for monitoring device battery status
 */
import { invoke } from "@tauri-apps/api/core";
import { Batteries, batteries } from "tauri-plugin-system-info-api";

/**
 * Battery information interface
 */
export interface BatteryInfo {
  level: number;  // Battery charge percentage
  state: string;  // Battery state (charging, discharging, etc)
}

/**
 * Get complete battery information including level and state
 * Uses Tauri's native capabilities to access system information
 * @returns Promise with battery information
 */
export async function getBatteryInfo(): Promise<BatteryInfo> {
  try {
    const batteryInfo = await batteries();
    const firstBattery = batteryInfo[0];
    return {
      level: firstBattery?.state_of_charge || 100,
      state: firstBattery?.state || 'unknown'
    };
  } catch (error) {
    console.error("Error fetching battery information:", error);
    return { level: 100, state: 'unknown' }; // Default values if unable to get actual info
  }
}

/**
 * Get current battery level percentage
 * Uses Tauri's native capabilities to access system information
 * @returns Promise with battery level percentage
 */
export async function getBatteryLevel(): Promise<number> {
  const batteryInfo = await getBatteryInfo();
  return batteryInfo.level;
}

/**
 * Get current battery state (charging, discharging, etc)
 * @returns Promise with battery state string
 */
export async function getBatteryState(): Promise<string> {
  const batteryInfo = await getBatteryInfo();
  return batteryInfo.state;
}

/**
 * Setup a battery information update interval
 * @param callback Function to call with updated battery information
 * @param interval Interval in milliseconds (default: 60000ms - 1 minute)
 * @returns Function to clear the interval
 */
export function setupBatteryInfoInterval(
  callback: (batteryInfo: BatteryInfo) => void,
  interval: number = 60000
): () => void {
  // Get initial battery info
  getBatteryInfo().then(callback);
  
  // Set up interval for updates
  const batteryInterval = setInterval(async () => {
    const info = await getBatteryInfo();
    callback(info);
  }, interval);

  return () => clearInterval(batteryInterval);
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