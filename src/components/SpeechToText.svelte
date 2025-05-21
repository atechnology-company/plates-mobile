<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, createEventDispatcher } from "svelte";
  import PredictiveTextInput from "./PredictiveTextInput.svelte";

  // Create event dispatcher to communicate with parent component
  const dispatch = createEventDispatcher();

  // State variables
  let isModelLoaded = false;
  let isRecording = false;
  let transcription = "";
  let errorMessage = "";
  let isProcessing = false;
  let showTranscription = false;
  let showTextInput = false;
  
  // Export props to allow parent component to control this component
  export let activateOnLongPress = false; // Whether to activate on long press
  
  // Reference to the predictive text input component
  let predictiveTextInput: PredictiveTextInput;

  // Initialize the STT model on component mount
  onMount(async () => {
    try {
      await invoke("initialize_stt");
      isModelLoaded = true;
    } catch (error) {
      errorMessage = `Failed to initialize speech recognition: ${error}`;
      console.error(errorMessage);
    }
  });

  // Function to start recording audio
  async function startRecording() {
    if (!isModelLoaded) {
      errorMessage = "Speech recognition not initialized yet. Please wait.";
      return;
    }

    try {
      // Start recording using the Tauri command
      await invoke("start_recording");
      
      isRecording = true;
      errorMessage = "";
      showTranscription = false;
      transcription = "";
      
      console.log("Recording started...");
      
      // Notify parent component that recording has started
      dispatch('recordingStarted');
    } catch (error) {
      errorMessage = `Failed to start recording: ${error}`;
      console.error(errorMessage);
    }
  }

  // Define the TranscriptionResult interface to match the Rust struct
  interface TranscriptionResult {
    text: string;
    language: string;
  }

  // Function to stop recording and transcribe
  async function stopRecording() {
    if (!isRecording) return;
    
    isRecording = false;
    isProcessing = true;
    console.log("Recording stopped. Processing audio...");
    
    try {
      // Stop recording and transcribe the audio in one step
      const result: TranscriptionResult = await invoke<TranscriptionResult>("stop_recording");
      transcription = result.text;
      showTranscription = true;
      isProcessing = false;
      
      // Notify parent component that recording has stopped with the transcription
      dispatch('recordingStopped', { text: transcription });
    } catch (error) {
      errorMessage = `Transcription failed: ${error}`;
      console.error(errorMessage);
      isProcessing = false;
      dispatch('recordingStopped', { error: errorMessage });
    }
  }
  
  // Function to handle long press activation from parent component
  export function handleLongPressStart() {
    if (activateOnLongPress && !isRecording && !isProcessing) {
      startRecording();
    }
  }
  
  // Function to handle long press end from parent component
  export function handleLongPressEnd() {
    if (activateOnLongPress && isRecording) {
      stopRecording();
    }
  }
  
  // Function to show the text input
  export function showTextInputField() {
    if (predictiveTextInput) {
      predictiveTextInput.show();
      showTextInput = true;
    }
  }
  
  // Handle text submission from the predictive text input
  function handleTextSubmitted(event: { detail: { text: any; response: any; }; }) {
    const { text, response } = event.detail;
    console.log("Text submitted:", text);
    console.log("Response:", response);
    
    // Notify parent component
    dispatch('textSubmitted', { text, response });
    
    showTextInput = false;
    showTranscription = false;
  }
  
  // Handle text input cancellation
  function handleInputCancelled() {
    showTextInput = false;
  }
</script>

