use hxo_parser::{ParseState, StyleParser};
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

impl StyleParser for ScssParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<String> {
        let mut parser = ScssParserImpl { state, variables: HashMap::new() };
        parser.parse()
    }
}

#[derive(Debug, Clone)]
enum CssExpr {
    Number(f64, String),
    Variable(String),
    Binary(Box<CssExpr>, String, Box<CssExpr>),
    Raw(String),
}

impl CssExpr {
    fn to_string(&self, variables: &HashMap<String, String>) -> String {
        match self {
            CssExpr::Number(n, unit) => format!("{}{}", n, unit),
            CssExpr::Variable(name) => variables.get(name).cloned().unwrap_or_else(|| format!("${}", name)),
            CssExpr::Binary(left, op, right) => {
                let l = left.to_string(variables);
                let r = right.to_string(variables);
                // Simple evaluation if both are numbers and have same unit
                if let (CssExpr::Number(ln, lu), CssExpr::Number(rn, ru)) = (&**left, &**right) {
                    if lu == ru || ru.is_empty() || lu.is_empty() {
                        let unit = if lu.is_empty() { ru } else { lu };
                        let res = match op.as_str() {
                            "+" => ln + rn,
                            "-" => ln - rn,
                            "*" => ln * rn,
                            "/" => {
                                if *rn != 0.0 {
                                    ln / rn
                                }
                                else {
                                    0.0
                                }
                            }
                            _ => 0.0,
                        };
                        return format!("{}{}", res, unit);
                    }
                }
                format!("{} {} {}", l, op, r)
            }
            CssExpr::Raw(s) => s.clone(),
        }
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
        let expr = self.parse_expression()?;
        self.state.cursor.skip_whitespace();
        if self.state.cursor.peek() == ';' {
            self.state.cursor.consume();
        }
        self.variables.insert(name, expr.to_string(&self.variables));
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
                let expr = self.parse_expression()?;
                self.state.cursor.skip_whitespace();
                if self.state.cursor.peek() == ';' {
                    self.state.cursor.consume();
                }

                let processed_val = expr.to_string(&self.variables);
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

    fn parse_expression(&mut self) -> Result<CssExpr> {
        self.parse_pratt_expr(0)
    }

    fn parse_pratt_expr(&mut self, min_precedence: i32) -> Result<CssExpr> {
        self.state.cursor.skip_whitespace();
        let mut left = self.parse_nud()?;

        loop {
            self.state.cursor.skip_whitespace();
            let op = self.peek_operator();
            if op.is_empty() {
                break;
            }

            let precedence = self.get_precedence(&op);
            if precedence <= min_precedence {
                break;
            }

            self.state.cursor.consume_n(op.len());
            let right = self.parse_pratt_expr(precedence)?;
            left = CssExpr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn parse_nud(&mut self) -> Result<CssExpr> {
        let c = self.state.cursor.peek();
        if c == '$' {
            self.state.cursor.consume();
            let name = self.consume_ident()?;
            Ok(CssExpr::Variable(name))
        }
        else if c.is_numeric() {
            let start = self.state.cursor.pos;
            while !self.state.cursor.is_eof() && (self.state.cursor.peek().is_numeric() || self.state.cursor.peek() == '.') {
                self.state.cursor.consume();
            }
            let n = self.state.cursor.current_str(start).parse().unwrap_or(0.0);
            let unit_start = self.state.cursor.pos;
            while !self.state.cursor.is_eof() && self.state.cursor.peek().is_alphabetic() {
                self.state.cursor.consume();
            }
            let unit = self.state.cursor.current_str(unit_start).to_string();
            Ok(CssExpr::Number(n, unit))
        }
        else if c == '(' {
            self.state.cursor.consume();
            let expr = self.parse_expression()?;
            self.state.cursor.expect(')')?;
            Ok(expr)
        }
        else {
            let start = self.state.cursor.pos;
            while !self.state.cursor.is_eof()
                && !";)}".contains(self.state.cursor.peek())
                && !"+-*/".contains(self.state.cursor.peek())
            {
                self.state.cursor.consume();
            }
            Ok(CssExpr::Raw(self.state.cursor.current_str(start).trim().to_string()))
        }
    }

    fn get_precedence(&self, op: &str) -> i32 {
        match op {
            "+" | "-" => 1,
            "*" | "/" => 2,
            _ => 0,
        }
    }

    fn peek_operator(&self) -> String {
        let c = self.state.cursor.peek();
        if "+-*/".contains(c) { c.to_string() } else { "".to_string() }
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
