use hxo_parser::{ParseState, StyleParser};
use hxo_types::{Position, Result, Span};
use std::collections::HashMap;

pub struct TailwindParser;

impl StyleParser for TailwindParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<String> {
        let mut engine = StyleEngine::new();
        let content = state.cursor.source[state.cursor.pos..].to_string();
        engine.parse_classes(&content, state.cursor.span_from(state.cursor.position()))?;

        Ok(engine.generate_css())
    }
}

pub struct StyleRule {
    pub selector: String,
    pub declarations: Vec<(String, String)>,
    pub span: Span,
}

struct TailwindConfig {
    spacing: HashMap<String, String>,
    colors: HashMap<String, String>,
}

#[derive(Default)]
pub struct StyleEngine {
    pub rules: Vec<StyleRule>,
    pub raw_css: Vec<String>,
    config: TailwindConfig,
}

impl StyleEngine {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TailwindConfig {
    pub fn new() -> Self {
        let mut spacing = HashMap::new();
        spacing.insert("0".to_string(), "0".to_string());
        spacing.insert("1".to_string(), "0.25rem".to_string());
        spacing.insert("2".to_string(), "0.5rem".to_string());
        spacing.insert("4".to_string(), "1rem".to_string());
        spacing.insert("6".to_string(), "1.5rem".to_string());
        spacing.insert("8".to_string(), "2rem".to_string());

        let mut colors = HashMap::new();
        colors.insert("red".to_string(), "#ff0000".to_string());
        colors.insert("blue".to_string(), "#0000ff".to_string());
        colors.insert("white".to_string(), "#ffffff".to_string());
        colors.insert("black".to_string(), "#000000".to_string());
        colors.insert("gray-100".to_string(), "#f3f4f6".to_string());
        colors.insert("gray-800".to_string(), "#1f2937".to_string());

        Self { spacing, colors }
    }
}

impl Default for TailwindConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleEngine {
    pub fn add_raw_css(&mut self, css: &str) {
        self.raw_css.push(css.to_string());
    }

    /// Add a single Tailwind class
    pub fn add_style(&mut self, class: &str) {
        if let Some(rule) = self.resolve_class(class, Span::unknown()) {
            // Avoid duplicates
            if !self.rules.iter().any(|r| r.selector == rule.selector) {
                self.rules.push(rule);
            }
        }
    }

    /// Add multiple Tailwind classes from a space-separated string
    pub fn add_styles(&mut self, classes: &str) {
        let _ = self.parse_classes(classes, Span::unknown());
    }

    pub fn parse_classes(&mut self, class_str: &str, base_span: Span) -> Result<()> {
        let mut current_column = base_span.start.column;
        let mut current_offset = base_span.start.offset;

        for class in class_str.split_whitespace() {
            let class_len = class.len() as u32;
            let span = Span {
                start: Position { line: base_span.start.line, column: current_column, offset: current_offset },
                end: Position {
                    line: base_span.start.line,
                    column: current_column + class_len,
                    offset: current_offset + class_len,
                },
            };

            if let Some(rule) = self.resolve_class(class, span) {
                self.rules.push(rule);
            }

            current_column += class_len + 1;
            current_offset += class_len + 1;
        }
        Ok(())
    }

