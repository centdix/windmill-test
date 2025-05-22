import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import CounterCard from './CounterCard.svelte';

describe('CounterCard Component', () => {
    it('renders with default props', () => {
        const { getByText } = render(CounterCard);
        
        // Check if the default title is rendered
        expect(getByText('Counter')).toBeTruthy();
        
        // Check if the default count is rendered
        expect(getByText('0')).toBeTruthy();
        
        // Check if the min, max, and step values are rendered
        expect(getByText('Min: 0 | Max: 100 | Step: 1')).toBeTruthy();
    });
    
    it('renders with custom props', () => {
        const { getByText } = render(CounterCard, {
            props: {
                title: 'Custom Counter',
                initialValue: 10,
                minValue: 5,
                maxValue: 15,
                stepSize: 2
            }
        });
        
        // Check if the custom title is rendered
        expect(getByText('Custom Counter')).toBeTruthy();
        
        // Check if the custom count is rendered
        expect(getByText('10')).toBeTruthy();
        
        // Check if the custom min, max, and step values are rendered
        expect(getByText('Min: 5 | Max: 15 | Step: 2')).toBeTruthy();
    });
    
    it('increments the counter when clicking the increment button', async () => {
        const { getByText } = render(CounterCard, {
            props: {
                initialValue: 5
            }
        });
        
        // Get the increment button and click it
        const incrementButton = getByText('+');
        await fireEvent.click(incrementButton);
        
        // Check if the count has been incremented
        expect(getByText('6')).toBeTruthy();
        
        // Check if the action message is displayed
        expect(getByText('Counter incremented by 1')).toBeTruthy();
    });
    
    it('decrements the counter when clicking the decrement button', async () => {
        const { getByText } = render(CounterCard, {
            props: {
                initialValue: 5
            }
        });
        
        // Get the decrement button and click it
        const decrementButton = getByText('-');
        await fireEvent.click(decrementButton);
        
        // Check if the count has been decremented
        expect(getByText('4')).toBeTruthy();
        
        // Check if the action message is displayed
        expect(getByText('Counter decremented by 1')).toBeTruthy();
    });
    
    it('resets the counter when clicking the reset button', async () => {
        const { getByText } = render(CounterCard, {
            props: {
                initialValue: 5
            }
        });
        
        // First, increment the counter
        const incrementButton = getByText('+');
        await fireEvent.click(incrementButton);
        expect(getByText('6')).toBeTruthy();
        
        // Then, click the reset button
        const resetButton = getByText('Reset');
        await fireEvent.click(resetButton);
        
        // Check if the count has been reset
        expect(getByText('5')).toBeTruthy();
        
        // Check if the action message is displayed
        expect(getByText('Counter reset to 5')).toBeTruthy();
    });
    
    it('disables the increment button when at max value', async () => {
        const { getByText } = render(CounterCard, {
            props: {
                initialValue: 9,
                maxValue: 10
            }
        });
        
        // Get the increment button
        const incrementButton = getByText('+');
        
        // Click it once to reach the max value
        await fireEvent.click(incrementButton);
        expect(getByText('10')).toBeTruthy();
        
        // The button should now be disabled
        expect(incrementButton).toBeDisabled();
    });
    
    it('disables the decrement button when at min value', () => {
        const { getByText } = render(CounterCard, {
            props: {
                initialValue: 0,
                minValue: 0
            }
        });
        
        // Get the decrement button
        const decrementButton = getByText('-');
        
        // The button should be disabled
        expect(decrementButton).toBeDisabled();
    });
});