<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Tutorial from "../components/tutorial.svelte";
  import { onMount } from "svelte";
  
  // Import modules
  import { getCurrentTime, getCurrentDate, setupTimeInterval } from "../modules/time";
  import { getCurrentWeather, type WeatherData } from "../modules/weather";
  import { getBatteryLevel, setupBatteryInterval } from "../modules/battery";

  // State variables
  let isFirstRun = false;
  let time = getCurrentTime();
  let date = getCurrentDate();
  let temp = "--°F";
  let tempIcon = "🌤️";
  let batteryLevel = 100;

  // Check if this is the first run and initialize all modules
  onMount(() => {
    try {
      // Check if this is first run
      invoke("is_first_run").then(result => {
        isFirstRun = result as boolean;
      });
      
      // Setup time updates
      const clearTimeInterval = setupTimeInterval((newTime, newDate) => {
        time = newTime;
        date = newDate;
      });
      
      // Setup weather updates
      const clearWeatherInterval = setupWeatherInterval((weatherData: WeatherData) => {
        temp = weatherData.temperature;
        tempIcon = weatherData.icon;
      });
      
      // Setup battery updates
      const clearBatteryInterval = setupBatteryInterval((level: number) => {
        batteryLevel = level;
      });

      // Clean up intervals on component unmount
      return () => {
        clearTimeInterval();
        clearWeatherInterval();
        clearBatteryInterval();
      };
    } catch (error) {
      console.error("Error initializing app:", error);
    }
  });

  // Handle tutorial completion
  function handleTutorialComplete() {
    isFirstRun = false;
  }
  
  // Setup weather update interval
  function setupWeatherInterval(callback: (data: WeatherData) => void, interval = 600000) {
    // Get initial weather
    getCurrentWeather().then(callback);
    
    // Set up interval for updates
    const weatherInterval = setInterval(async () => {
      const weatherData = await getCurrentWeather();
      callback(weatherData);
    }, interval);

    return () => clearInterval(weatherInterval);
  }
</script>

<main class="container">
  {#if isFirstRun}
    <Tutorial on:tutorialComplete={handleTutorialComplete} />
  {:else}
    <h1>{time}</h1>
    <h2>{date} {temp} {tempIcon}</h2>
    <h3 class="bottomright">{batteryLevel}%</h3>
  {/if}

</main>

<style>
  .container {
    display: flex;
    flex-direction: column;
    justify-content: left;
    align-items: center;
    height: 100vh;
  }

  .bottomright {
    position: absolute;
    bottom: 10px;
    right: 10px;
  }
</style>
