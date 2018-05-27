use hxo_parser::{ParseState, StyleParser};
use hxo_types::Result;
use std::collections::HashMap;

pub struct SassParser;

#[derive(Default)]
pub struct SassParserOptions {}

pub fn compile(source: &str, _options: &SassParserOptions) -> Result<String> {
    let mut state = ParseState::new(source);
    let parser = SassParser;
    parser.parse(&mut state, "sass")
}

impl StyleParser for SassParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<String> {
        let mut parser = SassParserImpl { state, variables: HashMap::new() };
        parser.parse()
    }
}

pub struct SassParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
    variables: HashMap<String, String>,
}

impl<'a, 'b> SassParserImpl<'a, 'b> {
    pub fn parse(&mut self) -> Result<String> {
        let mut css = String::new();
        let mut selector_stack: Vec<(usize, String)> = Vec::new();

        while !self.state.cursor.is_eof() {
            let (indent, line) = self.consume_line_with_indent()?;
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if line.starts_with('$') {
                self.parse_variable_line(line)?;
            }
            else if line.contains(':') && !self.is_selector(&line) {
                // Property
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                let prop = parts[0].trim().to_string();
                let mut val = parts[1].trim().to_string();

                // Replace variables
                for (name, var_val) in &self.variables {
                    val = val.replace(&format!("${}", name), var_val);
                }

                // Find the current rule in css and append property
                let current_selector = self.get_current_selector(&selector_stack);
                let rule_header = format!("{} {{\n", current_selector);
                let prop_line = format!("  {}: {};\n", prop, val);

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
                css.push_str(&format!("{} {{\n}}\n", current_selector));
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
            } // Tab as 2 spaces
            self.state.cursor.consume();
        }
        let line = self.consume_until_newline()?;
        Ok((indent, line))
    }

    fn parse_variable_line(&mut self, line: String) -> Result<()> {
        let parts: Vec<&str> = line[1..].splitn(2, ':').collect();
        if parts.len() == 2 {
            let name = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            self.variables.insert(name, value);
        }
        Ok(())
    }

    fn is_selector(&self, line: &str) -> bool {
        line.contains('.') || line.contains('#') || (!line.contains(':') && !line.starts_with('$'))
    }

    fn get_current_selector(&self, stack: &[(usize, String)]) -> String {
        stack.iter().map(|(_, s)| s.as_str()).collect::<Vec<&str>>().join(" ")
    }

    fn consume_until_newline(&mut self) -> Result<String> {
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '\n' {
            self.state.cursor.consume();
        }
        let s = self.state.cursor.current_str(start).to_string();
        if self.state.cursor.peek() == '\n' {
            self.state.cursor.consume();
        }
        Ok(s.trim().to_string())
    }
}
