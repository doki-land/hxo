use hxo_ir::{AttributeIR, IRModule, JsExpr, JsStmt, TemplateNodeIR};
use hxo_parser_tailwind::StyleEngine;
use hxo_types::{HxoValue, Result};
use std::collections::HashMap;

pub struct Optimizer {
    pub style_engine: StyleEngine,
}

impl Optimizer {
    pub fn new() -> Self {
        Self { style_engine: StyleEngine::new() }
    }

    pub fn default() -> Self {
        Self::new()
    }

    pub fn optimize(&mut self, ir: &mut IRModule, locale: Option<&str>, is_prod: bool) {
        // 1. Static analysis
        if let Some(template) = &mut ir.template {
            for node in &mut template.nodes {
                self.optimize_node(node);
            }
        }

        // 2. i18n optimization
        if let Some(_locale_val) = locale {
            // Load messages for locale
            let messages = HashMap::new(); // In real case, load from locale files
            self.optimize_i18n(ir, &messages);
        }

        // 3. Call count tracking for inlining decisions
        let mut call_counts = HashMap::new();
        if let Some(script) = &ir.script {
            self.track_script_calls(script, &mut call_counts);
        }
        if let Some(template) = &ir.template {
            for node in &template.nodes {
                self.track_node_calls(node, &mut call_counts);
            }
        }

        if is_prod {
            // Additional production optimizations using call_counts etc.
        }
    }

    pub fn process_styles(&mut self, ir: &IRModule) -> Result<()> {
        if let Some(template) = &ir.template {
            self.collect_styles_from_nodes(&template.nodes)?;
        }
        Ok(())
    }

    pub fn apply_scope_id(&mut self, ir: &mut IRModule, scope_id: &str) {
        if let Some(template) = &mut ir.template {
            self.apply_scope_id_to_nodes(&mut template.nodes, scope_id);
        }

        // Also transform scoped styles
        for style in &mut ir.styles {
            if style.scoped {
                style.code = self.transform_scoped_css(&style.code, scope_id);
            }
        }
    }

    pub fn generate_scope_id(&self, name: &str) -> String {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        format!("data-h-{:x}", hash)
    }

    pub fn get_css(&self) -> String {
        self.style_engine.generate_css()
    }