    fn resolve_class(&self, class: &str, span: Span) -> Option<StyleRule> {
        let mut declarations = Vec::new();

        if let Some(val) = class.strip_prefix("m-") {
            if let Some(spacing) = self.config.spacing.get(val) {
                declarations.push(("margin".to_string(), spacing.clone()));
            }
        }
        else if let Some(val) = class.strip_prefix("mx-") {
            if let Some(spacing) = self.config.spacing.get(val) {
                declarations.push(("margin-left".to_string(), spacing.clone()));
                declarations.push(("margin-right".to_string(), spacing.clone()));
            }
        }
        else if let Some(val) = class.strip_prefix("my-") {
            if let Some(spacing) = self.config.spacing.get(val) {
                declarations.push(("margin-top".to_string(), spacing.clone()));
                declarations.push(("margin-bottom".to_string(), spacing.clone()));
            }
        }
        else if let Some(val) = class.strip_prefix("p-") {
            if let Some(spacing) = self.config.spacing.get(val) {
                declarations.push(("padding".to_string(), spacing.clone()));
            }
        }
        else if let Some(val) = class.strip_prefix("px-") {
            if let Some(spacing) = self.config.spacing.get(val) {
                declarations.push(("padding-left".to_string(), spacing.clone()));
                declarations.push(("padding-right".to_string(), spacing.clone()));
            }
        }
        else if let Some(val) = class.strip_prefix("py-") {
            if let Some(spacing) = self.config.spacing.get(val) {
                declarations.push(("padding-top".to_string(), spacing.clone()));
                declarations.push(("padding-bottom".to_string(), spacing.clone()));
            }
        }
        else if let Some(val) = class.strip_prefix("w-") {
            if let Some(spacing) = self.config.spacing.get(val) {
                declarations.push(("width".to_string(), spacing.clone()));
            }
            else if val == "full" {
                declarations.push(("width".to_string(), "100%".to_string()));
            }
            else if val == "screen" {
                declarations.push(("width".to_string(), "100vw".to_string()));
            }
        }
        else if let Some(val) = class.strip_prefix("h-") {
            if let Some(spacing) = self.config.spacing.get(val) {
                declarations.push(("height".to_string(), spacing.clone()));
            }
            else if val == "full" {
                declarations.push(("height".to_string(), "100%".to_string()));
            }
            else if val == "screen" {
                declarations.push(("height".to_string(), "100vh".to_string()));
            }
        }
        else if let Some(val) = class.strip_prefix("text-") {
            if let Some(color) = self.config.colors.get(val) {
                declarations.push(("color".to_string(), color.clone()));
            }
            else if val == "center" {
                declarations.push(("text-align".to_string(), "center".to_string()));
            }
            else if val == "xl" {
                declarations.push(("font-size".to_string(), "1.25rem".to_string()));
            }
            else if val == "2xl" {
                declarations.push(("font-size".to_string(), "1.5rem".to_string()));
            }
        }
        else if let Some(val) = class.strip_prefix("bg-") {
            if let Some(color) = self.config.colors.get(val) {
                declarations.push(("background-color".to_string(), color.clone()));
            }
        }
        else if class == "flex" {
            declarations.push(("display".to_string(), "flex".to_string()));
        }
        else if class == "grid" {
            declarations.push(("display".to_string(), "grid".to_string()));
        }
        else if let Some(val) = class.strip_prefix("grid-cols-") {
            if let Ok(n) = val.parse::<u32>() {
                declarations.push(("grid-template-columns".to_string(), format!("repeat({}, minmax(0, 1fr))", n)));
            }
        }
        else if class == "block" {
            declarations.push(("display".to_string(), "block".to_string()));
        }
        else if class == "inline-block" {
            declarations.push(("display".to_string(), "inline-block".to_string()));
        }
        else if class == "hidden" {
            declarations.push(("display".to_string(), "none".to_string()));
        }
        else if class == "items-center" {
            declarations.push(("align-items".to_string(), "center".to_string()));
        }
        else if class == "justify-center" {
            declarations.push(("justify-content".to_string(), "center".to_string()));
        }
        else if let Some(val) = class.strip_prefix("gap-") {
            if let Some(spacing) = self.config.spacing.get(val) {
                declarations.push(("gap".to_string(), spacing.clone()));
            }
        }
        else if class == "font-bold" {
            declarations.push(("font-weight".to_string(), "700".to_string()));
        }
        else if class == "rounded-lg" {
            declarations.push(("border-radius".to_string(), "0.5rem".to_string()));
        }
        else if class == "rounded-xl" {
            declarations.push(("border-radius".to_string(), "0.75rem".to_string()));
        }
        else if class == "shadow-sm" {
            declarations.push(("box-shadow".to_string(), "0 1px 2px 0 rgba(0, 0, 0, 0.05)".to_string()));
        }
        else if class == "absolute" {
            declarations.push(("position".to_string(), "absolute".to_string()));
        }
        else if class == "relative" {
            declarations.push(("position".to_string(), "relative".to_string()));
        }
        else if class == "fixed" {
            declarations.push(("position".to_string(), "fixed".to_string()));
        }
        else if let Some(val) = class.strip_prefix("top-") {
            if let Some(spacing) = self.config.spacing.get(val) {
                declarations.push(("top".to_string(), spacing.clone()));
            }
        }
        else if let Some(val) = class.strip_prefix("left-") {
            if let Some(spacing) = self.config.spacing.get(val) {
                declarations.push(("left".to_string(), spacing.clone()));
            }
        }

        if !declarations.is_empty() { Some(StyleRule { selector: format!(".{}", class), declarations, span }) } else { None }
    }

    pub fn generate_css(&self) -> String {
        let mut css = String::new();
        for rule in &self.rules {
            css.push_str(&format!("{} {{\n", rule.selector));
            for (prop, val) in &rule.declarations {
                css.push_str(&format!("  {}: {};\n", prop, val));
            }
            css.push_str("}\n");
        }
        for raw in &self.raw_css {
            css.push_str(raw);
            css.push('\n');
        }
        css
    }
}
