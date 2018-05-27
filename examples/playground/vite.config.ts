import { execSync } from "child_process";
import path from "path";
import { defineConfig, type Plugin } from "vite";

function hxoPlugin(): Plugin {
    return {
        name: "vite-plugin-hxo",
        transform(code, id) {
            if (id.endsWith(".hxo") || id.includes(".hxo?")) {
                const filePath = id.split("?")[0];
                const compilerPath = path.resolve(
                    __dirname,
                    "../../target/debug/hxo-compile.exe",
                );
                const result = execSync(`${compilerPath} ${filePath}`);
                return {
                    code: result.toString(),
                    map: null,
                };
            }
        },
    };
}

export default defineConfig({
    plugins: [hxoPlugin()],
    resolve: {
        alias: {
            "@hxo/core": path.resolve(
                __dirname,
                "../../runtimes/hxo-core/src/index.ts",
            ),
            "@hxo/dom": path.resolve(
                __dirname,
                "../../runtimes/hxo-dom/src/index.ts",
            ),
            "@hxo/shared": path.resolve(
                __dirname,
                "../../runtimes/hxo-shared/src/index.ts",
            ),
        },
    },
    server: {
        port: 5173,
    },
    optimizeDeps: {
        include: ["monaco-editor/esm/vs/editor/editor.worker"],
    },
});
