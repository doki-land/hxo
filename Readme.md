# HXO: A Better Vue

> **Rust 驱动的极速编译，Signal 原生的极致性能**

HXO 是一款高性能 Web 框架，旨在提供无缝的类 Vue 开发体验的同时，通过底层架构的根本性革新，彻底解决传统框架在性能和心智负担上的痛点。

---

## 💡 核心哲学

### 1. 原生响应式 (Signal First)
HXO 彻底摒弃了 VDOM 这种先污染后治理的更新模式。
- **精准打击**：基于 Signal 的响应式系统实现了**原子级更新**。当状态改变时，框架能精准定位并仅更新受影响的 DOM 节点。
- **按需计算**：没有全量 `diff`，没有不必要的组件重渲染，运行效率直逼原生 JS。

### 2. 编译器魔法 (Compiler Magic)
我们利用 Rust 编写的编译器实现了深度 **Reactive Transform**，让“魔法”发生在编译时而非运行时。
- **无 `.value`**：开发者可以像操作普通变量一样操作 `ref`。
- **零心智负担**：告别 `count.value` 或 `count()` 这种冗余语法。你写的是 JavaScript 的直觉，编译器产出的是极致优化的代码。
- **静态提升**：编译器自动分析并提升静态节点，进一步减少内存占用。

## 📦 项目结构

HXO 采用了高度模块化的单仓架构：

- **`compilers/`**: 核心 Rust 编译器集群。包含解析器 (Parser)、中间表示 (IR)、代码生成器 (Codegen) 以及命令行工具 (CLI)。
- **`runtimes/`**: 极简 JavaScript 运行时。核心包仅 ~3KB，包含响应式系统、轻量化 DOM 渲染及 SSR 支持。
- **`examples/`**: 包含功能丰富的交互式 [Playground](examples/playground)，可实时查看 HXO 源码到优化后 JS 的转换过程。
- **`scripts/`**: 自动化构建、测试与性能分析脚本。

## 🚀 快速上手

感受 HXO 带来的清爽编码体验：

```html
<!-- App.HXO -->
<template>
  <div class="counter-card">
    <h1>HXO: A Better Vue</h1>
    <p>计数器: {{ count }}</p>
    <p>双倍值: {{ double }}</p>
    <button @click="increment">累加</button>
  </div>
</template>

<script>
import { ref, computed } from '@hxo/core';

// 编译器会自动处理响应式转换，无需 .value
let count = ref(0);
const double = computed(() => count * 2);

const increment = () => {
  count++; // 直接操作变量，简单直观
};
</script>

<style scoped>
.counter-card {
  padding: 2rem;
  border-radius: 12px;
  box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
}
</style>
```

## 🛠️ A Better Vue

1. **更小**：运行时体积仅为 Vue 的 1/10，且完全支持 Tree-shaking。
2. **更快**：Rust 编译器比原生 JS 编译器快 10-100 倍，极大提升 HMR 体验。
3. **更简单**：彻底告别 `.value`，回归最纯粹的编程逻辑。
4. **更强**：深度 AOT 优化与 Signal-First 架构，确保在超大规模应用中依然保持丝滑。
