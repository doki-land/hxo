export interface VNode {
    type: string | symbol;
    props?: Record<string, any>;
    children?: VNode[] | string;
    el?: any; // Platform specific element
}

export const Text = Symbol("Text");
export const Fragment = Symbol("Fragment");

/**
 * Create a Virtual Node
 */
export function h(
    type: string | symbol,
    props?: Record<string, any>,
    children?: any,
): VNode {
    return { type, props, children };
}

/**
 * Create a Text Virtual Node
 */
export function createTextVNode(text: string): VNode {
    return h(Text, undefined, text);
}
