use hxo_types::{Position, Result, Span};
use std::collections::HashMap;

pub struct StyleRule {
    pub selector: String,
    pub declarations: Vec<(String, String)>,
    pub span: Span,
}

struct TailwindConfig {
    spacing: HashMap<String, String>,
    colors: HashMap<String, String>,
}

pub struct StyleEngine {
    pub rules: Vec<StyleRule>,
    config: TailwindConfig,
}

impl StyleEngine {
    pub fn new() -> Self {
        let mut spacing = HashMap::new();
        spacing.insert("0".to_string(), "0".to_string());
        spacing.insert("1".to_string(), "0.25rem".to_string());
        spacing.insert("2".to_string(), "0.5rem".to_string());
        spacing.insert("4".to_string(), "1rem".to_string());

        let mut colors = HashMap::new();
        colors.insert("red".to_string(), "#ff0000".to_string());
        colors.insert("blue".to_string(), "#0000ff".to_string());
        colors.insert("white".to_string(), "#ffffff".to_string());

        Self { rules: Vec::new(), config: TailwindConfig { spacing, colors } }
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
        if class.starts_with("m-") {
            let val = &class[2..];
            if let Some(spacing) = self.config.spacing.get(val) {
                return Some(StyleRule {
                    selector: format!(".{}", class),
                    declarations: vec![("margin".to_string(), spacing.clone())],
                    span,
                });
            }
        }
        else if class.starts_with("p-") {
            let val = &class[2..];
            if let Some(spacing) = self.config.spacing.get(val) {
                return Some(StyleRule {
                    selector: format!(".{}", class),
                    declarations: vec![("padding".to_string(), spacing.clone())],
                    span,
                });
            }
        }
        else if class.starts_with("text-") {
            let val = &class[5..];
            if let Some(color) = self.config.colors.get(val) {
                return Some(StyleRule {
                    selector: format!(".{}", class),
                    declarations: vec![("color".to_string(), color.clone())],
                    span,
                });
            }
        }
        else if class.starts_with("bg-") {
            let val = &class[3..];
            if let Some(color) = self.config.colors.get(val) {
                return Some(StyleRule {
                    selector: format!(".{}", class),
                    declarations: vec![("background-color".to_string(), color.clone())],
                    span,
                });
            }
        }
        else if class == "rounded-lg" {
            return Some(StyleRule {
                selector: ".rounded-lg".to_string(),
                declarations: vec![("border-radius".to_string(), "0.5rem".to_string())],
                span,
            });
        }
        None
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
        css
    }
}
