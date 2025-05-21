<script lang="ts">
    import {checkPermissions, requestPermissions} from "@tauri-apps/plugin-geolocation";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, createEventDispatcher } from "svelte";

    // Create event dispatcher to communicate with parent component
    const dispatch = createEventDispatcher();

    // Tutorial steps
    const tutorialSteps = [
        {
            title: "Heyo! Looks like you're new around here.",
            subtitle: "Ready to get started?",
            action: null
        },
        {
            title: "You're going to want to set this as your launcher.",
            subtitle: "Click anywhere, we'll open the menu for you.",
            action: "setLauncher"
        },
        {
            title: "Login with Google, we'll save your data.",
            subtitle: "Just in case you get a link a new device... or you get a new phone... or you regretfully deleted the app (why would you ever do that?)",
            action: null
        },
        {
            title: "Also we'll need access to other stuff on your phone, just accept the permissions.",
            subtitle: "Sounds scary, I know, but trust us, we'll make everything easier for you, and our code is open source",
            extraText: "What that means is that anybody can see our code and we can't really do any bad things",
            action: "requestPermissions"
        }
    ];

    // Current step index
    let currentStep = 0;

    // Function to navigate to the next step
    function nextStep() {
        if (currentStep < tutorialSteps.length - 1) {
            currentStep++;
        } else {
            completeTutorial();
        }
    }

    // Function to navigate to the previous step
    function prevStep() {
        if (currentStep > 0) {
            currentStep--;
        }
    }

    // Function to handle step-specific actions
    async function handleAction() {
        const action = tutorialSteps[currentStep].action;
        if (action === "setLauncher") {
            await invoke("set_as_launcher");
        } else if (action === "requestPermissions") {
            // Request location and microphone permissions
            // This would typically use platform-specific APIs
            console.log("Requesting permissions");
        }
    }

    // Function to complete the tutorial
    async function completeTutorial() {
        try {
            await invoke("complete_tutorial");
            // Dispatch an event to notify the parent component
            dispatch('tutorialComplete');
        } catch (error) {
            console.error("Failed to complete tutorial:", error);
        }
    }

    // Handle clicks on the tutorial area
    function handleClick() {
        if (tutorialSteps[currentStep].action) {
            handleAction();
        }
        nextStep();
    }

    // Handle swipe gestures
    let touchStartY = 0;
    
    function handleTouchStart(event: TouchEvent) {
        touchStartY = event.touches[0].clientY;
    }
    
    function handleTouchEnd(event: TouchEvent) {
        const touchEndY = event.changedTouches[0].clientY;
        const diff = touchStartY - touchEndY;
        
        // Swipe up
        if (diff > 50) {
            nextStep();
        }
        // Swipe down
        else if (diff < -50) {
            prevStep();
        }
    }

    // Initialize when component mounts
    onMount(() => {
        // Add event listeners for swipe gestures
        document.addEventListener('touchstart', handleTouchStart, false);
        document.addEventListener('touchend', handleTouchEnd, false);
        
        (async () => {
            try {
                let permissions = await checkPermissions();
                if (
                    permissions.location === 'prompt' ||
                    permissions.location === 'prompt-with-rationale'
                ) {
                    permissions = await requestPermissions(['location']);
                }
            } catch (error) {
                console.error('Failed to check/request permissions:', error);
            }
        })();

        // Return cleanup function
        return () => {
            document.removeEventListener('touchstart', handleTouchStart);
            document.removeEventListener('touchend', handleTouchEnd);
        };
    });
</script>

<div class="tutorial-container" 
     on:click={handleClick}
     on:touchstart={handleTouchStart}
     on:touchend={handleTouchEnd}>
    
    <div class="tutorial-content">
        <h1>{tutorialSteps[currentStep].title}</h1>
        <h2>{tutorialSteps[currentStep].subtitle}</h2>
        {#if tutorialSteps[currentStep].extraText}
            <h4>{tutorialSteps[currentStep].extraText}</h4>
        {/if}
    </div>
    
    <div class="tutorial-navigation">
        <div class="progress-dots">
            {#each tutorialSteps as _, i}
                <div class="dot {i === currentStep ? 'active' : ''}"></div>
            {/each}
        </div>
        <h6 class="bottomright">
            {currentStep === tutorialSteps.length - 1 ? 'swipe up to finish' : 'swipe up to continue'}
        </h6>
    </div>
</div>

<style>
    .tutorial-container {
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        height: 100vh;
        width: 100%;
        padding: 2rem;
        box-sizing: border-box;
    }
    
    .tutorial-content {
        flex-grow: 1;
        display: flex;
        flex-direction: column;
        justify-content: center;
    }
    
    .tutorial-navigation {
        position: relative;
        height: 60px;
    }
    
    .progress-dots {
        display: flex;
        justify-content: center;
        gap: 10px;
        margin-bottom: 20px;
    }
    
    .dot {
        width: 10px;
        height: 10px;
        border-radius: 50%;
        background-color: rgba(255, 255, 255, 0.3);
    }
    
    .dot.active {
        background-color: white;
    }

    h1 {
        font-size: 3rem;
        margin-bottom: 1rem;
    }

    h2 {
        font-size: 2rem;
        margin-bottom: 1rem;
    }
    
    h4 {
        font-size: 1.2rem;
        margin-top: 1rem;
        opacity: 0.8;
    }

    .bottomright {
        position: absolute;
        bottom: 10px;
        right: 10px;
        animation: bounce 1s infinite;
        margin: 0;
    }

    @keyframes bounce {
        0% {transform: translateY(0);}
        50% {transform: translateY(-10px);}
        100% {transform: translateY(0);}
    }
</style>