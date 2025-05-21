<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, createEventDispatcher } from "svelte";

  // Create event dispatcher to communicate with parent component
  const dispatch = createEventDispatcher();

  // State variables
  let inputText = "";
  let isVisible = false;
  let inputElement: HTMLInputElement;

  // Show the text input and focus it
  export function show() {
    isVisible = true;
    // Use setTimeout to ensure the DOM has updated before focusing
    setTimeout(() => {
      if (inputElement) {
        inputElement.focus();
      }
    }, 50);
  }

  // Hide the text input
  export function hide() {
    isVisible = false;
    inputText = "";
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
    }
  }
</script>

<div class="text-input-overlay" class:visible={isVisible} on:keydown={handleKeydown}>
  <div class="text-input-container">
    <form on:submit|preventDefault={handleSubmit}>
      <input 
        type="text" 
        bind:value={inputText} 
        bind:this={inputElement}
        placeholder="Type your message here..."
        autocomplete="off"
      />
      <div class="button-container">
        <button type="button" class="cancel-button" on:click={handleCancel}>Cancel</button>
        <button type="submit" class="submit-button" disabled={!inputText.trim()}>Send</button>
      </div>
    </form>
  </div>
</div>

<style>
  .text-input-overlay {
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

  .text-input-overlay.visible {
    opacity: 1;
    pointer-events: auto;
  }

  .text-input-container {
    width: 90%;
    max-width: 500px;
    background-color: white;
    border-radius: 8px;
    padding: 16px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  input {
    padding: 12px 16px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 16px;
    width: 100%;
    box-sizing: border-box;
  }

  input:focus {
    outline: none;
    border-color: #4a86e8;
    box-shadow: 0 0 0 2px rgba(74, 134, 232, 0.2);
  }

  .button-container {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }

  button {
    padding: 10px 16px;
    border: none;
    border-radius: 4px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .cancel-button {
    background-color: #f1f3f4;
    color: #5f6368;
  }

  .cancel-button:hover {
    background-color: #e8eaed;
  }

  .submit-button {
    background-color: #4a86e8;
    color: white;
  }

  .submit-button:hover {
    background-color: #3a76d8;
  }

  .submit-button:disabled {
    background-color: #c1d1f0;
    cursor: not-allowed;
  }
</style>