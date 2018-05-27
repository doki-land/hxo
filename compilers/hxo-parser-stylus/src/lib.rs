use hxo_parser::{ParseState, StyleParser};
use hxo_types::Result;
use std::collections::HashMap;

pub struct StylusParser;

pub fn compile(source: &str) -> Result<String> {
    let mut state = ParseState::new(source);
    let parser = StylusParser;
    parser.parse(&mut state, "stylus")
}

impl StyleParser for StylusParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<String> {
        let mut parser = StylusParserImpl { state, variables: HashMap::new() };
        parser.parse()
    }
}

struct StylusParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
    variables: HashMap<String, String>,
}

impl<'a, 'b> StylusParserImpl<'a, 'b> {
    pub fn parse(&mut self) -> Result<String> {
        let mut css = String::new();
        let mut selector_stack: Vec<(usize, String)> = Vec::new();

        while !self.state.cursor.is_eof() {
            let (indent, line) = self.consume_line_with_indent()?;
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // Handle variables: var = value
            if line.contains('=') && !line.contains('{') && !line.contains(':') {
                self.parse_variable_line(&line)?;
                continue;
            }

            // Determine if it's a property or selector
            // In Stylus, if it's indented and contains multiple words or a colon, it's likely a property
            let is_property = indent > 0 && (line.contains(':') || line.contains(' '));

            if is_property {
                let (prop, val) = if line.contains(':') {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();
                    (parts[0].trim().to_string(), parts[1].trim().to_string())
                }
                else {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        (parts[0].trim().to_string(), parts[1].trim().to_string())
                    }
                    else {
                        (line.clone(), "".to_string())
                    }
                };

                let mut final_val = val;
                for (name, var_val) in &self.variables {
                    final_val = final_val.replace(name, var_val);
                }

                let current_selector = self.get_current_selector(&selector_stack);
                let rule_header = format!("{} {{\n", current_selector);
                let prop_line = format!("  {}: {};\n", prop, final_val);

                if let Some(pos) = css.rfind(&rule_header) {
                    let end_pos = css[pos..].find('}').map(|i| i + pos).unwrap_or(css.len());
                    css.insert_str(end_pos, &prop_line);
                }
                else {
                    css.push_str(&rule_header);
                    css.push_str(&prop_line);
                    css.push_str("}\n");
                }
            }
            else {
                // Selector
                while !selector_stack.is_empty() && indent <= selector_stack.last().unwrap().0 {
                    selector_stack.pop();
                }
                selector_stack.push((indent, line));
                let current_selector = self.get_current_selector(&selector_stack);
                // Pre-create the rule if it doesn't exist
                let rule_header = format!("{} {{\n", current_selector);
                if !css.contains(&rule_header) {
                    css.push_str(&rule_header);
                    css.push_str("}\n");
                }
            }
        }

        Ok(css.replace("{\n}\n", "").trim().to_string())
    }

    fn consume_line_with_indent(&mut self) -> Result<(usize, String)> {
        let mut indent = 0;
        while !self.state.cursor.is_eof() && (self.state.cursor.peek() == ' ' || self.state.cursor.peek() == '\t') {
            if self.state.cursor.peek() == ' ' {
                indent += 1;
            }
            else {
                indent += 2;
            }
            self.state.cursor.consume();
        }
        let line = self.consume_until_newline()?;
        Ok((indent, line))
    }

    fn consume_until_newline(&mut self) -> Result<String> {
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '\n' {
            self.state.cursor.consume();
        }
        let s = self.state.cursor.current_str(start).to_string();
        if !self.state.cursor.is_eof() && self.state.cursor.peek() == '\n' {
            self.state.cursor.consume();
        }
        Ok(s.trim().to_string())
    }

    fn parse_variable_line(&mut self, line: &str) -> Result<()> {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            let name = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            self.variables.insert(name, value);
        }
        Ok(())
    }

    fn get_current_selector(&self, stack: &[(usize, String)]) -> String {
        stack.iter().map(|(_, s)| s.as_str()).collect::<Vec<&str>>().join(" ")
    }
}
