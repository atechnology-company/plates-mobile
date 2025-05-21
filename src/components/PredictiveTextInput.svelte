<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, createEventDispatcher } from "svelte";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Button } from "$lib/components/ui/button";
  import { cn } from "$lib/utils";
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  
  // Import node-suggest for real-time search suggestions
  const Suggest = require("node-suggest");
  
  // Create event dispatcher to communicate with parent component
  const dispatch = createEventDispatcher();

  // State variables
  let inputText = "";
  let isVisible = false;
  let predictions: string[] = [];
  let isLoading = false;
  let selectedIndex = -1;

  // Show the text input and focus it
  export function show() {
    isVisible = true;
    // Use setTimeout to ensure the DOM has updated before focusing
    setTimeout(() => {
      const textareaElement = document.querySelector('textarea');
      if (textareaElement) {
        textareaElement.focus();
      }
    }, 50);
  }

  // Hide the text input
  export function hide() {
    isVisible = false;
    inputText = "";
    predictions = [];
    selectedIndex = -1;
  }

  // Handle form submission
  async function handleSubmit() {
    if (!inputText.trim()) return;
    
    try {
      // Call the AI model API in engine.rs
      const response = await invoke("process_text_input", { text: inputText.trim() });
      
      // Notify parent component about the submission
      dispatch('textSubmitted', { text: inputText.trim(), response });
      
      // Clear the input and hide
      inputText = "";
      hide();
    } catch (error) {
      console.error("Error processing text input:", error);
      dispatch('textSubmitted', { error: `Failed to process: ${error}` });
    }
  }

  // Handle cancel button click
  function handleCancel() {
    hide();
    dispatch('inputCancelled');
  }

  // Handle keyboard events
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleCancel();
      return;
    }
    
    if (predictions.length > 0) {
      if (event.key === 'ArrowDown') {
        event.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, predictions.length - 1);
      } else if (event.key === 'ArrowUp') {
        event.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, -1);
      } else if (event.key === 'Enter' && selectedIndex >= 0) {
        event.preventDefault();
        inputText = predictions[selectedIndex];
        predictions = [];
        selectedIndex = -1;
      }
    }
  }

  // Get predictive search results as user types using node-suggest
  async function getPredictions() {
    if (!inputText.trim() || inputText.length < 2) {
      predictions = [];
      return;
    }
    
    isLoading = true;
    
    try {
      // Use node-suggest to get real Google search suggestions
      const query = inputText.trim();
      const suggestResults = await Suggest.google(query);
      
      // Filter out the original query if it's in the results
      predictions = suggestResults.filter((suggestion: string) => 
        suggestion.toLowerCase() !== query.toLowerCase()
      );
      
      selectedIndex = -1;
    } catch (error) {
      console.error("Error getting predictions:", error);
      predictions = [];
    } finally {
      isLoading = false;
    }
  }

  // Store the timeout handler
  let predictionTimeoutHandler: ReturnType<typeof setTimeout> | null = null;
  
  // Update predictions when input text changes
  $: {
    if (isVisible) {
      // Clear any existing timeout
      if (predictionTimeoutHandler) {
        clearTimeout(predictionTimeoutHandler);
      }
      
      // Set new timeout
      predictionTimeoutHandler = setTimeout(() => {
        getPredictions();
      }, 300);
    }
  }
  
  // Clean up the timeout when the component is destroyed
  onMount(() => {
    return () => {
      if (predictionTimeoutHandler) {
        clearTimeout(predictionTimeoutHandler);
      }
    };
  });

  // Select a prediction
  function selectPrediction(prediction: string) {
    inputText = prediction;
    predictions = [];
    selectedIndex = -1;
    
    // Focus back on textarea
    setTimeout(() => {
      const textareaElement = document.querySelector('textarea');
      if (textareaElement) {
        textareaElement.focus();
      }
    }, 50);
  }
</script>

<div class="predictive-text-overlay" class:visible={isVisible} on:keydown={handleKeydown}>
  <div class="predictive-text-container">
    <form on:submit|preventDefault={handleSubmit}>
      <div class="textarea-container">
        <Textarea 
          bind:value={inputText} 
          placeholder="Type your message here..."
          class="min-h-[100px]"
        />
        
        {#if predictions.length > 0}
          <div class="predictions-container" transition:fly={{ y: 10, duration: 150, easing: cubicOut }}>
            {#each predictions as prediction, i}
              <button 
                type="button"
                class={cn(
                  "prediction-item", 
                  selectedIndex === i && "selected"
                )}
                on:click={() => selectPrediction(prediction)}
                on:mouseenter={() => selectedIndex = i}
              >
                {prediction}
              </button>
            {/each}
          </div>
        {/if}
      </div>
      
      <div class="button-container">
        <Button variant="outline" type="button" on:click={handleCancel}>Cancel</Button>
        <Button type="submit" disabled={!inputText.trim()}>Send</Button>
      </div>
    </form>
  </div>
</div>

<style>
  .predictive-text-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.2s ease;
  }

  .predictive-text-overlay.visible {
    opacity: 1;
    pointer-events: auto;
  }

  .predictive-text-container {
    width: 90%;
    max-width: 500px;
    background-color: white;
    border-radius: var(--radius);
    padding: 16px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .textarea-container {
    position: relative;
  }

  .predictions-container {
    position: absolute;
    top: 100%;
    left: 0;
    width: 100%;
    background-color: white;
    border: 1px solid hsl(var(--border));
    border-radius: 0 0 var(--radius) var(--radius);
    max-height: 200px;
    overflow-y: auto;
    z-index: 10;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .prediction-item {
    padding: 8px 12px;
    width: 100%;
    text-align: left;
    border: none;
    background: none;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .prediction-item:hover,
  .prediction-item.selected {
    background-color: hsl(var(--muted));
  }

  .button-container {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }
</style>