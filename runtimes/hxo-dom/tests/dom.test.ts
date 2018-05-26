/**
 * @vitest-environment happy-dom
 */
import { describe, it, expect } from 'vitest';
import { h, createSignal, nextTick } from '@hxo/core';
import { renderComponent, mount } from '../src/index';

describe('dom renderer', () => {
    it('should mount a simple vnode', () => {
        const container = document.createElement('div');
        const vnode = h('div', { id: 'test' }, 'hello');
        mount(vnode, container);
        expect(container.innerHTML).toBe('<div id="test">hello</div>');
    });

    it('should render a reactive component', async () => {
        const container = document.createElement('div');
        const [count, setCount] = createSignal(0);
        
        const MyComponent = {
            setup() {
                return { count };
            },
            render({ count }: { count: () => number }) {
                return h('div', null, [
                    h('span', null, `Count: ${count()}`),
                    h('button', { onClick: () => setCount(count() + 1) }, 'Increment')
                ]);
            }
        };

        renderComponent(MyComponent, container);
        expect(container.innerHTML).toContain('Count: 0');

        const button = container.querySelector('button');
        button?.click();

        await nextTick();
        expect(container.innerHTML).toContain('Count: 1');
    });

    it('should handle complex patching', async () => {
        const container = document.createElement('div');
        const [show, setShow] = createSignal(true);
        
        const ToggleComponent = {
            setup() {
                return { show };
            },
            render({ show }: { show: () => boolean }) {
                return h('div', null, 
                    show() 
                        ? [h('p', { key: 'a' }, 'Visible'), h('span', null, 'Text')]
                        : [h('p', { key: 'a' }, 'Hidden')]
                );
            }
        };

        renderComponent(ToggleComponent, container);
        expect(container.innerHTML).toContain('Visible');
        expect(container.innerHTML).toContain('Text');

        setShow(false);
        await nextTick();
        expect(container.innerHTML).toContain('Hidden');
        expect(container.innerHTML).not.toContain('Visible');
        expect(container.innerHTML).not.toContain('Text');
    });
});
