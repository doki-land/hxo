import { hxoHandler } from "./lib/hxo_wit.js";

async function test() {
    try {
        const template = `<template><div>{{ count }}</div></template>
<script>
const [count, setCount] = createSignal(0);
</script>`;

        console.log("Compiling template...");
        const result = hxoHandler.render(template, {
            minify: false,
            sourceMap: true,
        });

        console.log("Result:", result);
    } catch (e) {
        console.error("Error:", e);
    }
}

test();
