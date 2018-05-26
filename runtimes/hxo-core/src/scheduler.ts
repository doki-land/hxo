export type Task = () => void;

const queue: Task[] = [];
let isFlushing = false;
const p = Promise.resolve();

/**
 * Push a task to the queue and schedule a flush.
 */
export function nextTick(fn?: () => void): Promise<void> {
    return fn ? p.then(fn) : p;
}

/**
 * Queue a job for the next tick.
 */
export function queueJob(job: Task): void {
    if (!queue.includes(job)) {
        queue.push(job);
        scheduleFlush();
    }
}

function scheduleFlush(): void {
    if (!isFlushing) {
        isFlushing = true;
        nextTick(flushJobs);
    }
}

function flushJobs(): void {
    try {
        for (let i = 0; i < queue.length; i++) {
            queue[i]();
        }
    } finally {
        isFlushing = false;
        queue.length = 0;
    }
}
