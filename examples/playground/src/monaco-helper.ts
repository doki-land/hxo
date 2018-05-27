export async function initEditor(
    container: HTMLElement,
    initialValue: string,
    language: string,
    onChange: (value: string) => void,
) {
    const monaco = await import("monaco-editor");

    // Register HXO language if not already registered
    if (!monaco.languages.getLanguages().some((lang) => lang.id === "hxo")) {
        monaco.languages.register({ id: "hxo" });

        // Define syntax highlighting for HXO
        monaco.languages.setMonarchTokensProvider("hxo", {
            defaultToken: "",
            tokenPostfix: ".hxo",
            ignoreCase: true,

            keywords: [
                "if",
                "else",
                "for",
                "of",
                "in",
                "return",
                "const",
                "let",
                "var",
                "function",
                "async",
                "await",
            ],

            tokenizer: {
                root: [
                    // Blocks
                    [/<template>/, { token: "tag", next: "@template" }],
                    [/<script>/, { token: "tag", next: "@script" }],
                    [/<style(\s+scoped)?>/, { token: "tag", next: "@style" }],
                    [/<meta>/, { token: "tag", next: "@meta" }],
                    [/<router>/, { token: "tag", next: "@router" }],
                    { include: "html" },
                ],

                template: [
                    [/<\/template>/, { token: "tag", next: "@pop" }],
                    { include: "html" },
                ],

                script: [
                    [/<\/script>/, { token: "tag", next: "@pop" }],
                    [/\/\/.*$/, "comment"],
                    [/\/\*/, "comment", "@comment"],
                    [
                        /[a-zA-Z_]\w*/,
                        {
                            cases: {
                                "@keywords": "keyword",
                                "@default": "identifier",
                            },
                        },
                    ],
                    [/[{}()[\]]/, "@brackets"],
                    [/[0-9]+/, "number"],
                    [/"([^"\\]|\\.)*"/, "string"],
                    [/'([^'\\]|\\.)*'/, "string"],
                    [/`/, "string", "@string_backtick"],
                ],

                comment: [
                    [/[^/*]+/, "comment"],
                    [/\*\//, "comment", "@pop"],
                    [/[/*]/, "comment"],
                ],

                string_backtick: [
                    [/[^\\`$]+/, "string"],
                    [/\\./, "string.escape"],
                    [
                        /\${/,
                        {
                            token: "delimiter.bracket",
                            next: "@bracket_counting",
                        },
                    ],
                    [/`/, "string", "@pop"],
                ],

                bracket_counting: [
                    [/\{/, "delimiter.bracket", "@push"],
                    [/\}/, "delimiter.bracket", "@pop"],
                    { include: "script" },
                ],

                style: [
                    [/<\/style>/, { token: "tag", next: "@pop" }],
                    [/[^{};]+/, "attribute.name"],
                    [/\{/, "delimiter.curly", "@style_body"],
                ],

                style_body: [
                    [/\}/, "delimiter.curly", "@pop"],
                    [/[^:]+/, "attribute.name"],
                    [/[:]/, "delimiter"],
                    [/[^;]+/, "attribute.value"],
                    [/;/, "delimiter"],
                ],

                meta: [
                    [/<\/meta>/, { token: "tag", next: "@pop" }],
                    [/[\s\S]*?(?=<\/meta>)/, "text.json"],
                ],

                router: [
                    [/<\/router>/, { token: "tag", next: "@pop" }],
                    [/[\s\S]*?(?=<\/router>)/, "text.json"],
                ],

                html: [
                    [/<!--/, "comment", "@html_comment"],
                    [/<[a-zA-Z0-9-]+/, "tag", "@html_tag"],
                    [/<\/ [a-zA-Z0-9-]+>/, "tag"],
                    [
                        /{{/,
                        { token: "delimiter.curly", next: "@interpolation" },
                    ],
                    [/[^<]+/, ""],
                ],

                html_comment: [
                    [/-->/, "comment", "@pop"],
                    [/[^-]+/, "comment"],
                    [/./, "comment"],
                ],

                html_tag: [
                    [/>/, "tag", "@pop"],
                    [/@\w+/, "attribute.name"], // Directives like @click
                    [/v-\w+/, "attribute.name"], // Directives like v-if
                    [/:\w+/, "attribute.name"], // Bindings like :class
                    [/\w+/, "attribute.name"],
                    [/=/, "delimiter"],
                    [/"[^"]*"/, "attribute.value"],
                    [/'[^']*'/, "attribute.value"],
                    [/\s+/, ""],
                ],

                interpolation: [
                    [/}}/, { token: "delimiter.curly", next: "@pop" }],
                    [/[\s\S]+?/, "variable"],
                ],
            },
        });

        // Set configuration for HXO
        monaco.languages.setLanguageConfiguration("hxo", {
            comments: {
                blockComment: ["<!--", "-->"],
            },
            brackets: [
                ["<!--", "-->"],
                ["<", ">"],
                ["{", "}"],
                ["(", ")"],
            ],
            autoClosingPairs: [
                { open: "{", close: "}" },
                { open: "[", close: "]" },
                { open: "(", close: ")" },
                { open: '"', close: '"' },
                { open: "'", close: "'" },
                { open: "<!--", close: "-->" },
            ],
            surroundingPairs: [
                { open: '"', close: '"' },
                { open: "'", close: "'" },
                { open: "{", close: "}" },
                { open: "[", close: "]" },
                { open: "(", close: ")" },
                { open: "<", close: ">" },
            ],
        });

        // Add snippets for HXO
        monaco.languages.registerCompletionItemProvider("hxo", {
            provideCompletionItems: () => {
                const suggestions = [
                    {
                        label: "hxo-template",
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        insertText: [
                            "<template>",
                            "  ${1}",
                            "</template>",
                            "",
                            "<script>",
                            "import { ref, computed, watchEffect } from '@hxo/core';",
                            "",
                            "${2}",
                            "</script>",
                            "",
                            "<style scoped>",
                            "${3}",
                            "</style>",
                        ].join("\n"),
                        insertTextRules:
                            monaco.languages.CompletionItemInsertTextRule
                                .InsertAsSnippet,
                        documentation: "HXO Single File Component",
                    },
                    {
                        label: "import-core",
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        insertText:
                            "import { ref, computed, watchEffect } from '@hxo/core';",
                        insertTextRules:
                            monaco.languages.CompletionItemInsertTextRule
                                .InsertAsSnippet,
                        documentation: "Import HXO core reactivity functions",
                    },
                    {
                        label: "ref",
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        insertText: "let ${1:count} = ref(${2:0});",
                        insertTextRules:
                            monaco.languages.CompletionItemInsertTextRule
                                .InsertAsSnippet,
                        documentation: "Create a reactive ref",
                    },
                    {
                        label: "computed",
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        insertText:
                            "const ${1:double} = computed(() => ${2:count} * 2);",
                        insertTextRules:
                            monaco.languages.CompletionItemInsertTextRule
                                .InsertAsSnippet,
                        documentation: "Create a computed property",
                    },
                    {
                        label: "watchEffect",
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        insertText:
                            "watchEffect(() => {\n  console.log(${1:count});\n});",
                        insertTextRules:
                            monaco.languages.CompletionItemInsertTextRule
                                .InsertAsSnippet,
                        documentation: "Create a reactive effect",
                    },
                    {
                        label: "v-for",
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        insertText: 'v-for="${1:item} in ${2:items}"',
                        insertTextRules:
                            monaco.languages.CompletionItemInsertTextRule
                                .InsertAsSnippet,
                        documentation: "HXO for loop directive",
                    },
                ];
                return { suggestions };
            },
        });
    }

    const editor = monaco.editor.create(container, {
        value: initialValue,
        language: language === "html" ? "hxo" : language, // Use hxo for html (hxo files)
        theme: "vs-dark",
        automaticLayout: true,
        minimap: { enabled: false },
        fontSize: 14,
        fontFamily: "'Fira Code', monospace",
        lineNumbers: "on",
        roundedSelection: false,
        scrollBeyondLastLine: false,
        readOnly: false,
        cursorStyle: "line",
        bracketPairColorization: { enabled: true },
        guides: { bracketPairs: true },
        wordWrap: "on",
        padding: { top: 10, bottom: 10 },
    });

    editor.onDidChangeModelContent(() => {
        onChange(editor.getValue());
    });

    return editor;
}
