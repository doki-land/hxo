use hxo_ir::{IRModule, JsExpr, JsStmt, TemplateNodeIR};
use hxo_source_map::{SourceMap, SourceMapBuilder};
use hxo_types::{CodeWriter, Position, Result, Span};
use std::collections::HashSet;

#[derive(Clone)]
pub struct JsWriter {
    inner: CodeWriter,
}

impl JsWriter {
    pub fn new() -> Self {
        Self { inner: CodeWriter::new() }
    }

    pub fn write(&mut self, text: &str) {
        self.inner.write(text);
    }

    pub fn write_with_span(&mut self, text: &str, span: Span) {
        self.inner.write_with_span(text, span);
    }

    pub fn write_line(&mut self, text: &str) {
        self.inner.write_line(text);
    }

    pub fn write_line_with_span(&mut self, text: &str, span: Span) {
        self.inner.write_line_with_span(text, span);
    }

    pub fn newline(&mut self) {
        self.inner.newline();
    }

    pub fn indent(&mut self) {
        self.inner.indent();
    }

    pub fn dedent(&mut self) {
        self.inner.dedent();
    }

    pub fn append(&mut self, other: JsWriter) {
        self.inner.append(other.inner);
    }

    pub fn write_block<F>(&mut self, open: &str, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.inner.write(open);
        if !open.ends_with(' ') && !open.is_empty() {
            self.inner.write(" ");
        }
        self.inner.write("{");
        self.inner.newline();
        self.inner.indent();
        f(self);
        self.inner.dedent();
        self.inner.write_line("}");
    }

    /// 写入箭头函数: (params) => { body } 或 (params) => expr
    pub fn write_arrow_function<F>(&mut self, params: &[&str], is_block: bool, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.inner.write("(");
        self.inner.write(&params.join(", "));
        self.inner.write(") => ");
        if is_block {
            self.write_block("", f);
        }
        else {
            f(self);
        }
    }

    /// 写入解构赋值: const { a, b } = obj;
    pub fn write_destructuring(&mut self, kind: &str, properties: &[&str], source: &str) {
        self.inner.write(kind);
        self.inner.write(" { ");
        self.inner.write(&properties.join(", "));
        self.inner.write(" } = ");
        self.inner.write(source);
        self.inner.write_line(";");
    }

    /// 写入导入语句: import { a, b } from 'source';
    pub fn write_import(&mut self, specifiers: &[&str], source: &str) {
        if specifiers.is_empty() {
            self.inner.write_line(&format!("import '{}';", source));
        }
        else {
            self.inner.write_line(&format!("import {{ {} }} from '{}';", specifiers.join(", "), source));
        }
    }

    pub fn finish(self) -> (String, Vec<(Position, Span)>) {
        self.inner.finish()
    }
}

pub struct JsBackend {
    pub minify: bool,
    pub is_prod: bool,
    pub target: Option<String>,
    pub runtime_path: String,
}

impl JsBackend {
    pub fn new(minify: bool, is_prod: bool, target: Option<String>) -> Self {
        Self { minify, is_prod, target, runtime_path: "@hxo".to_string() }
    }

    pub fn generate(&self, ir: &IRModule) -> Result<(String, SourceMap)> {
        let mut writer = JsWriter::new();
        let mut used_core = HashSet::new();
        let mut used_dom = HashSet::new();

        // 1. Generate Component Body First to track used features
        let mut body_writer = JsWriter::new();
        self.generate_component_body(ir, &mut body_writer, &mut used_core, &mut used_dom)?;

        // 2. Generate Imports based on used features
        if !used_core.is_empty() {
            let mut imports: Vec<_> = used_core.into_iter().collect();
            imports.sort();
            let specifiers: Vec<&str> = imports.iter().map(|s| s.as_str()).collect();
            writer.write_import(&specifiers, &format!("{}/core", self.runtime_path));
        }
        if !used_dom.is_empty() {
            let mut imports: Vec<_> = used_dom.into_iter().collect();
            imports.sort();
            let specifiers: Vec<&str> = imports.iter().map(|s| s.as_str()).collect();
            writer.write_import(&specifiers, &format!("{}/dom", self.runtime_path));
        }
        writer.newline();

        // 3. Append Body
        writer.append(body_writer);

        let (code, mappings) = writer.finish();
        let mut builder = SourceMapBuilder::new();
        builder.add_from_writer(&mappings, Some(ir.name.clone()));

        Ok((code, builder.finish()))
    }

