use hxo_parser::{ParseState, StyleSubParser};
use hxo_types::Result;
use std::collections::HashMap;

pub struct LessParser;

impl StyleSubParser for LessParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<String> {
        let mut parser = LessParserImpl { state, variables: HashMap::new() };
        parser.parse()
    }
}

pub struct LessParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
    variables: HashMap<String, String>,
}

impl<'a, 'b> LessParserImpl<'a, 'b> {
    pub fn parse(&mut self) -> Result<String> {
        let mut css = String::new();
        let mut selector_stack: Vec<String> = Vec::new();

        while !self.state.cursor.is_eof() {
            self.skip_comments_and_whitespace();
            if self.state.cursor.is_eof() {
                break;
            }

            if self.state.cursor.peek() == '@' {
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

    fn consume_ident(&mut self) -> Result<String> {
        self.state.cursor.skip_whitespace();
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && (self.state.cursor.peek().is_alphanumeric() || self.state.cursor.peek() == '-') {
            self.state.cursor.consume();
        }
        Ok(self.state.cursor.source[start..self.state.cursor.pos].to_string())
    }

    fn consume_until_semicolon(&mut self) -> Result<String> {
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != ';' {
            self.state.cursor.consume();
        }
        let val = self.state.cursor.source[start..self.state.cursor.pos].to_string();
        if self.state.cursor.peek() == ';' {
            self.state.cursor.consume();
        }
        Ok(val)
    }

    fn consume_until_char(&mut self, target: char) -> Result<String> {
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != target {
            self.state.cursor.consume();
        }
        Ok(self.state.cursor.source[start..self.state.cursor.pos].to_string())
    }

    fn is_nested_rule(&self) -> bool {
        let mut i = 0;
        while let Some(c) = self.state.cursor.source[self.state.cursor.pos..].chars().nth(i) {
            if c == '{' {
                return true;
            }
            if c == ';' || c == '}' {
                return false;
            }
            if c == ':' {
                // Could be a pseudo-selector like :hover, or a property like color:
                // If it's followed by whitespace and then not a { it's probably a property
                let next = self.state.cursor.source[self.state.cursor.pos..].chars().nth(i + 1).unwrap_or('\0');
                if next.is_whitespace() || next.is_alphanumeric() {
                     // Check if there is a { before a ;
                     let mut j = i + 1;
                     while let Some(c2) = self.state.cursor.source[self.state.cursor.pos..].chars().nth(j) {
                         if c2 == '{' { return true; }
                         if c2 == ';' || c2 == '}' { return false; }
                         j += 1;
                     }
                     return false;
                }
            }
            i += 1;
        }
        false
    }

    fn parse_variable(&mut self) -> Result<()> {
        self.state.cursor.consume(); // @
        let name = self.consume_ident()?;
        self.state.cursor.skip_whitespace();
        self.state.cursor.expect(':')?;
        self.state.cursor.skip_whitespace();
        let value = self.consume_until_semicolon()?;
        self.variables.insert(name, value.trim().to_string());
        Ok(())
    }

    fn parse_rule(&mut self, selector_stack: &mut Vec<String>) -> Result<String> {
        self.state.cursor.skip_whitespace();
        let selector = self.consume_until_char('{')?.trim().to_string();
        self.state.cursor.expect('{')?;

        selector_stack.push(selector.clone());
        let full_selector = selector_stack.join(" ");

        let mut css = String::new();
        let mut declarations = Vec::new();

        self.skip_comments_and_whitespace();
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
            self.skip_comments_and_whitespace();
            if self.state.cursor.is_eof() || self.state.cursor.peek() == '}' {
                break;
            }

            if self.state.cursor.peek() == '.'
                || self.state.cursor.peek() == '#'
                || self.is_nested_rule()
            {
                // Nested rule
                css.push_str(&self.parse_rule(selector_stack)?);
            }
            else {
                // Declaration
                let prop = self.consume_until_char(':')?.trim().to_string();
                if self.state.cursor.peek() == ':' {
                    self.state.cursor.consume();
                    let mut val = self.consume_until_semicolon()?.trim().to_string();

                    // Replace variables
                    for (name, var_val) in &self.variables {
                        val = val.replace(&format!("@{}", name), var_val);
                    }

                    declarations.push(format!("  {}: {};", prop, val));
                }
            }
            self.skip_comments_and_whitespace();
        }

        if !self.state.cursor.is_eof() && self.state.cursor.peek() == '}' {
            self.state.cursor.consume();
        }

        selector_stack.pop();

        let mut result = String::new();
        if !declarations.is_empty() {
            result.push_str(&format!("{} {{\n", full_selector));
            for decl in declarations {
                result.push_str(&decl);
                result.push('\n');
            }
            result.push_str("}\n");
        }
        
        Ok(result + &css)
    }

}

pub fn compile(source: &str) -> Result<String> {
    let mut state = ParseState::new(source);
    let mut parser = LessParserImpl { state: &mut state, variables: HashMap::new() };
    parser.parse()
}
