use hxo_ir::{AttributeIR, IRModule, JsExpr, JsStmt, TemplateNodeIR};
use hxo_parser_tailwind::StyleEngine;
use hxo_types::{HxoValue, Result};
use std::collections::HashMap;

pub struct Optimizer {
    pub style_engine: StyleEngine,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Optimizer {
    pub fn new() -> Self {
        Self { style_engine: StyleEngine::new() }
    }

    pub fn optimize(&mut self, ir: &mut IRModule, locale: Option<&str>, is_prod: bool) {
        // 1. Static analysis
        if let Some(template) = &mut ir.template {
            for node in &mut template.nodes {
                Self::optimize_node(node);
            }
        }

        // 2. i18n optimization
        if let Some(_locale_val) = locale {
            // Load messages for locale
            let messages = HashMap::new(); // In real case, load from locale files
            Self::optimize_i18n(ir, &messages);
        }

        // 3. Call count tracking for inlining decisions
        let mut call_counts = HashMap::new();
        if let Some(script) = &ir.script {
            Self::track_script_calls(script, &mut call_counts);
        }
        if let Some(template) = &ir.template {
            for node in &template.nodes {
                Self::track_node_calls(node, &mut call_counts);
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

        if let Some(script) = &ir.script {
            self.collect_styles_from_script(script)?;
        }

        for style in &ir.styles {
            // All style parsers (css, scss, tailwind, etc.) return CSS code.
            // We just need to collect it.
            self.style_engine.add_raw_css(&style.code);
        }
        Ok(())
    }

    fn collect_styles_from_script(&mut self, script: &hxo_ir::JsProgram) -> Result<()> {
        for stmt in &script.body {
            self.collect_styles_from_stmt(stmt)?;
        }
        Ok(())
    }

    fn collect_styles_from_stmt(&mut self, stmt: &JsStmt) -> Result<()> {
        match stmt {
            JsStmt::Expr(expr, _) => self.collect_styles_from_expr(expr)?,
            JsStmt::VariableDecl { init: Some(expr), .. } => {
                self.collect_styles_from_expr(expr)?;
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    self.collect_styles_from_stmt(s)?;
                }
            }
            JsStmt::Export { declaration, .. } => {
                self.collect_styles_from_stmt(declaration)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn collect_styles_from_expr(&mut self, expr: &JsExpr) -> Result<()> {
        match expr {
            JsExpr::Literal(HxoValue::String(s), _) => {
                self.style_engine.add_styles(s);
            }
            JsExpr::Call { callee, args, span: _ } => {
                if let JsExpr::Identifier(id, _) = &**callee {
                    if id == "addStyle" {
                        for arg in args {
                            if let JsExpr::Literal(HxoValue::String(s), _) = arg {
                                self.style_engine.add_styles(s);
                            }
                        }
                    }
                }
                self.collect_styles_from_expr(callee)?;
                for arg in args {
                    self.collect_styles_from_expr(arg)?;
                }
            }
            JsExpr::Binary { left, right, .. } => {
                self.collect_styles_from_expr(left)?;
                self.collect_styles_from_expr(right)?;
            }
            JsExpr::Unary { argument, .. } => {
                self.collect_styles_from_expr(argument)?;
            }
            JsExpr::Array(elements, _) => {
                for el in elements {
                    self.collect_styles_from_expr(el)?;
                }
            }
            JsExpr::Object(props, _) => {
                for val in props.values() {
                    self.collect_styles_from_expr(val)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn apply_scope_id(&mut self, ir: &mut IRModule, scope_id: &str) {
        if let Some(template) = &mut ir.template {
            Self::apply_scope_id_to_nodes(&mut template.nodes, scope_id);
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
            if let TemplateNodeIR::Element(el) = node {
                for attr in &el.attributes {
                    if attr.name == "class" {
                        if let Some(value) = &attr.value {
                            self.style_engine.parse_classes(value, attr.span)?;
                        }
                    }
                    else if attr.name == ":class" {
                        if let Some(expr) = &attr.value_ast {
                            self.collect_styles_from_expr(expr)?;
                        }
                    }
                }
                self.collect_styles_from_nodes(&el.children)?;
            }
        }
        Ok(())
    }

    fn apply_scope_id_to_nodes(nodes: &mut [TemplateNodeIR], scope_id: &str) {
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
                Self::apply_scope_id_to_nodes(&mut el.children, scope_id);
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

    fn track_script_calls(script: &hxo_ir::JsProgram, counts: &mut HashMap<String, usize>) {
        for stmt in &script.body {
            Self::track_stmt_calls(stmt, counts);
        }
    }

    fn track_node_calls(node: &TemplateNodeIR, counts: &mut HashMap<String, usize>) {
        match node {
            TemplateNodeIR::Element(el) => {
                for attr in &el.attributes {
                    if let Some(expr) = &attr.value_ast {
                        Self::track_expr_calls(expr, counts);
                    }
                }
                for child in &el.children {
                    Self::track_node_calls(child, counts);
                }
            }
            TemplateNodeIR::Interpolation(expr) => {
                if let Some(ast) = &expr.ast {
                    Self::track_expr_calls(ast, counts);
                }
            }
            _ => {}
        }
    }

    fn track_stmt_calls(stmt: &JsStmt, counts: &mut HashMap<String, usize>) {
        match stmt {
            JsStmt::Expr(expr, _) => Self::track_expr_calls(expr, counts),
            JsStmt::VariableDecl { init: Some(expr), .. } => {
                Self::track_expr_calls(expr, counts);
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    Self::track_stmt_calls(s, counts);
                }
            }
            JsStmt::Export { declaration, .. } => {
                Self::track_stmt_calls(declaration, counts);
            }
            _ => {}
        }
    }

    fn track_expr_calls(expr: &JsExpr, counts: &mut HashMap<String, usize>) {
        match expr {
            JsExpr::Call { callee, args, .. } => {
                if let JsExpr::Identifier(id, _) = &**callee {
                    let count = counts.entry(id.clone()).or_insert(0);
                    *count += 1;
                }
                Self::track_expr_calls(callee, counts);
                for arg in args {
                    Self::track_expr_calls(arg, counts);
                }
            }
            JsExpr::Binary { left, right, .. } => {
                Self::track_expr_calls(left, counts);
                Self::track_expr_calls(right, counts);
            }
            JsExpr::Unary { argument, .. } => {
                Self::track_expr_calls(argument, counts);
            }
            JsExpr::Array(elements, _) => {
                for el in elements {
                    Self::track_expr_calls(el, counts);
                }
            }
            JsExpr::Object(props, _) => {
                for val in props.values() {
                    Self::track_expr_calls(val, counts);
                }
            }
            _ => {}
        }
    }

    fn _is_pure_function(body: &[JsStmt]) -> bool {
        // Very conservative purity check
        for stmt in body {
            match stmt {
                JsStmt::Expr(expr, _) => {
                    if !Self::_is_pure_expr(expr) {
                        return false;
                    }
                }
                JsStmt::VariableDecl { init: Some(init_expr), .. } => {
                    if !Self::_is_pure_expr(init_expr) {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        true
    }

    fn _is_pure_expr(expr: &JsExpr) -> bool {
        match expr {
            JsExpr::Literal(_, _) => true,
            JsExpr::Identifier(_, _) => true,
            JsExpr::Binary { left, right, .. } => Self::_is_pure_expr(left) && Self::_is_pure_expr(right),
            JsExpr::Unary { argument, .. } => Self::_is_pure_expr(argument),
            JsExpr::Array(elements, _) => elements.iter().all(Self::_is_pure_expr),
            JsExpr::Object(properties, _) => properties.values().all(Self::_is_pure_expr),
            JsExpr::Call { .. } => {
                // Only consider it pure if we know the callee is pure (simplified)
                false
            }
            _ => false,
        }
    }

    pub fn optimize_i18n(ir: &mut IRModule, messages: &HashMap<String, String>) {
        // Optimize template
        if let Some(template) = &mut ir.template {
            for node in &mut template.nodes {
                Self::optimize_node_i18n(node, messages);
            }
        }
        // Optimize script (simplified: only handle simple $t('key') calls)
        if let Some(script) = &mut ir.script {
            for stmt in &mut script.body {
                Self::optimize_stmt_i18n(stmt, messages);
            }
        }
    }

    fn optimize_node_i18n(node: &mut TemplateNodeIR, messages: &HashMap<String, String>) {
        match node {
            TemplateNodeIR::Element(el) => {
                for attr in &mut el.attributes {
                    if let Some(ast) = &mut attr.value_ast {
                        Self::optimize_expr_i18n(ast, messages);
                        // Update value if it's now a literal
                        if let JsExpr::Literal(HxoValue::String(s), _) = ast {
                            attr.value = Some(format!("'{}'", s));
                        }
                    }
                }
                for child in &mut el.children {
                    Self::optimize_node_i18n(child, messages);
                }
            }
            TemplateNodeIR::Interpolation(expr) => {
                if let Some(ast) = &mut expr.ast {
                    Self::optimize_expr_i18n(ast, messages);
                    // Simplified: update code string if it's now a literal
                    if let JsExpr::Literal(HxoValue::String(s), _) = ast {
                        expr.code = format!("'{}'", s);
                    }
                }
            }
            _ => {}
        }
    }

    fn optimize_stmt_i18n(stmt: &mut JsStmt, messages: &HashMap<String, String>) {
        match stmt {
            JsStmt::Expr(expr, _) => Self::optimize_expr_i18n(expr, messages),
            JsStmt::VariableDecl { init: Some(expr), .. } => {
                Self::optimize_expr_i18n(expr, messages);
            }
            JsStmt::FunctionDecl { body, .. } => {
                for s in body {
                    Self::optimize_stmt_i18n(s, messages);
                }
            }
            JsStmt::Export { declaration, .. } => {
                Self::optimize_stmt_i18n(declaration, messages);
            }
            _ => {}
        }
    }

    fn optimize_expr_i18n(expr: &mut JsExpr, messages: &HashMap<String, String>) {
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
                    Self::optimize_expr_i18n(arg, messages);
                }
            }
            JsExpr::Binary { left, right, .. } => {
                Self::optimize_expr_i18n(left, messages);
                Self::optimize_expr_i18n(right, messages);
            }
            JsExpr::Array(elements, _) => {
                for el in elements {
                    Self::optimize_expr_i18n(el, messages);
                }
            }
            JsExpr::Object(props, _) => {
                for val in props.values_mut() {
                    Self::optimize_expr_i18n(val, messages);
                }
            }
            _ => {}
        }
    }

    fn optimize_node(node: &mut TemplateNodeIR) {
        if let TemplateNodeIR::Element(el) = node {
            // Optimize children first
            for child in &mut el.children {
                Self::optimize_node(child);
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
    }
}
