/**
 * Time module for handling time and date functionality
 */

/**
 * Get the current formatted time
 * @returns Current time as a string in locale format
 */
export function getCurrentTime(): string {
  return new Date().toLocaleTimeString();
}

/**
 * Get the current formatted date
 * @returns Current date as a string in locale format
 */
export function getCurrentDate(): string {
  return new Date().toLocaleDateString();
}

/**
 * Setup a time update interval
 * @param callback Function to call with updated time and date
 * @param interval Interval in milliseconds (default: 1000ms)
 * @returns Function to clear the interval
 */
export function setupTimeInterval(
  callback: (time: string, date: string) => void,
  interval: number = 1000
): () => void {
  const timeInterval = setInterval(() => {
    callback(getCurrentTime(), getCurrentDate());
  }, interval);

  return () => clearInterval(timeInterval);
}