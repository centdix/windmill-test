<script lang="ts">
    import { twMerge } from 'tailwind-merge';
    import { fade } from 'svelte/transition';
    
    /**
     * CounterCard - A simple component demonstrating Svelte 5 features
     * This component displays a card with a counter that can be incremented or decremented
     */

    // Props interface
    interface CounterCardProps {
        title?: string;
        initialValue?: number;
        minValue?: number;
        maxValue?: number;
        stepSize?: number;
        class?: string;
    }

    // Define component props using Svelte 5's $props()
    let { 
        title = 'Counter', 
        initialValue = 0, 
        minValue = 0, 
        maxValue = 100,
        stepSize = 1,
        class: className = ''
    }: CounterCardProps = $props();

    // Reactive state using $state
    let count = $state(initialValue);
    let lastAction = $state<string | null>(null);
    let showActionMessage = $state(false);

    // Computed values with $derived
    const isAtMin = $derived(count <= minValue);
    const isAtMax = $derived(count >= maxValue);

    // Functions to update state
    function increment() {
        if (count < maxValue) {
            count = Math.min(maxValue, count + stepSize);
            updateLastAction('increment');
        }
    }

    function decrement() {
        if (count > minValue) {
            count = Math.max(minValue, count - stepSize);
            updateLastAction('decrement');
        }
    }

    function reset() {
        count = initialValue;
        updateLastAction('reset');
    }

    function updateLastAction(action: string) {
        lastAction = action;
        showActionMessage = true;
        
        // Hide the message after 2 seconds
        setTimeout(() => {
            showActionMessage = false;
        }, 2000);
    }

    // Reactive effect using $effect
    $effect(() => {
        console.log(`Counter value changed to ${count}`);
    });
</script>

<div class={twMerge('bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 max-w-md mx-auto', className)}>
    <h2 class="text-xl font-semibold mb-4 text-gray-800 dark:text-white">{title}</h2>
    
    <div class="flex items-center justify-center my-4">
        <button 
            on:click={decrement}
            class="bg-blue-500 hover:bg-blue-600 text-white py-2 px-4 rounded-l-md disabled:opacity-50 disabled:cursor-not-allowed"
            disabled={isAtMin}
        >
            -
        </button>
        
        <div class="bg-gray-100 dark:bg-gray-700 py-2 px-6 text-center text-xl font-semibold text-gray-800 dark:text-white">
            {count}
        </div>
        
        <button 
            on:click={increment}
            class="bg-blue-500 hover:bg-blue-600 text-white py-2 px-4 rounded-r-md disabled:opacity-50 disabled:cursor-not-allowed"
            disabled={isAtMax}
        >
            +
        </button>
    </div>
    
    <div class="flex justify-center mt-4">
        <button 
            on:click={reset}
            class="bg-gray-300 dark:bg-gray-600 hover:bg-gray-400 dark:hover:bg-gray-500 text-gray-800 dark:text-white py-1 px-4 rounded-md text-sm"
        >
            Reset
        </button>
    </div>
    
    {#if showActionMessage}
        <div 
            transition:fade={{ duration: 200 }}
            class="mt-4 text-center text-sm text-gray-600 dark:text-gray-300"
        >
            {#if lastAction === 'increment'}
                Counter incremented by {stepSize}
            {:else if lastAction === 'decrement'}
                Counter decremented by {stepSize}
            {:else if lastAction === 'reset'}
                Counter reset to {initialValue}
            {/if}
        </div>
    {/if}
    
    <div class="mt-6 text-xs text-gray-500 dark:text-gray-400 text-center">
        Min: {minValue} | Max: {maxValue} | Step: {stepSize}
    </div>
</div>

<style>
    /* Any component-specific styles can go here if needed */
</style>