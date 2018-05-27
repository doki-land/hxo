use hxo_parser::{ParseState, StyleParser};
use hxo_types::Result;
use std::collections::HashMap;

pub struct LessParser;

impl StyleParser for LessParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<String> {
        let mut parser = LessParserImpl { state, variables: HashMap::new() };
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
            CssExpr::Variable(name) => variables.get(name).cloned().unwrap_or_else(|| format!("@{}", name)),
            CssExpr::Binary(left, op, right) => {
                let l = left.to_string(variables);
                let r = right.to_string(variables);
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
            else if self.is_rule() {
                css.push_str(&self.parse_rule(&mut selector_stack)?);
            }
            else {
                self.state.cursor.consume();
            }
        }

        Ok(css.trim().to_string())
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
        if c == '@' {
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

    fn is_rule(&self) -> bool {
        let mut pos = self.state.cursor.pos;
        let bytes = self.state.cursor.source.as_bytes();
        while pos < bytes.len() {
            let c = bytes[pos] as char;
            if c == '{' {
                return true;
            }
            if c == ';' || c == '}' {
                return false;
            }
            pos += 1;
        }
        false
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

    fn consume_until_char(&mut self, target: char) -> Result<String> {
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != target {
            self.state.cursor.consume();
        }
        Ok(self.state.cursor.source[start..self.state.cursor.pos].to_string())
    }

    fn parse_variable(&mut self) -> Result<()> {
        self.state.cursor.consume(); // @
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
        self.state.cursor.skip_whitespace();
        let selector = self.consume_until_char('{')?.trim().to_string();
        self.state.cursor.expect('{')?;

        selector_stack.push(selector.clone());
        let full_selector = selector_stack.join(" ");

        let mut css = String::new();
        let mut declarations = Vec::new();

        self.skip_comments_and_whitespace();
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
            if self.is_rule() {
                // Nested rule
                css.push_str(&self.parse_rule(selector_stack)?);
            }
            else if self.state.cursor.peek() != '}' {
                // Declaration
                let prop = self.consume_until_char(':')?.trim().to_string();
                if self.state.cursor.peek() == ':' {
                    self.state.cursor.expect(':')?;
                    self.state.cursor.skip_whitespace();
                    let expr = self.parse_expression()?;
                    self.state.cursor.skip_whitespace();
                    if self.state.cursor.peek() == ';' {
                        self.state.cursor.consume();
                    }

                    let val = expr.to_string(&self.variables);
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
