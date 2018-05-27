use dashmap::{DashMap, DashSet};
use hxo_ir::{ElementIR, ExpressionIR, JsExpr, JsProgram, JsStmt, TemplateNodeIR};
use hxo_parser::{ParseState, ScriptParser, TemplateParser};
use hxo_parser_expression::ExprParser;
use hxo_parser_template::TemplateParser;
use hxo_parser_toml::TomlParser;
use hxo_types::{Position as HxoPosition, Span as HxoSpan, is_pos_in_span};
use serde::Deserialize;
use tower_lsp::{Client, LanguageServer, LspService, Server, jsonrpc::Result, lsp_types::*};

use std::path::PathBuf;
use url::Url;

#[derive(Debug, Deserialize)]
struct HxoConfig {
    #[serde(default)]
    compiler: CompilerConfig,
}

#[derive(Debug, Deserialize, Default)]
struct CompilerConfig {
    #[serde(default)]
    auto_imports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CompletionContext {
    Tag,
    Attribute(String),
    Expression,
}

pub struct Backend {
    client: Client,
    documents: DashMap<String, String>,
    workspace_folders: DashMap<String, PathBuf>,
    auto_imports: DashSet<String>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let mut root_path = None;
        if let Some(folders) = params.workspace_folders {
            for folder in folders {
                if let Ok(path) = folder.uri.to_file_path() {
                    self.workspace_folders.insert(folder.uri.to_string(), path.clone());
                    if root_path.is_none() {
                        root_path = Some(path);
                    }
                }
            }
        }
        else if let Some(uri) = params.root_uri {
            if let Ok(path) = uri.to_file_path() {
                self.workspace_folders.insert(uri.to_string(), path.clone());
                root_path = Some(path);
            }
        }

