import { Fragment, Text, type VNode } from "@hxo/core";

/**
 * Render a VNode to a string
 */
export function renderToString(vnode: VNode): string {
    if (vnode.type === Text) {
        return vnode.children as string;
    }

    if (vnode.type === Fragment) {
        if (Array.isArray(vnode.children)) {
            return vnode.children
                .map((child) => renderToString(child))
                .join("");
        }
        return "";
    }

    if (typeof vnode.type === "string") {
        let props = "";
        if (vnode.props) {
            for (const key in vnode.props) {
                if (!key.startsWith("on")) {
                    props += ` ${key}="${vnode.props[key]}"`;
                }
            }
        }

        const tag = vnode.type;
        let children = "";
        if (typeof vnode.children === "string") {
            children = vnode.children;
        } else if (Array.isArray(vnode.children)) {
            children = vnode.children
                .map((child) => renderToString(child))
                .join("");
        }

        return `<${tag}${props}>${children}</${tag}>`;
    }

    return "";
}

/**
 * Render a component to a string
 */
export function renderComponentToString(component: any): string {
    const setupResult = component.setup ? component.setup() : {};
    const vnode = component.render(setupResult);
    return renderToString(vnode);
}