<div class="speech-to-text-container {isRecording ? 'recording' : ''} {isProcessing ? 'processing' : ''}" style="display: {activateOnLongPress && !isRecording && !isProcessing && !showTranscription && !showTextInput ? 'none' : 'block'}">
  {#if !activateOnLongPress}
    <h2>Speech to Text</h2>
    
    {#if errorMessage}
      <div class="error">{errorMessage}</div>
    {/if}
    
    <div class="controls">
      {#if !isRecording && !isProcessing}
        <div class="button-group">
          <button on:click={startRecording} disabled={!isModelLoaded || isRecording || isProcessing}>
            Start Recording
          </button>
          <button on:click={showTextInputField} class="text-button">
            Text Input
          </button>
        </div>
      {:else if isRecording}
        <button on:click={stopRecording} disabled={isProcessing}>
          Stop Recording
        </button>
      {:else if isProcessing}
        <button disabled={true}>
          Processing...
        </button>
      {/if}
    </div>
  {:else}
    {#if isRecording}
      <div class="recording-indicator">
        <div class="pulse"></div>
        <p>Listening...</p>
      </div>
    {:else if isProcessing}
      <div class="processing-indicator">
        <div class="spinner"></div>
        <p>Processing...</p>
      </div>
    {/if}
  {/if}
  
  {#if showTranscription && transcription}
    <div class="transcription-result">
      <p>{transcription}</p>
    </div>
  {/if}
  
  {#if !activateOnLongPress}
    <div class="status">
      <p>Status: {!isModelLoaded ? "Initializing..." : isRecording ? "Recording" : isProcessing ? "Processing" : "Ready"}</p>
    </div>
  {/if}
</div>

<PredictiveTextInput 
  bind:this={predictiveTextInput}
  on:textSubmitted={handleTextSubmitted}
  on:inputCancelled={handleInputCancelled}
/>

<style>
  .speech-to-text-container {
    padding: 1rem;
    background-color: rgba(245, 245, 245, 0.8);
    border-radius: 8px;
    margin: 1rem 0;
    transition: all 0.3s ease;
  }
  
  .speech-to-text-container.recording {
    background-color: rgba(74, 134, 232, 0.1);
  }
  
  .speech-to-text-container.processing {
    background-color: rgba(255, 193, 7, 0.1);
  }
  
  .controls {
    margin: 1rem 0;
  }
  
  .button-group {
    display: flex;
    gap: 10px;
  }
  
  button {
    padding: 0.5rem 1rem;
    background-color: #4a86e8;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
    min-width: 150px;
  }
  
  .text-button {
    background-color: #34a853;
  }
  
  .text-button:hover:not(:disabled) {
    background-color: #2d9249;
  }
  
  button:disabled {
    background-color: #cccccc;
    cursor: not-allowed;
  }
  
  button:hover:not(:disabled) {
    background-color: #3a76d8;
  }
  
  .error {
    color: #d32f2f;
    margin: 0.5rem 0;
    padding: 0.5rem;
    background-color: #ffebee;
    border-radius: 4px;
  }
  
  .transcription-result {
    margin-top: 1rem;
    padding: 1rem;
    background-color: rgba(255, 255, 255, 0.9);
    border-radius: 4px;
    border-left: 4px solid #4a86e8;
    max-width: 80%;
    margin-left: auto;
    margin-right: auto;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }
  
  .status {
    margin-top: 1rem;
    font-size: 0.9rem;
    color: #666;
  }
  
  .recording-indicator, .processing-indicator {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 1rem;
  }
  
  .pulse {
    width: 50px;
    height: 50px;
    background-color: rgba(74, 134, 232, 0.6);
    border-radius: 50%;
    margin-bottom: 10px;
    animation: pulse 1.5s infinite ease-in-out;
  }
  
  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid rgba(255, 193, 7, 0.3);
    border-radius: 50%;
    border-top-color: #ffc107;
    margin-bottom: 10px;
    animation: spin 1s infinite linear;
  }
  
  @keyframes pulse {
    0% {
      transform: scale(0.95);
      box-shadow: 0 0 0 0 rgba(74, 134, 232, 0.7);
    }
    70% {
      transform: scale(1);
      box-shadow: 0 0 0 10px rgba(74, 134, 232, 0);
    }
    100% {
      transform: scale(0.95);
      box-shadow: 0 0 0 0 rgba(74, 134, 232, 0);
    }
  }
  
  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }
</style>