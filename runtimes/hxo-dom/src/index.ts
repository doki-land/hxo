import type { VNode } from "@hxo/core";
import { createEffect, Fragment, h, Text, createTextVNode } from "@hxo/core";

export { Text, Fragment, h, createTextVNode };
export type { VNode };

/**
 * Mount a VNode to a DOM container
 */
export function mount(vnode: VNode, container: HTMLElement): void {
    const el = createElement(vnode);
    if (el) {
        container.appendChild(el);
        vnode.el = el;
    }
}

/**
 * Render a component reactively
 */
export function renderComponent(component: any, container: HTMLElement): void {
    let oldVNode: VNode | null = null;
    const setupResult = component.setup ? component.setup() : {};

    createEffect(() => {
        const newVNode = component.render(setupResult);
        if (!oldVNode) {
            mount(newVNode, container);
        } else {
            patch(oldVNode, newVNode, container);
        }
        oldVNode = newVNode;
    });
}

/**
 * Patch two VNodes
 */
export function patch(
    oldVNode: VNode,
    newVNode: VNode,
    container: HTMLElement,
): void {
    if (oldVNode.type !== newVNode.type) {
        // Replace if type is different
        const el = createElement(newVNode);
        if (el && oldVNode.el) {
            container.replaceChild(el, oldVNode.el as Node);
        }
    } else {
        const el = (newVNode.el = oldVNode.el as HTMLElement);

        // Update Props
        patchProps(el, oldVNode.props || {}, newVNode.props || {});

        // Update Children
        patchChildren(el, oldVNode.children, newVNode.children);
    }
}

function patchProps(
    el: HTMLElement,
    oldProps: Record<string, any>,
    newProps: Record<string, any>,
) {
    // Remove old props
    for (const key in oldProps) {
        if (!(key in newProps)) {
            if (key.startsWith("on")) {
                el.removeEventListener(
                    key.slice(2).toLowerCase(),
                    oldProps[key],
                );
            } else {
                (el as any)[key] = undefined;
            }
        }
    }
    // Add/Update new props
    for (const key in newProps) {
        if (oldProps[key] !== newProps[key]) {
            if (key.startsWith("on")) {
                const eventName = key.slice(2).toLowerCase();
                if (oldProps[key])
                    el.removeEventListener(eventName, oldProps[key]);
                el.addEventListener(eventName, newProps[key]);
            } else {
                (el as any)[key] = newProps[key];
            }
        }
    }
}

function patchChildren(el: HTMLElement, oldChildren: any, newChildren: any) {
    if (typeof newChildren === "string") {
        if (oldChildren !== newChildren) {
            el.textContent = newChildren;
        }
    } else if (Array.isArray(newChildren)) {
        if (Array.isArray(oldChildren)) {
            // Simple diff: sync lengths
            const commonLength = Math.min(
                oldChildren.length,
                newChildren.length,
            );
            for (let i = 0; i < commonLength; i++) {
                patch(oldChildren[i], newChildren[i], el);
            }
            if (newChildren.length > oldChildren.length) {
                newChildren.slice(commonLength).forEach((child) => {
                    const childEl = createElement(child);
                    if (childEl) el.appendChild(childEl);
                });
            } else if (oldChildren.length > newChildren.length) {
                oldChildren.slice(commonLength).forEach((child) => {
                    if (child.el) el.removeChild(child.el);
                });
            }
        } else {
            el.textContent = "";
            newChildren.forEach((child) => {
                const childEl = createElement(child);
                if (childEl) el.appendChild(childEl);
            });
        }
    }
}

function createElement(vnode: VNode): Node | null {
    if (vnode.type === Text) {
        return document.createTextNode(vnode.children as string);
    }

    if (vnode.type === Fragment) {
        const fragment = document.createDocumentFragment();
        if (Array.isArray(vnode.children)) {
            vnode.children.forEach((child) => {
                const childEl = createElement(child);
                if (childEl) fragment.appendChild(childEl);
            });
        }
        return fragment;
    }

    if (typeof vnode.type === "string") {
        const el = document.createElement(vnode.type);

        // Props
        if (vnode.props) {
            for (const key in vnode.props) {
                if (key.startsWith("on")) {
                    el.addEventListener(
                        key.slice(2).toLowerCase(),
                        vnode.props[key],
                    );
                } else {
                    (el as any)[key] = vnode.props[key];
                }
            }
        }

        // Children
        if (typeof vnode.children === "string") {
            el.textContent = vnode.children;
        } else if (Array.isArray(vnode.children)) {
            vnode.children.forEach((child) => {
                const childEl = createElement(child);
                if (childEl) el.appendChild(childEl);
            });
        }

        vnode.el = el;
        return el;
    }

    return null;
}