    fn collect_styles_from_nodes(&mut self, nodes: &[TemplateNodeIR]) -> Result<()> {
        for node in nodes {
            match node {
                TemplateNodeIR::Element(el) => {
                    for attr in &el.attributes {
                        if attr.name == "class" {
                            if let Some(value) = &attr.value {
                                self.style_engine.parse_classes(value, attr.span)?;
                            }
                        }
                    }
                    self.collect_styles_from_nodes(&el.children)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn apply_scope_id_to_nodes(&self, nodes: &mut [TemplateNodeIR], scope_id: &str) {
        for node in nodes {
            if let TemplateNodeIR::Element(el) = node {
                el.attributes.push(AttributeIR {
                    name: scope_id.to_string(),
                    value: None,
                    value_ast: None,
                    is_directive: false,
                    is_dynamic: false,
                    span: hxo_types::Span::unknown(),
                });
                self.apply_scope_id_to_nodes(&mut el.children, scope_id);
            }
        }
    }

    fn transform_scoped_css(&self, css: &str, scope_id: &str) -> String {
        // Simple transformation: append [data-h-xxxx] to each selector
        let mut result = String::new();
        for line in css.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.contains('{') && !line.starts_with('@') {
                let parts: Vec<&str> = line.split('{').collect();
                let selectors: Vec<&str> = parts[0].split(',').collect();
                let scoped_selectors: Vec<String> = selectors.iter().map(|s| format!("{}[{}]", s.trim(), scope_id)).collect();
                result.push_str(&scoped_selectors.join(", "));
                result.push_str(" {");
                if parts.len() > 1 {
                    result.push_str(parts[1]);
                }
                result.push('\n');
            }
            else {
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    }

    fn track_script_calls(&self, script: &hxo_ir::JsProgram, counts: &mut HashMap<String, usize>) {
        for stmt in &script.body {
            self.track_stmt_calls(stmt, counts);
        }
    }

    fn track_node_calls(&self, node: &TemplateNodeIR, counts: &mut HashMap<String, usize>) {
        match node {
            TemplateNodeIR::Element(el) => {
                for attr in &el.attributes {
                    if let Some(expr) = &attr.value_ast {
                        self.track_expr_calls(expr, counts);
                    }
                }
                for child in &el.children {
                    self.track_node_calls(child, counts);
                }
            }
            TemplateNodeIR::Interpolation(expr) => {
                if let Some(ast) = &expr.ast {
                    self.track_expr_calls(ast, counts);
                }
            }
            _ => {}
        }
    }

    fn track_stmt_calls(&self, stmt: &JsStmt, counts: &mut HashMap<String, usize>) {
        match stmt {
            JsStmt::Expr(expr, _) => self.track_expr_calls(expr, counts),
            JsStmt::VariableDecl { init, .. } => {
                if let Some(expr) = init {
                    self.track_expr_calls(expr, counts);
                }
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    self.track_stmt_calls(s, counts);
                }
            }
            JsStmt::Export { declaration, .. } => {
                self.track_stmt_calls(declaration, counts);
            }
            _ => {}
        }
    }

    fn track_expr_calls(&self, expr: &JsExpr, counts: &mut HashMap<String, usize>) {
        match expr {
            JsExpr::Call { callee, args, .. } => {
                if let JsExpr::Identifier(id, _) = &**callee {
                    let count = counts.entry(id.clone()).or_insert(0);
                    *count += 1;
                }
                self.track_expr_calls(callee, counts);
                for arg in args {
                    self.track_expr_calls(arg, counts);
                }
            }
            JsExpr::Binary { left, right, .. } => {
                self.track_expr_calls(left, counts);
                self.track_expr_calls(right, counts);
            }
            JsExpr::Array(elements, _) => {
                for el in elements {
                    self.track_expr_calls(el, counts);
                }
            }
            JsExpr::Object(props, _) => {
                for val in props.values() {
                    self.track_expr_calls(val, counts);
                }
            }
            _ => {}
        }
    }

    fn _is_pure_function(&self, body: &[JsStmt]) -> bool {
        // Very conservative purity check
        for stmt in body {
            match stmt {
                JsStmt::Expr(expr, _) => {
                    if !self._is_pure_expr(expr) {
                        return false;
                    }
                }
                JsStmt::VariableDecl { init, .. } => {
                    if let Some(init_expr) = init {
                        if !self._is_pure_expr(init_expr) {
                            return false;
                        }
                    }
                }
                _ => return false,
            }
        }
        true
    }

    fn _is_pure_expr(&self, expr: &JsExpr) -> bool {
        match expr {
            JsExpr::Literal(_, _) => true,
            JsExpr::Identifier(_, _) => true,
            JsExpr::Binary { left, right, .. } => self._is_pure_expr(left) && self._is_pure_expr(right),
            JsExpr::Array(elements, _) => elements.iter().all(|e| self._is_pure_expr(e)),
            JsExpr::Object(properties, _) => properties.values().all(|e| self._is_pure_expr(e)),
            JsExpr::Call { .. } => {
                // Only consider it pure if we know the callee is pure (simplified)
                false
            }
            _ => false,
        }
    }

    pub fn optimize_i18n(&self, ir: &mut IRModule, messages: &HashMap<String, String>) {
        // Optimize template
        if let Some(template) = &mut ir.template {
            for node in &mut template.nodes {
                self.optimize_node_i18n(node, messages);
            }
        }
        // Optimize script (simplified: only handle simple $t('key') calls)
        if let Some(script) = &mut ir.script {
            for stmt in &mut script.body {
                self.optimize_stmt_i18n(stmt, messages);
            }
        }
    }

    fn optimize_node_i18n(&self, node: &mut TemplateNodeIR, messages: &HashMap<String, String>) {
        match node {
            TemplateNodeIR::Element(el) => {
                for attr in &mut el.attributes {
                    if let Some(ast) = &mut attr.value_ast {
                        self.optimize_expr_i18n(ast, messages);
                        // Update value if it's now a literal
                        if let JsExpr::Literal(HxoValue::String(s), _) = ast {
                            attr.value = Some(format!("'{}'", s));
                        }
                    }
                }
                for child in &mut el.children {
                    self.optimize_node_i18n(child, messages);
                }
            }
            TemplateNodeIR::Interpolation(expr) => {
                if let Some(ast) = &mut expr.ast {
                    self.optimize_expr_i18n(ast, messages);
                    // Simplified: update code string if it's now a literal
                    if let JsExpr::Literal(HxoValue::String(s), _) = ast {
                        expr.code = format!("'{}'", s);
                    }
                }
            }
            _ => {}
        }
    }

    fn optimize_stmt_i18n(&self, stmt: &mut JsStmt, messages: &HashMap<String, String>) {
        match stmt {
            JsStmt::Expr(expr, _) => self.optimize_expr_i18n(expr, messages),
            JsStmt::VariableDecl { init, .. } => {
                if let Some(expr) = init {
                    self.optimize_expr_i18n(expr, messages);
                }
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    self.optimize_stmt_i18n(s, messages);
                }
            }
            JsStmt::Export { declaration, .. } => {
                self.optimize_stmt_i18n(declaration, messages);
            }
            _ => {}
        }
    }

    fn optimize_expr_i18n(&self, expr: &mut JsExpr, messages: &HashMap<String, String>) {
        match expr {
            JsExpr::Call { callee, args, .. } => {
                if let JsExpr::Identifier(id, _) = &**callee {
                    if id == "$t" && args.len() == 1 {
                        if let JsExpr::Literal(HxoValue::String(key), _) = &args[0] {
                            if let Some(translated) = messages.get(key) {
                                *expr = JsExpr::Literal(HxoValue::String(translated.clone()), hxo_types::Span::unknown());
                                return;
                            }
                        }
                    }
                }
                for arg in args {
                    self.optimize_expr_i18n(arg, messages);
                }
            }
            JsExpr::Binary { left, right, .. } => {
                self.optimize_expr_i18n(left, messages);
                self.optimize_expr_i18n(right, messages);
            }
            JsExpr::Array(elements, _) => {
                for el in elements {
                    self.optimize_expr_i18n(el, messages);
                }
            }
            JsExpr::Object(props, _) => {
                for val in props.values_mut() {
                    self.optimize_expr_i18n(val, messages);
                }
            }
            _ => {}
        }
    }

    fn optimize_node(&self, node: &mut TemplateNodeIR) {
        match node {
            TemplateNodeIR::Element(el) => {
                // Optimize children first
                for child in &mut el.children {
                    self.optimize_node(child);
                }

                // An element is static if it has no dynamic attributes
                // and all its children are static (text or static elements)
                let has_dynamic_attr = el.attributes.iter().any(|a| a.is_dynamic);
                let all_children_static = el.children.iter().all(|c| match c {
                    TemplateNodeIR::Text(_, _) => true,
                    TemplateNodeIR::Element(child_el) => child_el.is_static,
                    TemplateNodeIR::Comment(_, _) => true,
                    TemplateNodeIR::Interpolation(_) => false,
                });

                el.is_static = !has_dynamic_attr && all_children_static;
            }
            _ => {}
        }
    }
}
