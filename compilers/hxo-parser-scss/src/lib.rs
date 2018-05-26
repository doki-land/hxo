use hxo_parser::{ParseState, StyleSubParser};
use hxo_types::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ScssParserOptions {
    pub is_compressed: bool,
    pub load_paths: Vec<std::path::PathBuf>,
}

pub fn compile(source: &str, _options: &ScssParserOptions) -> Result<String> {
    let mut state = ParseState::new(source);
    ScssParser.parse(&mut state, "scss")
}

pub struct ScssParser;

impl StyleSubParser for ScssParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<String> {
        let mut parser = ScssParserImpl { state, variables: HashMap::new() };
        parser.parse()
    }
}

pub struct ScssParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
    variables: HashMap<String, String>,
}

impl<'a, 'b> ScssParserImpl<'a, 'b> {
    pub fn parse(&mut self) -> Result<String> {
        let mut css = String::new();
        let mut selector_stack: Vec<String> = Vec::new();

        while !self.state.cursor.is_eof() {
            self.skip_comments_and_whitespace();
            if self.state.cursor.is_eof() {
                break;
            }

            if self.state.cursor.peek() == '$' {
                self.parse_variable()?;
            }
            else if self.state.cursor.peek() == '.'
                || self.state.cursor.peek() == '#'
                || self.state.cursor.peek().is_ascii_alphabetic()
            {
                css.push_str(&self.parse_rule(&mut selector_stack)?);
            }
            else {
                self.state.cursor.consume();
            }
        }

        Ok(css.trim().to_string())
    }

    fn skip_comments_and_whitespace(&mut self) {
        while !self.state.cursor.is_eof() {
            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek_str("//") {
                self.consume_until_newline();
            }
            else if self.state.cursor.peek_str("/*") {
                self.consume_until_str("*/");
                if self.state.cursor.peek_str("*/") {
                    self.state.cursor.consume_n(2);
                }
            }
            else {
                break;
            }
        }
    }

    fn consume_until_newline(&mut self) {
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '\n' {
            self.state.cursor.consume();
        }
    }

    fn consume_until_str(&mut self, target: &str) {
        while !self.state.cursor.is_eof() && !self.state.cursor.peek_str(target) {
            self.state.cursor.consume();
        }
    }

    fn parse_variable(&mut self) -> Result<()> {
        self.state.cursor.consume(); // $
        let name = self.consume_ident()?;
        self.state.cursor.skip_whitespace();
        self.state.cursor.expect(':')?;
        self.state.cursor.skip_whitespace();
        let value = self.consume_until(';')?;
        self.state.cursor.expect(';')?;
        self.variables.insert(name, value.trim().to_string());
        Ok(())
    }

    fn parse_rule(&mut self, selector_stack: &mut Vec<String>) -> Result<String> {
        let selector = self.consume_until('{')?;
        self.state.cursor.expect('{')?;
        selector_stack.push(selector.trim().to_string());

        let mut css = String::new();
        let mut declarations = Vec::new();

        self.skip_comments_and_whitespace();
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
            if self.state.cursor.peek() == '.'
                || self.state.cursor.peek() == '#'
                || (self.state.cursor.peek().is_ascii_alphabetic() && !self.is_property())
            {
                css.push_str(&self.parse_rule(selector_stack)?);
            }
            else if self.state.cursor.peek() == '$' {
                self.parse_variable()?;
            }
            else if self.state.cursor.peek().is_ascii_alphabetic() {
                let prop = self.consume_until(':')?;
                self.state.cursor.expect(':')?;
                self.state.cursor.skip_whitespace();
                let val = self.consume_until(';')?;
                self.state.cursor.expect(';')?;

                let mut processed_val = val.trim().to_string();
                // Replace variables
                for (name, var_val) in &self.variables {
                    processed_val = processed_val.replace(&format!("${}", name), var_val);
                }

                declarations.push(format!("  {}: {};\n", prop.trim(), processed_val));
            }
            else {
                self.state.cursor.consume();
            }
            self.skip_comments_and_whitespace();
        }
        self.state.cursor.expect('}')?;

        let mut output = String::new();
        if !declarations.is_empty() {
            let full_selector = selector_stack.join(" ");
            output.push_str(&format!("{} {{\n", full_selector));
            for decl in declarations {
                output.push_str(&decl);
            }
            output.push_str("}\n");
        }
        output.push_str(&css);

        selector_stack.pop();
        Ok(output)
    }

    fn is_property(&self) -> bool {
        let mut pos = self.state.cursor.pos;
        while pos < self.state.cursor.source.len() {
            let c = self.state.cursor.source.as_bytes()[pos] as char;
            if c == ':' {
                return true;
            }
            if c == '{' || c == '}' || c == ';' {
                return false;
            }
            pos += 1;
        }
        false
    }

    fn consume_ident(&mut self) -> Result<String> {
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof()
            && (self.state.cursor.peek().is_ascii_alphanumeric()
                || self.state.cursor.peek() == '-'
                || self.state.cursor.peek() == '_')
        {
            self.state.cursor.consume();
        }
        Ok(self.state.cursor.current_str(start).to_string())
    }

    fn consume_until(&mut self, c: char) -> Result<String> {
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != c {
            self.state.cursor.consume();
        }
        Ok(self.state.cursor.current_str(start).to_string())
    }
}

pub fn parse(source: &str, _options: &ScssParserOptions) -> Result<String> {
    let mut state = ParseState::new(source);
    let mut parser = ScssParserImpl { state: &mut state, variables: HashMap::new() };
    parser.parse()
}