    fn generate_component_body(
        &self,
        ir: &IRModule,
        writer: &mut JsWriter,
        used_core: &mut HashSet<String>,
        used_dom: &mut HashSet<String>,
    ) -> Result<()> {
        let mut hoisted_nodes: Vec<(String, JsWriter)> = Vec::new();
        let mut render_writer = JsWriter::new();

        // Render Function
        render_writer.write_block("render(ctx)", |writer| {
            writer.write("return ");
            if let Some(template) = &ir.template {
                if template.nodes.is_empty() {
                    writer.write("null");
                }
                else if template.nodes.len() == 1 {
                    self.generate_node_with_hoisting(&template.nodes[0], writer, ir, used_core, used_dom, &mut hoisted_nodes);
                }
                else {
                    used_dom.insert("h".to_string());
                    used_core.insert("Fragment".to_string());
                    writer.write_line("h(Fragment, null, [");
                    writer.indent();
                    for node in &template.nodes {
                        self.generate_node_with_hoisting(node, writer, ir, used_core, used_dom, &mut hoisted_nodes);
                        writer.write_line(",");
                    }
                    writer.dedent();
                    writer.write("])");
                }
            }
            else {
                writer.write("null");
            }
            writer.write_line(";");
        });

        // Generate hoisted nodes first
        let has_hoisted = !hoisted_nodes.is_empty();
        for (name, hoisted_writer) in hoisted_nodes {
            writer.write("const ");
            writer.write(&name);
            writer.write(" = /*#__PURE__*/ ");
            writer.append(hoisted_writer);
            writer.write_line(";");
        }

        if has_hoisted {
            writer.newline();
        }

        // Component Definition
        writer.write_block("export default", |writer| {
            writer.write_line(&format!("name: '{}',", ir.name));
            writer.newline();

            // i18n Data
            if let Some(i18n) = &ir.i18n {
                writer.write("i18n: ");
                let mut i18n_writer = JsWriter::new();
                let i18n_val = hxo_types::HxoValue::Object(
                    i18n.iter()
                        .map(|(lang, msgs)| {
                            (
                                lang.clone(),
                                hxo_types::HxoValue::Object(
                                    msgs.iter().map(|(k, v)| (k.clone(), hxo_types::HxoValue::String(v.clone()))).collect(),
                                ),
                            )
                        })
                        .collect(),
                );
                self.generate_expr(
                    &hxo_ir::JsExpr::Literal(i18n_val, hxo_types::Span::unknown()),
                    &mut i18n_writer,
                    ir,
                    used_core,
                    used_dom,
                );
                writer.append(i18n_writer);
                writer.write_line(",");
                writer.newline();
            }

            // Setup Function
            writer.write_block("setup(props, { i18n })", |writer| {
                if ir.i18n.is_some() {
                    used_core.insert("useI18n".to_string());
                    writer.write_line("const { t: $t } = useI18n(i18n);");
                }
                if let Some(script) = &ir.script {
                    for stmt in &script.body {
                        self.generate_stmt(stmt, writer, ir, used_core, used_dom);
                    }

                    // Collect all identifiers to return them
                    let mut returned_ids = Vec::new();
                    for stmt in &script.body {
                        match stmt {
                            JsStmt::VariableDecl { id, .. } => {
                                if id.starts_with('[') || id.starts_with('{') {
                                    let cleaned = id.trim_matches(|c| c == '[' || c == ']' || c == '{' || c == '}');
                                    for part in cleaned.split(',') {
                                        let part = part.trim();
                                        if !part.is_empty() {
                                            if let Some(colon_idx) = part.find(':') {
                                                returned_ids.push(part[colon_idx + 1..].trim().to_string());
                                            }
                                            else {
                                                returned_ids.push(part.to_string());
                                            }
                                        }
                                    }
                                }
                                else {
                                    returned_ids.push(id.clone());
                                }
                            }
                            JsStmt::FunctionDecl { id, .. } => returned_ids.push(id.clone()),
                            _ => {}
                        }
                    }

                    if ir.i18n.is_some() {
                        returned_ids.push("$t".to_string());
                    }

                    writer.write("return { ");
                    for (i, id) in returned_ids.iter().enumerate() {
                        if i > 0 {
                            writer.write(", ");
                        }
                        writer.write(id);
                    }
                    writer.write_line(" };");
                }
                else {
                    writer.write_line("return {};");
                }
            });
            writer.write_line(",");
            writer.newline();

            writer.append(render_writer);
        });
        writer.write_line(";");
        Ok(())
    }

