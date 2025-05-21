<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Tutorial from "../components/tutorial.svelte";
  import { onMount } from "svelte";
  
  // Import modules
  import { getCurrentTime, getCurrentDate, setupTimeInterval } from "../modules/time";
  import { getCurrentWeather, type WeatherData } from "../modules/weather";
  import { getBatteryInfo, setupBatteryInfoInterval, type BatteryInfo } from "../modules/battery";
  import SpeechToText from "../components/SpeechToText.svelte";
  
  
  // References to components
  let speechToTextComponent: SpeechToText;
  

  // State variables
  let isFirstRun = false;
  let time = getCurrentTime();
  let date = getCurrentDate();
  let temp = "--°F";
  let tempIcon = "🌤️";
  let batteryInfo: BatteryInfo = { level: 100, state: 'unknown' };
  let longPressTimer: number | null = null;
  let isLongPressing = false;
  let lastTranscription = "";
  let lastTextInput = "";
  let lastResponse = "";

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
      const clearBatteryInterval = setupBatteryInfoInterval((info: BatteryInfo) => {
        batteryInfo = info;
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
  
  // Long press detection functions
  function handleTouchStart(event: TouchEvent | MouseEvent) {
    if (isFirstRun) return;
    
    // Start a timer to detect long press
    longPressTimer = setTimeout(() => {
      isLongPressing = true;
      if (speechToTextComponent) {
        speechToTextComponent.handleLongPressStart();
      }
    }, 500) as unknown as number; // 500ms threshold for long press
  }
  
  function handleTouchEnd(event: TouchEvent | MouseEvent) {
    if (isFirstRun) return;
    
    // Clear the timer if touch ends before long press threshold
    if (longPressTimer) {
      clearTimeout(longPressTimer);
      longPressTimer = null;
    }
    
    // If we were long pressing, handle the end of long press
    if (isLongPressing) {
      isLongPressing = false;
      if (speechToTextComponent) {
        speechToTextComponent.handleLongPressEnd();
      }
    }
  }
  
  // Handle recording events from SpeechToText component
  function handleRecordingStopped(event: CustomEvent) {
    if (event.detail && event.detail.text) {
      lastTranscription = event.detail.text;
    }
  }
  
  // Handle tap event to show text input
  function handleTap(event: MouseEvent | TouchEvent) {
    // Ignore if long press is active or this is the first run
    if (isLongPressing || isFirstRun) return;
    
    // Only handle simple taps, not long presses
    if (longPressTimer) {
      clearTimeout(longPressTimer);
      longPressTimer = null;
    }
    
    // Show the text input component
    if (speechToTextComponent) {
      speechToTextComponent.showTextInputField();
    }
  }
  
  // Handle text submission from TextInput component
  function handleTextSubmitted(event: CustomEvent) {
    if (event.detail) {
      if (event.detail.error) {
        console.error(event.detail.error);
        return;
      }
      
      lastTextInput = event.detail.text;
      lastResponse = event.detail.response;
    }
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

<main 
  class="container {isLongPressing ? 'listening' : ''}"
  on:touchstart={handleTouchStart}
  on:touchend={handleTouchEnd}
  on:touchcancel={handleTouchEnd}
  on:mousedown={handleTouchStart}
  on:mouseup={handleTouchEnd}
  on:mouseleave={handleTouchEnd}
  on:click={handleTap}
>
  {#if isFirstRun}
    <Tutorial on:tutorialComplete={handleTutorialComplete} />
  {:else}
    <div class="homescreen-content">
      <h1>{time}</h1>
      <h2>{date} {temp} {tempIcon}</h2>
      <h3 class="bottomright">{batteryInfo.level}% ({batteryInfo.state})</h3>
      
      <!-- Speech to Text Component -->
      <SpeechToText 
        bind:this={speechToTextComponent}
        activateOnLongPress={true}
        on:recordingStopped={handleRecordingStopped}
      />
      

      
      {#if !isLongPressing && lastTranscription}
        <div class="last-command">
          <p>Last command: {lastTranscription}</p>
        </div>
      {/if}
      
      {#if lastTextInput && lastResponse}
        <div class="last-text-input">
          <p>You: {lastTextInput}</p>
          <p>Response: {lastResponse}</p>
        </div>
      {/if}
    </div>
  {/if}
</main>

<style>
  .container {
    display: flex;
    flex-direction: column;
    justify-content: left;
    align-items: center;
    height: 100vh;
    width: 100%;
    position: relative;
    transition: background-color 0.3s ease;
  }
  
  .container.listening {
    background-color: rgba(74, 134, 232, 0.05);
  }
  
  .homescreen-content {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-top: 20%;
  }

  .bottomright {
    position: absolute;
    bottom: 10px;
    right: 10px;
  }
  
  .last-command, .last-text-input {
    position: absolute;
    left: 0;
    right: 0;
    text-align: center;
    padding: 10px;
    font-size: 0.9rem;
    color: rgba(0, 0, 0, 0.7);
    background-color: rgba(255, 255, 255, 0.7);
    border-radius: 4px;
    margin: 0 20px;
    max-width: 80%;
    margin-left: auto;
    margin-right: auto;
  }
  
  .last-command {
    bottom: 40px;
  }
  
  .last-text-input {
    bottom: 100px;
  }
</style>
