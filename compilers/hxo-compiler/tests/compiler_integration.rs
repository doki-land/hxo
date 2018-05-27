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

#[test]
fn test_compiler_tailwind() {
    let mut compiler = Compiler::new();
    let source = r#"
<script>
addStyle("p-6 m-4");
</script>
<template>
  <div class="p-4 m-2 flex items-center bg-blue text-white rounded-lg shadow-sm">
    <span :class="'font-bold text-2xl'">Tailwind Test</span>
    <p :class="['text-center', 'mx-4']">Dynamic array</p>
  </div>
</template>
"#;
    let res = compiler.compile("TailwindComp", source).unwrap();
    let css = res.css;

    // Check CSS from static template classes
    assert!(css.contains(".p-4"));
    assert!(css.contains("padding: 1rem;"));
    assert!(css.contains(".m-2"));
    assert!(css.contains("margin: 0.5rem;"));
    assert!(css.contains(".flex"));
    assert!(css.contains("display: flex;"));
    assert!(css.contains(".items-center"));
    assert!(css.contains("align-items: center;"));
    assert!(css.contains(".bg-blue"));
    assert!(css.contains("background-color: #0000ff;"));
    assert!(css.contains(".text-white"));
    assert!(css.contains("color: #ffffff;"));
    assert!(css.contains(".rounded-lg"));
    assert!(css.contains("border-radius: 0.5rem;"));
    assert!(css.contains(".shadow-sm"));

    // Check CSS from :class dynamic template classes (static parts extracted)
    assert!(css.contains(".font-bold"));
    assert!(css.contains("font-weight: 700;"));
    assert!(css.contains(".text-2xl"));
    assert!(css.contains("font-size: 1.5rem;"));
    assert!(css.contains(".text-center"));
    assert!(css.contains("text-align: center;"));
    assert!(css.contains(".mx-4"));
    assert!(css.contains("margin-left: 1rem;"));
    assert!(css.contains("margin-right: 1rem;"));

    // Check CSS from script addStyle calls
    assert!(css.contains(".p-6"));
    assert!(css.contains("padding: 1.5rem;"));
    assert!(css.contains(".m-4"));
    assert!(css.contains("margin: 1rem;"));
}

#[test]
fn test_compiler_pug() {
    let mut compiler = Compiler::new();
    let source = r#"
<template lang="pug">
div.container
  h1 Hello Pug
  p(id="desc", :class="activeClass") This is a pug template
</template>
"#;
    let res = compiler.compile("PugComp", source).unwrap();
    println!("Generated Pug JS:\n{}", res.code);

    // Check if the output contains the correct h calls
    assert!(res.code.contains("h('div', { 'class': 'container' }"));
    assert!(res.code.contains("h('h1', null"));
    assert!(res.code.contains("createTextVNode('Hello Pug')"));
    assert!(res.code.contains("h('p', { 'id': 'desc', 'class': ctx.activeClass }"));
    assert!(res.code.contains("createTextVNode('This is a pug template')"));
}