        // Load hxo.config.toml
        if let Some(root) = root_path {
            let config_path = root.join("hxo.config.toml");
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(config_path) {
                    let parser = TomlParser::new();
                    if let Ok(config) = parser.parse_to_type::<HxoConfig>(&content) {
                        for import in config.compiler.auto_imports {
                            self.auto_imports.insert(import);
                        }
                    }
                }
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                definition_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["<".to_string(), "@".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "HXO Language Server initialized").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.documents.insert(params.text_document.uri.to_string(), params.text_document.text);
        self.validate_document(params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            self.documents.insert(params.text_document.uri.to_string(), change.text);
            self.validate_document(params.text_document.uri).await;
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        let mut items = Vec::new();

        if let Some(content) = self.documents.get(&uri) {
            let hxo_pos = HxoPosition { line: position.line + 1, column: position.character + 1, offset: 0 };

            let mut state = ParseState::new(&content);
            let parser = TemplateParser;
            if let Ok(nodes) = parser.parse(&mut state, "html") {
                if let Some(context) = Backend::get_completion_context(&nodes, hxo_pos) {
                    match context {
                        CompletionContext::Tag => {
                            // Basic HTML tags
                            let tags = vec!["div", "span", "button", "input", "h1", "h2", "p", "section", "article"];
                            for tag in tags {
                                items.push(CompletionItem {
                                    label: tag.to_string(),
                                    kind: Some(CompletionItemKind::KEYWORD),
                                    detail: Some("HTML Tag".to_string()),
                                    ..Default::default()
                                });
                            }
                        }
                        CompletionContext::Attribute(tag_name) => {
                            // Directives
                            let directives = vec!["@click", "@input", "@change", ":class", ":style", ":value"];
                            for dir in directives {
                                items.push(CompletionItem {
                                    label: dir.to_string(),
                                    kind: Some(CompletionItemKind::CONSTANT),
                                    detail: Some("Directive".to_string()),
                                    ..Default::default()
                                });
                            }

                            // Tag specific attributes
                            if tag_name == "input" {
                                items.push(CompletionItem {
                                    label: "type".to_string(),
                                    kind: Some(CompletionItemKind::PROPERTY),
                                    ..Default::default()
                                });
                            }
                        }
                        CompletionContext::Expression => {
                            // Auto-imports (Signals, etc.)
                            for import in self.auto_imports.iter() {
                                items.push(CompletionItem {
                                    label: import.clone(),
                                    kind: Some(CompletionItemKind::FUNCTION),
                                    detail: Some("HXO Auto-import".to_string()),
                                    ..Default::default()
                                });
                            }

                            // Script variables
                            if let Some(program) = Self::get_script_program(&nodes) {
                                for stmt in &program.body {
                                    if let JsStmt::VariableDecl { id, .. } = stmt {
                                        items.push(CompletionItem {
                                            label: id.clone(),
                                            kind: Some(CompletionItemKind::VARIABLE),
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if items.is_empty() {
            // Fallback to all items if no context found
            let tags = vec!["div", "span", "button", "input", "h1", "h2", "p", "section", "article"];
            for tag in tags {
                items.push(CompletionItem {
                    label: tag.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some("HTML Tag".to_string()),
                    ..Default::default()
                });
            }
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;

        if let Some(content) = self.documents.get(&uri) {
            let hxo_pos = HxoPosition { line: position.line + 1, column: position.character + 1, offset: 0 };

            let mut state = ParseState::new(&content);
            let parser = TemplateParser;
            if let Ok(nodes) = parser.parse(&mut state, "html") {
                return Ok(self.find_definition_in_nodes(&nodes, hxo_pos, &uri).await);
            }
        }

        Ok(None)
    }
}

impl Backend {
    async fn validate_document(&self, uri: Url) {
        let mut diagnostics = Vec::new();
        if let Some(content) = self.documents.get(&uri.to_string()) {
            let mut compiler = hxo_compiler::Compiler::new();
            let name = uri.path_segments().and_then(|mut s| s.next_back()).unwrap_or("App.hxo");

            if let Err(e) = compiler.compile(name, &content) {
                let span = e.span();
                let range = if span.is_unknown() {
                    Range { start: Position { line: 0, character: 0 }, end: Position { line: 0, character: 1 } }
                }
                else {
                    Range {
                        start: Position {
                            line: span.start.line.saturating_sub(1),
                            character: span.start.column.saturating_sub(1),
                        },
                        end: Position { line: span.end.line.saturating_sub(1), character: span.end.column.saturating_sub(1) },
                    }
                };

                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("{}", e),
                    source: Some("hxo-compiler".to_string()),
                    ..Default::default()
                });
            }
        }
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    fn get_completion_context(nodes: &[TemplateNodeIR], pos: HxoPosition) -> Option<CompletionContext> {
        for node in nodes {
            match node {
                TemplateNodeIR::Element(el) => {
                    if is_pos_in_span(pos, el.span) {
                        // Check if in tag name
                        let tag_start = el.span.start;
                        let tag_name_end = HxoPosition {
                            line: tag_start.line,
                            column: tag_start.column + (el.tag.len() as u32) + 1, // <tag
                            offset: 0,
                        };

                        if pos.line == tag_start.line && pos.column <= tag_name_end.column {
                            return Some(CompletionContext::Tag);
                        }

                        // Check attributes
                        for attr in &el.attributes {
                            if is_pos_in_span(pos, attr.span) {
                                return Some(CompletionContext::Attribute(el.tag.clone()));
                            }
                        }

                        // Check children
                        if let Some(ctx) = Self::get_completion_context(&el.children, pos) {
                            return Some(ctx);
                        }

                        // If in element but not in specific child/attr, might be in attribute area
                        return Some(CompletionContext::Attribute(el.tag.clone()));
                    }
                }
                TemplateNodeIR::Interpolation(expr) => {
                    if is_pos_in_span(pos, expr.span) {
                        return Some(CompletionContext::Expression);
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn new(client: Client) -> Self {
        let auto_imports = DashSet::new();
        // Add default auto-imports
        auto_imports.insert("createSignal".to_string());
        auto_imports.insert("useEffect".to_string());
        auto_imports.insert("createMemo".to_string());
        auto_imports.insert("createEffect".to_string());
        auto_imports.insert("onMount".to_string());
        auto_imports.insert("onCleanup".to_string());

        Self { client, documents: DashMap::new(), workspace_folders: DashMap::new(), auto_imports }
    }

    fn resolve_path(&self, current_uri: &str, relative_path: &str) -> Option<Url> {
        let base_url = Url::parse(current_uri).ok()?;
        let base_path = base_url.to_file_path().ok()?;

        if relative_path == "@hxo/core" {
            // 1. Try workspace folders
            for entry in self.workspace_folders.iter() {
                let folder = entry.value();
                let core_path = folder.join("runtimes/hxo-core/src/index.ts");
                if core_path.exists() {
                    return Url::from_file_path(core_path).ok();
                }
                // Try nested
                let core_path = folder.join("project-hxo/runtimes/hxo-core/src/index.ts");
                if core_path.exists() {
                    return Url::from_file_path(core_path).ok();
                }
            }

            // 2. Try relative to current file
            if let Ok(uri) = Url::parse(current_uri) {
                if let Ok(mut path) = uri.to_file_path() {
                    while let Some(parent) = path.parent() {
                        let core_path = parent.join("runtimes/hxo-core/src/index.ts");
                        if core_path.exists() {
                            return Url::from_file_path(core_path).ok();
                        }
                        path = parent.to_path_buf();
                    }
                }
            }
        }

        let parent = base_path.parent()?;

        let target_path = if relative_path.starts_with('.') {
            parent.join(relative_path)
        }
        else {
            // Handle aliases or node_modules here if needed
            return None;
        };

        // Try adding extensions if not present
        let extensions = ["", ".hxo", ".ts", ".js"];
        for ext in extensions {
            let mut p = target_path.clone();
            if !ext.is_empty() {
                p.set_extension(ext.trim_start_matches('.'));
            }
            if p.exists() {
                return Url::from_file_path(p).ok();
            }
        }

        None
    }

    async fn get_document_content(&self, uri: &Url) -> Option<String> {
        let uri_str = uri.to_string();
        if let Some(content) = self.documents.get(&uri_str) {
            return Some(content.clone());
        }

        // Read from disk if not in memory
        if let Ok(path) = uri.to_file_path() {
            return std::fs::read_to_string(path).ok();
        }

        None
    }

    async fn find_definition_in_nodes(
        &self,
        nodes: &[TemplateNodeIR],
        pos: HxoPosition,
        uri: &str,
    ) -> Option<GotoDefinitionResponse> {
        let script_program = Self::get_script_program(nodes);

        for node in nodes {
            match node {
                TemplateNodeIR::Element(el) => {
                    if is_pos_in_span(pos, el.span) {
                        if el.tag == "script" {
                            return self.find_definition_in_script(el, pos, uri).await;
                        }
                        else if el.tag == "style" {
                            return Self::find_definition_in_style(el, pos, uri);
                        }
                        else {
                            // 使用 Box::pin 处理异步递归
                            let res = self.find_definition_in_nodes_recursive(&el.children, pos, uri).await;
                            if res.is_some() {
                                return res;
                            }
                        }
                    }
                }
                TemplateNodeIR::Interpolation(expr) => {
                    if !expr.span.is_unknown() && is_pos_in_span(pos, expr.span) {
                        if let Some(symbol) = Self::find_symbol_in_template_expr(expr, pos) {
                            // 1. 首先在 script 块中查找
                            if let Some(program) = &script_program {
                                if let Some(def_span) = Self::find_definition_of_symbol(program, &symbol) {
                                    if Self::is_import(program, &symbol) {
                                        if let Some(loc) = self.find_external_definition(program, &symbol, uri).await {
                                            return Some(GotoDefinitionResponse::Scalar(loc));
                                        }
                                    }

                                    if let Some(script_el_span) = Self::get_script_element_span(nodes) {
                                        return Some(GotoDefinitionResponse::Scalar(Location {
                                            uri: Url::parse(uri).unwrap(),
                                            range: Range {
                                                start: Position {
                                                    line: script_el_span.start.line + def_span.start.line - 1,
                                                    character: def_span.start.column - 1,
                                                },
                                                end: Position {
                                                    line: script_el_span.start.line + def_span.end.line - 1,
                                                    character: def_span.end.column - 1,
                                                },
                                            },
                                        }));
                                    }
                                }
                            }

                            // 2. 检查是否为自动导入
                            if self.auto_imports.contains(&symbol) {
                                if let Some(loc) = self.find_implicit_definition(&symbol, "@hxo/core", uri).await {
                                    return Some(GotoDefinitionResponse::Scalar(loc));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    // Helper for recursion to avoid "async recursion" error
    fn find_definition_in_nodes_recursive<'a>(
        &'a self,
        nodes: &'a [TemplateNodeIR],
        pos: HxoPosition,
        uri: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<GotoDefinitionResponse>> + Send + 'a>> {
        Box::pin(self.find_definition_in_nodes(nodes, pos, uri))
    }

    fn is_import(program: &JsProgram, symbol: &str) -> bool {
        for stmt in &program.body {
            if let JsStmt::Import { specifiers, .. } = stmt {
                if specifiers.contains(&symbol.to_string()) {
                    return true;
                }
            }
        }
        false
    }

    fn get_script_program(nodes: &[TemplateNodeIR]) -> Option<JsProgram> {
        for node in nodes {
            if let TemplateNodeIR::Element(el) = node {
                if el.tag == "script" {
                    if let Some(TemplateNodeIR::Text(content, _)) = el.children.first() {
                        let mut state = ParseState::new(content);
                        let ts_parser = ExprParser;
                        return ts_parser.parse(&mut state, "ts").ok();
                    }
                }
                if let Some(p) = Self::get_script_program(&el.children) {
                    return Some(p);
                }
            }
        }
        None
    }

    fn get_script_element_span(nodes: &[TemplateNodeIR]) -> Option<HxoSpan> {
        for node in nodes {
            if let TemplateNodeIR::Element(el) = node {
                if el.tag == "script" {
                    return Some(el.span);
                }
                if let Some(s) = Self::get_script_element_span(&el.children) {
                    return Some(s);
                }
            }
        }
        None
    }

    fn find_symbol_in_template_expr(expr: &ExpressionIR, pos: HxoPosition) -> Option<String> {
        // In hxo-ir, ExpressionIR has a 'code' field for the raw string
        if !expr.span.is_unknown() && is_pos_in_span(pos, expr.span) {
            // If it's a simple identifier expression
            return Some(expr.code.clone());
        }
        None
    }

    async fn find_definition_in_script(&self, el: &ElementIR, pos: HxoPosition, uri: &str) -> Option<GotoDefinitionResponse> {
        if let Some(TemplateNodeIR::Text(content, _)) = el.children.first() {
            let mut state = ParseState::new(content);
            let ts_parser = ExprParser;
            if let Ok(program) = ts_parser.parse(&mut state, "ts") {
                // Adjust position relative to the script tag start
                let el_span = el.span;
                let relative_pos = HxoPosition {
                    line: pos.line - el_span.start.line,
                    column: if pos.line == el_span.start.line { pos.column - el_span.start.column } else { pos.column },
                    offset: 0,
                };

                if let Some(symbol) = Self::find_symbol_at(&program, relative_pos) {
                    // 1. Check if it's an import (external definition)
                    if Self::is_import(&program, &symbol) {
                        if let Some(loc) = self.find_external_definition(&program, &symbol, uri).await {
                            return Some(GotoDefinitionResponse::Scalar(loc));
                        }
                    }

                    // 2. Find where this symbol is defined locally
                    if let Some(def_span) = Self::find_definition_of_symbol(&program, &symbol) {
                        return Some(GotoDefinitionResponse::Scalar(Location {
                            uri: Url::parse(uri).unwrap(),
                            range: Range {
                                start: Position {
                                    line: el_span.start.line + def_span.start.line - 1,
                                    character: def_span.start.column - 1,
                                },
                                end: Position {
                                    line: el_span.start.line + def_span.end.line - 1,
                                    character: def_span.end.column - 1,
                                },
                            },
                        }));
                    }

                    // 3. Check if it's an auto-import
                    if self.auto_imports.contains(&symbol) {
                        if let Some(loc) = self.find_implicit_definition(&symbol, "@hxo/core", uri).await {
                            return Some(GotoDefinitionResponse::Scalar(loc));
                        }

                        // Fallback to script tag start
                        return Some(GotoDefinitionResponse::Scalar(Location {
                            uri: Url::parse(uri).unwrap(),
                            range: Range {
                                start: Position { line: el_span.start.line - 1, character: el_span.start.column - 1 },
                                end: Position {
                                    line: el_span.start.line - 1,
                                    character: el_span.start.column + 6, // Just the "<script" part
                                },
                            },
                        }));
                    }
                }
            }
        }
        None
    }

    fn find_symbol_at(program: &JsProgram, pos: HxoPosition) -> Option<String> {
        for stmt in &program.body {
            if let Some(symbol) = Self::find_symbol_in_stmt(stmt, pos) {
                return Some(symbol);
            }
        }
        None
    }

    fn find_symbol_in_stmt(stmt: &JsStmt, pos: HxoPosition) -> Option<String> {
        use JsStmt::*;
        match stmt {
            Expr(expr, _) => Self::find_symbol_in_expr(expr, pos),
            VariableDecl { id, init, span, .. } => {
                if is_pos_in_span(pos, *span) {
                    return Some(id.clone());
                }
                if let Some(init_expr) = init {
                    return Self::find_symbol_in_expr(init_expr, pos);
                }
                None
            }
            FunctionDecl { id, body, span, .. } => {
                if is_pos_in_span(pos, *span) {
                    // Check if pos is on the function name
                    return Some(id.clone());
                }
                for s in body {
                    if let Some(symbol) = Self::find_symbol_in_stmt(s, pos) {
                        return Some(symbol);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn find_symbol_in_expr(expr: &JsExpr, pos: HxoPosition) -> Option<String> {
        use JsExpr::*;
        match expr {
            Identifier(name, span) => {
                if is_pos_in_span(pos, *span) {
                    return Some(name.clone());
                }
            }
            Binary { left, right, span, .. } => {
                if is_pos_in_span(pos, *span) {
                    return Self::find_symbol_in_expr(left, pos).or_else(|| Self::find_symbol_in_expr(right, pos));
                }
            }
            Call { callee, args, span, .. } => {
                if is_pos_in_span(pos, *span) {
                    if let Some(symbol) = Self::find_symbol_in_expr(callee, pos) {
                        return Some(symbol);
                    }
                    for arg in args {
                        if let Some(symbol) = Self::find_symbol_in_expr(arg, pos) {
                            return Some(symbol);
                        }
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn find_definition_of_symbol(program: &JsProgram, symbol: &str) -> Option<HxoSpan> {
        for stmt in &program.body {
            use JsStmt::*;
            match stmt {
                VariableDecl { id, span, .. } if id == symbol => return Some(*span),
                FunctionDecl { id, span, .. } if id == symbol => return Some(*span),
                Import { specifiers, span, .. } if specifiers.contains(&symbol.to_string()) => return Some(*span),
                Export { declaration, .. } => match &**declaration {
                    VariableDecl { id, span, .. } if id == symbol => return Some(*span),
                    FunctionDecl { id, span, .. } if id == symbol => return Some(*span),
                    _ => {}
                },
                ExportNamed { source: None, specifiers, span, .. } if specifiers.contains(&symbol.to_string()) => {
                    // This is "export { x }". We should ideally find where x is defined.
                    // But for now, returning the export statement span is better than nothing.
                    return Some(*span);
                }
                _ => {}
            }
        }
        None
    }

    async fn find_external_definition(&self, program: &JsProgram, symbol: &str, current_uri: &str) -> Option<Location> {
        for stmt in &program.body {
            if let JsStmt::Import { source, specifiers, .. } = stmt {
                if specifiers.contains(&symbol.to_string()) {
                    if let Some(target_uri) = self.resolve_path(current_uri, source) {
                        return self.find_external_definition_in_file(target_uri, symbol).await;
                    }
                }
            }
        }
        None
    }

    async fn find_implicit_definition(&self, symbol: &str, source: &str, current_uri: &str) -> Option<Location> {
        if let Some(target_uri) = self.resolve_path(current_uri, source) {
            return self.find_external_definition_in_file(target_uri, symbol).await;
        }
        None
    }

    fn find_external_definition_in_file<'a>(
        &'a self,
        uri: Url,
        symbol: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<Location>> + Send + 'a>> {
        let symbol_owned = symbol.to_string();
        Box::pin(async move {
            let content = self.get_document_content(&uri).await?;
            let symbol = &symbol_owned;
            if uri.path().ends_with(".hxo") {
                let mut state = ParseState::new(&content);
                let parser = TemplateParser;
                if let Ok(nodes) = parser.parse(&mut state, "html") {
                    if let Some(ext_program) = Self::get_script_program(&nodes) {
                        if let Some(def_span) = Self::find_exported_definition(&ext_program, symbol) {
                            if let Some(script_el_span) = Self::get_script_element_span(&nodes) {
                                return Some(Location {
                                    uri,
                                    range: Range {
                                        start: Position {
                                            line: script_el_span.start.line + def_span.start.line - 1,
                                            character: def_span.start.column - 1,
                                        },
                                        end: Position {
                                            line: script_el_span.start.line + def_span.end.line - 1,
                                            character: def_span.end.column - 1,
                                        },
                                    },
                                });
                            }
                        }
                    }
                }
            }
            else {
                let mut state = ParseState::new(&content);
                let ts_parser = ExprParser;
                if let Ok(ext_program) = ts_parser.parse(&mut state, "ts") {
                    if let Some(def_span) = Self::find_exported_definition(&ext_program, symbol) {
                        return Some(Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position { line: def_span.start.line - 1, character: def_span.start.column - 1 },
                                end: Position { line: def_span.end.line - 1, character: def_span.end.column - 1 },
                            },
                        });
                    }

                    // Handle re-exports
                    for stmt in &ext_program.body {
                        match stmt {
                            JsStmt::ExportAll { source, .. } => {
                                if let Some(target_uri) = self.resolve_path(uri.as_str(), source) {
                                    if let Some(loc) = self.find_external_definition_in_file(target_uri, symbol).await {
                                        return Some(loc);
                                    }
                                }
                            }
                            JsStmt::ExportNamed { source: Some(source), specifiers, .. } => {
                                if specifiers.contains(&symbol.to_string()) {
                                    if let Some(target_uri) = self.resolve_path(uri.as_str(), source) {
                                        if let Some(loc) = self.find_external_definition_in_file(target_uri, symbol).await {
                                            return Some(loc);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            None
        })
    }

    fn find_exported_definition(program: &JsProgram, symbol: &str) -> Option<HxoSpan> {
        for stmt in &program.body {
            match stmt {
                JsStmt::Export { declaration, .. } => match &**declaration {
                    JsStmt::VariableDecl { id, span, .. } if id == symbol => return Some(*span),
                    JsStmt::FunctionDecl { id, span, .. } if id == symbol => return Some(*span),
                    _ => {}
                },
                JsStmt::ExportNamed { source: None, specifiers, span, .. } if specifiers.contains(&symbol.to_string()) => {
                    return Some(*span);
                }
                _ => {}
            }
        }
        None
    }

    fn find_definition_in_style(_el: &ElementIR, _pos: HxoPosition, _uri: &str) -> Option<GotoDefinitionResponse> {
        // Similar to script, use hxo-parser-css
        None
    }
}

pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
