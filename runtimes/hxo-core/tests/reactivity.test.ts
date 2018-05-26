import { describe, it, expect, vi } from 'vitest';
import { createSignal, createEffect, createComputed } from '../src/reactivity';
import { nextTick } from '../src/scheduler';

describe('reactivity', () => {
    it('should work with basic signal', () => {
        const [count, setCount] = createSignal(0);
        expect(count()).toBe(0);
        setCount(1);
        expect(count()).toBe(1);
    });

    it('should trigger effect when signal changes', async () => {
        const [count, setCount] = createSignal(0);
        const spy = vi.fn();
        
        createEffect(() => {
            spy(count());
        });

        expect(spy).toHaveBeenCalledWith(0);
        expect(spy).toHaveBeenCalledTimes(1);

        setCount(1);
        await nextTick();
        expect(spy).toHaveBeenCalledWith(1);
        expect(spy).toHaveBeenCalledTimes(2);
    });

    it('should work with computed signal', async () => {
        const [count, setCount] = createSignal(1);
        const doubled = createComputed(() => count() * 2);

        expect(doubled()).toBe(2);

        setCount(2);
        await nextTick();
        expect(doubled()).toBe(4);
    });

    it('should handle nested effects', async () => {
        const [count, setCount] = createSignal(0);
        const spy1 = vi.fn();
        const spy2 = vi.fn();

        createEffect(() => {
            spy1(count());
            createEffect(() => {
                spy2(count());
            });
        });

        expect(spy1).toHaveBeenCalledWith(0);
        expect(spy2).toHaveBeenCalledWith(0);
        expect(spy1).toHaveBeenCalledTimes(1);
        expect(spy2).toHaveBeenCalledTimes(1);

        setCount(1);
        await nextTick();
        expect(spy1).toHaveBeenCalledWith(1);
        expect(spy2).toHaveBeenCalledWith(1);
        expect(spy1).toHaveBeenCalledTimes(2);
        // Note: The nested effect will be re-created when the outer effect runs,
        // and it also runs once immediately upon creation.
        expect(spy2).toHaveBeenCalledTimes(3); 
    });
});
