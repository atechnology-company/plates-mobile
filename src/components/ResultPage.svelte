<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, createEventDispatcher } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { cn } from "$lib/utils";

  // Create event dispatcher to communicate with parent component
  const dispatch = createEventDispatcher();

  // Props
  export let query = "";
  export let isVisible = false;

  // State variables
  let results: SearchResult[] = [];
  let isLoading = false;
  let errorMessage = "";

  // Define the search result interface to match the Rust struct
  interface SearchResult {
    title: string;
    link: string;
    snippet: string;
    image_url?: string;
  }

  // Show the results page
  export function show(searchQuery: string) {
    query = searchQuery;
    isVisible = true;
    fetchResults();
  }

  // Hide the results page
  export function hide() {
    isVisible = false;
    results = [];
  }

  // Fetch search results from Google Search API
  async function fetchResults() {
    if (!query.trim()) return;
    
    isLoading = true;
    errorMessage = "";
    
    try {
      // Call the Tauri command to fetch search results from our Rust backend
      const searchResults = await invoke<SearchResult[]>("fetch_search_results", { query: query.trim() });
      
      results = searchResults;
      console.log("Search results:", results);
      
      if (results.length === 0) {
        errorMessage = "No results found. Please try a different search.";
      }
    } catch (error) {
      console.error("Error fetching search results:", error);
      errorMessage = `Failed to fetch results: ${error}`;
      results = [];
    } finally {
      isLoading = false;
    }
  }

  // Handle close button click
  function handleClose() {
    hide();
    dispatch('resultsClosed');
  }

  // Open a link in the default browser
  async function openLink(url: string) {
    try {
      await invoke("open_link", { url });
    } catch (error) {
      console.error("Error opening link:", error);
      errorMessage = `Failed to open link: ${error}`;
    }
  }
</script>

<div class="results-overlay" class:visible={isVisible}>
  <div 
    class="results-container"
    transition:fly={{ y: 20, duration: 200, easing: cubicOut }}
  >
    <div class="results-header">
      <h2>Results for "{query}"</h2>
      <Button variant="ghost" size="icon" on:click={handleClose} class="close-button">
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M18 6 6 18"/>
          <path d="m6 6 12 12"/>
        </svg>
        <span class="sr-only">Close</span>
      </Button>
    </div>
    
    <div class="results-content">
      {#if isLoading}
        <div class="loading-state">
          <svg class="spinner" viewBox="0 0 50 50">
            <circle class="path" cx="25" cy="25" r="20" fill="none" stroke-width="5"></circle>
          </svg>
          <p>Searching...</p>
        </div>
      {:else if errorMessage}
        <div class="error-state">
          <p>{errorMessage}</p>
        </div>
      {:else if results.length === 0}
        <div class="empty-state">
          <p>No results found. Try a different search term.</p>
        </div>
      {:else}
        <div class="results-list">
          {#each results as result}
            <div class="result-item">
              <div class="result-content">
                <h3>
                  <button on:click={() => openLink(result.link)} class="result-link">
                    {result.title}
                  </button>
                </h3>
                <p class="result-url">{result.link}</p>
                <p class="result-snippet">{result.snippet}</p>
              </div>
              {#if result.image_url}
                <div class="result-image">
                  <img src={result.image_url} alt={result.title} loading="lazy" />
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .results-overlay {
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

  .results-overlay.visible {
    opacity: 1;
    pointer-events: auto;
  }

  .results-container {
    width: 90%;
    max-width: 800px;
    max-height: 80vh;
    background-color: white;
    border-radius: var(--radius);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid hsl(var(--border));
  }

  .results-header h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .close-button {
    color: hsl(var(--foreground));
  }

  .results-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .loading-state,
  .error-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 32px 16px;
    text-align: center;
    color: hsl(var(--muted-foreground));
  }

  .spinner {
    width: 40px;
    height: 40px;
    animation: rotate 2s linear infinite;
    margin-bottom: 16px;
  }

  .spinner .path {
    stroke: hsl(var(--primary));
    stroke-linecap: round;
    animation: dash 1.5s ease-in-out infinite;
  }

  @keyframes rotate {
    100% {
      transform: rotate(360deg);
    }
  }

  @keyframes dash {
    0% {
      stroke-dasharray: 1, 150;
      stroke-dashoffset: 0;
    }
    50% {
      stroke-dasharray: 90, 150;
      stroke-dashoffset: -35;
    }
    100% {
      stroke-dasharray: 90, 150;
      stroke-dashoffset: -124;
    }
  }

  .results-list {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .result-item {
    display: flex;
    gap: 16px;
    padding-bottom: 16px;
    border-bottom: 1px solid hsl(var(--border));
  }

  .result-item:last-child {
    border-bottom: none;
  }

  .result-image {
    flex-shrink: 0;
    width: 120px;
    height: 90px;
    overflow: hidden;
    border-radius: var(--radius);
  }

  .result-image img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .result-content {
    flex: 1;
    min-width: 0;
  }

  .result-content h3 {
    margin: 0 0 4px 0;
    font-size: 1rem;
    font-weight: 600;
    line-height: 1.4;
  }

  .result-link {
    color: hsl(var(--primary));
    text-decoration: none;
    background: none;
    border: none;
    padding: 0;
    font-size: inherit;
    font-weight: inherit;
    cursor: pointer;
    text-align: left;
  }

  .result-link:hover {
    text-decoration: underline;
  }

  .result-url {
    margin: 0 0 8px 0;
    font-size: 0.875rem;
    color: hsl(var(--muted-foreground));
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .result-snippet {
    margin: 0;
    font-size: 0.875rem;
    line-height: 1.5;
    color: hsl(var(--foreground));
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>