    fn generate_node_with_hoisting(
        &self,
        node: &TemplateNodeIR,
        writer: &mut JsWriter,
        ir: &IRModule,
        used_core: &mut HashSet<String>,
        used_dom: &mut HashSet<String>,
        hoisted_nodes: &mut Vec<(String, JsWriter)>,
    ) {
        if self.is_static_node(node) {
            let mut node_writer = JsWriter::new();
            self.generate_node(node, &mut node_writer, ir, used_core, used_dom, hoisted_nodes);
            let name = format!("_hoisted_{}", hoisted_nodes.len() + 1);
            hoisted_nodes.push((name.clone(), node_writer));
            writer.write(&name);
        }
        else {
            self.generate_node(node, writer, ir, used_core, used_dom, hoisted_nodes);
        }
    }

    fn is_static_node(&self, node: &TemplateNodeIR) -> bool {
        match node {
            TemplateNodeIR::Element(el) => el.is_static,
            TemplateNodeIR::Text(_, _) => true,
            TemplateNodeIR::Interpolation(_) => false,
            TemplateNodeIR::Comment(_, _) => true,
        }
    }

    fn generate_stmt(
        &self,
        stmt: &JsStmt,
        writer: &mut JsWriter,
        ir: &IRModule,
        used_core: &mut HashSet<String>,
        used_dom: &mut HashSet<String>,
    ) {
        match stmt {
            JsStmt::Expr(expr, _) => {
                self.generate_expr(expr, writer, ir, used_core, used_dom);
                writer.write_line(";");
            }
            JsStmt::VariableDecl { kind, id, init, .. } => {
                writer.write(&format!("{} {} ", kind, id));
                if let Some(init) = init {
                    writer.write("= ");
                    self.generate_expr(init, writer, ir, used_core, used_dom);
                }
                writer.write_line(";");
            }
            JsStmt::FunctionDecl { id, params, body, .. } => {
                writer.write_block(&format!("function {}({})", id, params.join(", ")), |writer| {
                    for stmt in body {
                        self.generate_stmt(stmt, writer, ir, used_core, used_dom);
                    }
                });
            }
            JsStmt::Import { source, specifiers, .. } => {
                let specs: Vec<&str> = specifiers.iter().map(|s| s.as_str()).collect();
                writer.write_import(&specs, source);
            }
            JsStmt::Export { declaration, .. } => {
                writer.write("export ");
                self.generate_stmt(declaration, writer, ir, used_core, used_dom);
            }
            JsStmt::ExportAll { source, .. } => {
                writer.write_line(&format!("export * from '{}';", source));
            }
            JsStmt::ExportNamed { source, specifiers, .. } => {
                writer.write(&format!("export {{ {} }}", specifiers.join(", ")));
                if let Some(source) = source {
                    writer.write(&format!(" from '{}'", source));
                }
                writer.write_line(";");
            }
            JsStmt::Other(code, _) => {
                writer.write_line(code);
            }
        }
    }

