import { queueJob } from "./scheduler";

// Signal types
export type Getter<T> = () => T;
export type Setter<T> = (newValue: T) => void;

let currentEffect: (() => void) | null = null;

/**
 * Creates a reactive signal.
 */
export function createSignal<T>(value: T): [Getter<T>, Setter<T>] {
    let internalValue = value;
    const subscribers = new Set<() => void>();

    const getter: Getter<T> = () => {
        if (currentEffect) {
            subscribers.add(currentEffect);
        }
        return internalValue;
    };

    const setter: Setter<T> = (newValue: T) => {
        if (Object.is(internalValue, newValue)) return;
        internalValue = newValue;
        subscribers.forEach((effect) => queueJob(effect));
    };

    return [getter, setter];
}

/**
 * Creates a reactive effect.
 */
export function createEffect(fn: () => void): void {
    const effect = () => {
        const prevEffect = currentEffect;
        currentEffect = effect;
        try {
            fn();
        } finally {
            currentEffect = prevEffect;
        }
    };
    effect();
}

/**
 * Creates a computed signal that automatically updates when dependencies change.
 */
export function createComputed<T>(fn: () => T): Getter<T> {
    const [getter, setter] = createSignal<T>(undefined as any);
    createEffect(() => {
        setter(fn());
    });
    return getter;
}
