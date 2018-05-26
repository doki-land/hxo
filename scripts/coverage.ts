import { spawnSync } from "node:child_process";

console.log("\x1b[36mRunning test coverage...\x1b[0m");

const result = spawnSync("cargo", ["llvm-cov", "--workspace", "--html"], {
    stdio: "inherit",
    shell: true,
});

if (result.status === 0) {
    console.log(
        "\x1b[32mCoverage report generated in target/llvm-cov/html/index.html\x1b[0m",
    );
} else {
    console.error("\x1b[31mFailed to generate coverage report.\x1b[0m");
    process.exit(result.status ?? 1);
}