    fn generate_expr(
        &self,
        expr: &JsExpr,
        writer: &mut JsWriter,
        ir: &IRModule,
        used_core: &mut HashSet<String>,
        used_dom: &mut HashSet<String>,
    ) {
        match expr {
            JsExpr::Identifier(id, span) => writer.write_with_span(id, *span),
            JsExpr::Literal(val, span) => match val {
                hxo_types::HxoValue::String(s) => writer.write_with_span(&format!("'{}'", s), *span),
                hxo_types::HxoValue::Number(n) => writer.write_with_span(&n.to_string(), *span),
                hxo_types::HxoValue::Bool(b) => writer.write_with_span(&b.to_string(), *span),
                hxo_types::HxoValue::Null => writer.write_with_span("null", *span),
                hxo_types::HxoValue::Signal(s) => {
                    used_core.insert("useSignal".to_string());
                    writer.write_with_span(&format!("useSignal('{}')", s), *span);
                }
                hxo_types::HxoValue::Raw(code) => writer.write_with_span(code, *span),
                hxo_types::HxoValue::Ref(id) => writer.write_with_span(&format!("_refs.{}", id), *span),
                hxo_types::HxoValue::Binary(data) => {
                    writer.write_with_span(
                        &format!("new Uint8Array([{}])", data.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(", ")),
                        *span,
                    );
                }
                hxo_types::HxoValue::Array(arr) => {
                    writer.write_with_span("[", *span);
                    for (i, v) in arr.iter().enumerate() {
                        if i > 0 {
                            writer.write(", ");
                        }
                        // Recursive literal handling for simple arrays
                        let expr = JsExpr::Literal(v.clone(), Span::unknown());
                        self.generate_expr(&expr, writer, ir, used_core, used_dom);
                    }
                    writer.write("]");
                }
                hxo_types::HxoValue::Object(obj) => {
                    writer.write_with_span("{ ", *span);
                    for (i, (k, v)) in obj.iter().enumerate() {
                        if i > 0 {
                            writer.write(", ");
                        }
                        writer.write(&format!("{}: ", k));
                        let expr = JsExpr::Literal(v.clone(), Span::unknown());
                        self.generate_expr(&expr, writer, ir, used_core, used_dom);
                    }
                    writer.write(" }");
                }
            },
            JsExpr::Binary { left, op, right, span } => {
                self.generate_expr(left, writer, ir, used_core, used_dom);
                writer.write_with_span(&format!(" {} ", op), *span);
                self.generate_expr(right, writer, ir, used_core, used_dom);
            }
            JsExpr::Call { callee, args, span } => {
                if let JsExpr::Identifier(id, _) = &**callee {
                    if id == "createSignal" || id == "createComputed" {
                        used_core.insert(id.clone());
                    }

                    // Check if it's a pure function call
                    let is_pure = if let Some(hxo_types::HxoValue::Object(meta)) = &ir.script_meta {
                        meta.get("pure_functions")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().any(|val| val.as_str() == Some(id)))
                            .unwrap_or(false)
                    }
                    else {
                        false
                    };

                    if is_pure {
                        writer.write("/*#__PURE__*/ ");
                    }
                }
                self.generate_expr(callee, writer, ir, used_core, used_dom);
                writer.write_with_span("(", *span);
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    self.generate_expr(arg, writer, ir, used_core, used_dom);
                }
                writer.write(")");
            }
            JsExpr::Member { object, property, computed, span } => {
                self.generate_expr(object, writer, ir, used_core, used_dom);
                if *computed {
                    writer.write_with_span(&format!("[{}]", property), *span);
                }
                else {
                    writer.write_with_span(&format!(".{}", property), *span);
                }
            }
            JsExpr::Array(elements, span) => {
                writer.write_with_span("[", *span);
                for (i, el) in elements.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    self.generate_expr(el, writer, ir, used_core, used_dom);
                }
                writer.write("]");
            }
            JsExpr::Object(props, span) => {
                writer.write_with_span("{ ", *span);
                for (i, (key, val)) in props.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    writer.write(&format!("{}: ", key));
                    self.generate_expr(val, writer, ir, used_core, used_dom);
                }
                writer.write(" }");
            }
            JsExpr::ArrowFunction { params, body, span } => {
                writer.write_with_span(&format!("({}) => ", params.join(", ")), *span);
                self.generate_expr(body, writer, ir, used_core, used_dom);
            }
            JsExpr::TseElement { tag, attributes, children, span } => {
                used_dom.insert("h".to_string());
                writer.write_with_span(&format!("h('{}', {{", tag), *span);
                for (i, attr) in attributes.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    writer.write_with_span(&format!("'{}': ", attr.name), attr.span);
                    if let Some(val) = &attr.value {
                        self.generate_expr(val, writer, ir, used_core, used_dom);
                    }
                    else {
                        writer.write("true");
                    }
                }
                writer.write("}, [");
                for (i, child) in children.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    self.generate_expr(child, writer, ir, used_core, used_dom);
                }
                writer.write("])");
            }
            JsExpr::Other(code, span) => writer.write_with_span(code, *span),
        }
    }

    fn generate_node(
        &self,
        node: &TemplateNodeIR,
        writer: &mut JsWriter,
        ir: &IRModule,
        used_core: &mut HashSet<String>,
        used_dom: &mut HashSet<String>,
        hoisted_nodes: &mut Vec<(String, JsWriter)>,
    ) {
        match node {
            TemplateNodeIR::Element(el) => {
                used_dom.insert("h".to_string());
                writer.write_with_span(&format!("h('{}', {{ ", el.tag), el.span);
                for (i, attr) in el.attributes.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    if attr.name.starts_with('@') || attr.name.starts_with(':') {
                        let value = attr.value.as_deref().unwrap_or("");
                        let name = if attr.name.starts_with('@') {
                            let event = &attr.name[1..];
                            let mut c = event.chars();
                            match c.next() {
                                None => "on".to_string(),
                                Some(f) => format!("on{}{}", f.to_uppercase(), c.as_str()),
                            }
                        }
                        else {
                            attr.name[1..].to_string()
                        };

                        // Check if it's a signal or function
                        let is_signal = if let Some(meta) = &ir.script_meta {
                            meta.get("signals")
                                .and_then(|s| s.as_array())
                                .map(|a| a.iter().any(|v| v.as_str() == Some(value)))
                                .unwrap_or(false)
                        }
                        else {
                            false
                        };

                        let is_computed = if let Some(meta) = &ir.script_meta {
                            meta.get("computed")
                                .and_then(|s| s.as_array())
                                .map(|a| a.iter().any(|v| v.as_str() == Some(value)))
                                .unwrap_or(false)
                        }
                        else {
                            false
                        };

                        if is_signal || is_computed {
                            writer.write_with_span(&format!("'{}': ctx.{}()", name, value), attr.span);
                        }
                        else {
                            writer.write_with_span(&format!("'{}': ctx.{}", name, value), attr.span);
                        }
                    }
                    else {
                        match &attr.value {
                            Some(v) => writer.write_with_span(&format!("'{}': '{}'", attr.name, v), attr.span),
                            None => writer.write_with_span(&format!("'{}': true", attr.name), attr.span),
                        }
                    }
                }
                writer.write(" }");

                if el.children.is_empty() {
                    writer.write(")");
                }
                else {
                    writer.write_line(", [");
                    writer.indent();
                    for child in &el.children {
                        self.generate_node_with_hoisting(child, writer, ir, used_core, used_dom, hoisted_nodes);
                        writer.write_line(",");
                    }
                    writer.dedent();
                    writer.write("])");
                }
            }
            TemplateNodeIR::Text(t, span) => {
                used_dom.insert("createTextVNode".to_string());
                writer.write_with_span(&format!("createTextVNode('{}')", t.trim()), *span);
            }
            TemplateNodeIR::Interpolation(exp) => {
                used_dom.insert("createTextVNode".to_string());

                writer.write_with_span("createTextVNode(", exp.span);
                if let Some(ast) = &exp.ast {
                    self.generate_expr(ast, writer, ir, used_core, used_dom);
                }
                else {
                    // Fallback to code string
                    // Check if it's a signal
                    let is_signal = if let Some(meta) = &ir.script_meta {
                        meta.get("signals")
                            .and_then(|s| s.as_array())
                            .map(|a| a.iter().any(|v| v.as_str() == Some(&exp.code)))
                            .unwrap_or(false)
                    }
                    else {
                        false
                    };

                    let is_computed = if let Some(meta) = &ir.script_meta {
                        meta.get("computed")
                            .and_then(|s| s.as_array())
                            .map(|a| a.iter().any(|v| v.as_str() == Some(&exp.code)))
                            .unwrap_or(false)
                    }
                    else {
                        false
                    };

                    if is_signal || is_computed {
                        writer.write(&format!("ctx.{}()", exp.code));
                    }
                    else {
                        writer.write(&format!("ctx.{}", exp.code));
                    }
                }
                writer.write(")");
            }
            TemplateNodeIR::Comment(c, span) => {
                writer.write_with_span(&format!("// {}", c), *span);
            }
        }
    }
}
