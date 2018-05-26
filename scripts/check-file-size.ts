import fs from "node:fs";
import path from "node:path";

const WARNING_LIMIT = 800;
const ERROR_LIMIT = 1000;

const IGNORE_DIRS = ["node_modules", "target", ".git", "dist", "coverage", "lib"];

const EXTENSIONS = [".ts", ".rs", ".js", ".tsx", ".jsx"];

let errorCount = 0;
let warningCount = 0;

function checkFiles(dir: string) {
    const files = fs.readdirSync(dir, { withFileTypes: true });

    for (const file of files) {
        const fullPath = path.join(dir, file.name);

        if (file.isDirectory()) {
            if (IGNORE_DIRS.includes(file.name)) continue;
            checkFiles(fullPath);
        } else if (file.isFile()) {
            if (!EXTENSIONS.includes(path.extname(file.name))) continue;

            const content = fs.readFileSync(fullPath, "utf-8");
            const lines = content.split("\n").length;

            if (lines > ERROR_LIMIT) {
                console.error(
                    `\x1b[31m[ERROR]\x1b[0m ${fullPath}: ${lines} lines (Must be split, exceeds ${ERROR_LIMIT} lines)`,
                );
                errorCount++;
            } else if (lines > WARNING_LIMIT) {
                console.warn(
                    `\x1b[33m[WARNING]\x1b[0m ${fullPath}: ${lines} lines (Should be split, exceeds ${WARNING_LIMIT} lines)`,
                );
                warningCount++;
            }
        }
    }
}

console.log("Checking file sizes...");
const rootDir = path.join(__dirname, "..");
checkFiles(rootDir);

if (errorCount > 0) {
    console.log(
        `\n\x1b[31mFound ${errorCount} error(s) and ${warningCount} warning(s).\x1b[0m`,
    );
    process.exit(1);
} else if (warningCount > 0) {
    console.log(
        `\n\x1b[33mFound ${warningCount} warning(s). All files are within mandatory limits.\x1b[0m`,
    );
} else {
    console.log("\x1b[32mAll files are within size limits.\x1b[0m");
}
