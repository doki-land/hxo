use hxo_compiler::Compiler;

#[test]
fn test_compiler_pipeline() {
    let mut compiler = Compiler::new();
    let source = r#"
<script>
const [count, setCount] = createSignal(0);
function inc() { setCount(count() + 1); }
</script>
<template>
  <div class="p-4 text-blue">
    <span>Count: {{ count }}</span>
    <button @click="inc">+</button>
  </div>
</template>
"#;
    let js = compiler.compile("Counter", source).unwrap();
    let css = compiler.get_css();

    // Check CSS
    assert!(css.contains(".p-4"));
    assert!(css.contains(".text-blue"));

    // Check JS Output (Pure JS as per whitebook.md)
    assert!(js.code.contains("export default {"));
    assert!(js.code.contains("name: 'Counter'"));
    assert!(js.code.contains("setup(props"));
    assert!(js.code.contains("render(ctx)"));
    assert!(js.code.contains("h('div'"));
    assert!(js.code.contains("h('button'"));
    assert!(js.code.contains("createTextVNode(ctx.count)"));
}

#[test]
fn test_compiler_full_cycle() {
    let mut compiler = Compiler::new();
    let source = r#"
<template>
  <div class="container">
    <h1 @click="increment">Count: {{ count }}</h1>
    <button :disabled="isMax">Add</button>
  </div>
</template>

<script>
const [count, setCount] = createSignal(0);
const isMax = createComputed(() => count() >= 10);
function increment() {
    setCount(count() + 1);
}
</script>
"#;
    let js = compiler.compile("App", source).unwrap();
    println!("Generated JS:\n{}", js.code);

    // Check imports and component structure
    assert!(js.code.contains("import { Fragment, createComputed, createSignal } from '@hxo/core';"));
    assert!(js.code.contains("import { createTextVNode, h } from '@hxo/dom';"));
    assert!(js.code.contains("name: 'App'"));
    assert!(js.code.contains("setup(props"));
    assert!(js.code.contains("render(ctx)"));

    // Check script content
    assert!(js.code.contains("const [count, setCount] = createSignal(0);"));
    assert!(js.code.contains("function increment()"));
    assert!(js.code.contains("return { count, setCount, isMax, increment }"));

    // Check render content
    assert!(js.code.contains("h('div', { 'class': 'container' }"));
    assert!(js.code.contains("h('h1', { 'onClick': ctx.increment }"));
    assert!(js.code.contains("createTextVNode('Count:')"));
    // Check reactivity (signals should be called as functions)
    assert!(js.code.contains("createTextVNode(ctx.count)"));
    assert!(js.code.contains("h('button', { 'disabled': ctx.isMax }"));
}
