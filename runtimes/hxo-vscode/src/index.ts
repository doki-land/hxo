import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";
import {
    LanguageClient,
    type LanguageClientOptions,
    type ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    console.log("HXO extension is now active!");

    const config = vscode.workspace.getConfiguration("hxo");
    let serverPath = config.get<string>("lsp.path") || "hxo-lsp";

    // If we are in development and the server path is just 'hxo-lsp',
    // try to find it in the workspace's target directory
    if (serverPath === "hxo-lsp" && vscode.workspace.workspaceFolders) {
        const workspaceRoot = vscode.workspace.workspaceFolders[0].uri.fsPath;
        const possiblePaths = [
            path.join(workspaceRoot, "target", "debug", "hxo-lsp"),
            path.join(workspaceRoot, "target", "release", "hxo-lsp"),
            path.join(
                workspaceRoot,
                "compilers",
                "hxo-lsp",
                "target",
                "debug",
                "hxo-lsp",
            ),
            path.join(
                workspaceRoot,
                "compilers",
                "hxo-lsp",
                "target",
                "release",
                "hxo-lsp",
            ),
        ];

        // Add .exe extension on Windows
        const pathsToTry =
            process.platform === "win32"
                ? possiblePaths.map((p) => p + ".exe")
                : possiblePaths;

        for (const p of pathsToTry) {
            if (fs.existsSync(p)) {
                serverPath = p;
                break;
            }
        }
    }

    const serverOptions: ServerOptions = {
        command: serverPath,
        options: {
            env: {
                ...process.env,
                RUST_LOG: "info",
            },
        },
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: "file", language: "hxo" }],
        synchronize: {
            fileEvents:
                vscode.workspace.createFileSystemWatcher("**/hxo.config.toml"),
        },
    };

    client = new LanguageClient(
        "hxoLanguageServer",
        "HXO Language Server",
        serverOptions,
        clientOptions,
    );

    client.start();
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
