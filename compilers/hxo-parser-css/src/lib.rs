use hxo_parser::{ParseState, StyleSubParser};
use hxo_types::Result;

pub struct CssParser;

impl StyleSubParser for CssParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<String> {
        let mut parser = CssParserImpl { state, minify: false };
        parser.parse()
    }
}

pub struct CssParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
    minify: bool,
}

impl<'a, 'b> CssParserImpl<'a, 'b> {
    pub fn parse(&mut self) -> Result<String> {
        let mut output = String::new();
        while !self.state.cursor.is_eof() {
            self.state.cursor.skip_whitespace();
            if self.state.cursor.is_eof() {
                break;
            }

            // Skip comments
            if self.state.cursor.peek_str("/*") {
                self.skip_comment()?;
                continue;
            }

            // Handle At-rules
            if self.state.cursor.peek() == '@' {
                let at_rule = self.parse_at_rule()?;
                if !output.is_empty() && !self.minify {
                    output.push('\n');
                }
                output.push_str(&at_rule);
                continue;
            }

            // Simple selector parsing
            let selector = self.consume_until('{')?;
            if selector.trim().is_empty() && self.state.cursor.is_eof() {
                break;
            }

            self.state.cursor.expect('{')?;
            let declarations = self.parse_declarations()?;
            self.state.cursor.expect('}')?;

            if !output.is_empty() && !self.minify {
                output.push('\n');
            }

            if self.minify {
                output.push_str(selector.trim());
                output.push('{');
                output.push_str(&declarations);
                output.push('}');
            }
            else {
                output.push_str(selector.trim());
                output.push_str(" {\n");
                for line in declarations.lines() {
                    output.push_str("  ");
                    output.push_str(line);
                    output.push('\n');
                }
                output.push_str("}\n");
            }
        }
        Ok(output)
    }

    fn skip_comment(&mut self) -> Result<()> {
        self.state.cursor.expect_str("/*")?;
        while !self.state.cursor.is_eof() && !self.state.cursor.peek_str("*/") {
            self.state.cursor.consume();
        }
        self.state.cursor.expect_str("*/")?;
        Ok(())
    }

    fn parse_at_rule(&mut self) -> Result<String> {
        let start = self.state.cursor.pos;
        self.state.cursor.consume(); // '@'

        // Read until '{' or ';'
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '{' && self.state.cursor.peek() != ';' {
            self.state.cursor.consume();
        }

        if self.state.cursor.peek() == ';' {
            let rule = self.state.cursor.current_str(start).to_string();
            self.state.cursor.consume();
            return Ok(format!("{};", rule.trim()));
        }

        let rule_head = self.state.cursor.current_str(start).trim().to_string();
        self.state.cursor.expect('{')?;

        let mut brace_count = 1;
        let body_start = self.state.cursor.pos;

        while !self.state.cursor.is_eof() && brace_count > 0 {
            if self.state.cursor.peek() == '{' {
                brace_count += 1;
            }
            else if self.state.cursor.peek() == '}' {
                brace_count -= 1;
            }
            if brace_count > 0 {
                self.state.cursor.consume();
            }
        }

        let raw_body = self.state.cursor.current_str(body_start).to_string();
        self.state.cursor.expect('}')?;

        // Recursively parse body if it looks like CSS rules
        let mut inner_parser = CssParserImpl { state: &mut ParseState::new(&raw_body), minify: self.minify };
        let parsed_body = inner_parser.parse()?;

        if self.minify {
            Ok(format!("{}{{{}}}", rule_head, parsed_body))
        }
        else {
            let indented_body = parsed_body
                .lines()
                .map(|line| if line.is_empty() { line.to_string() } else { format!("  {}", line) })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(format!("{} {{\n{}\n}}\n", rule_head, indented_body))
        }
    }

    fn parse_declarations(&mut self) -> Result<String> {
        let mut declarations = String::new();
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek() == '}' {
                break;
            }

            let property = self.consume_until(':')?;
            self.state.cursor.expect(':')?;
            self.state.cursor.skip_whitespace();
            let value = self.consume_until(';')?;
            self.state.cursor.expect(';')?;

            let property = property.trim();
            let value = value.trim();

            if !property.is_empty() && !value.is_empty() {
                if !self.minify {
                    declarations.push_str("  ");
                }
                declarations.push_str(property);
                if self.minify {
                    declarations.push(':');
                }
                else {
                    declarations.push_str(": ");
                }
                declarations.push_str(value);
                declarations.push(';');
                if !self.minify {
                    declarations.push('\n');
                }
            }
            self.state.cursor.skip_whitespace();
        }
        Ok(declarations)
    }

    fn consume_until(&mut self, delimiter: char) -> Result<String> {
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != delimiter {
            self.state.cursor.consume();
        }
        Ok(self.state.cursor.source[start..self.state.cursor.pos].to_string())
    }
}

pub fn compile(source: &str) -> Result<String> {
    let mut state = ParseState::new(source);
    CssParser.parse(&mut state, "css")
}

pub fn parse(source: &str, minify: bool) -> Result<String> {
    let mut state = ParseState::new(source);
    let mut parser = CssParserImpl { state: &mut state, minify };
    parser.parse()
}
