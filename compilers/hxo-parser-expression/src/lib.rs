use hxo_ir::{JsExpr, JsProgram, JsStmt, TseAttribute};
use hxo_parser::{ParseState, ScriptParser};
use hxo_types::{Error, HxoValue, Result, Span, is_alphabetic};

pub struct ExprParser;

impl ScriptParser for ExprParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<JsProgram> {
        let mut parser = ExprParserImpl { state };
        parser.parse()
    }
}

pub fn parse_expression(source: &str) -> Result<JsExpr> {
    let mut state = ParseState::new(source);
    let mut parser = ExprParserImpl { state: &mut state };
    parser.parse_expr()
}

pub fn parse_program(source: &str) -> Result<JsProgram> {
    let mut state = ParseState::new(source);
    let mut parser = ExprParserImpl { state: &mut state };
    parser.parse()
}

struct ExprParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
}

impl<'a, 'b> ExprParserImpl<'a, 'b> {
    pub fn parse(&mut self) -> Result<JsProgram> {
        let mut body = Vec::new();
        let start_pos = self.state.cursor.position();

        while !self.state.cursor.is_eof() {
            self.state.cursor.skip_whitespace();
            if self.state.cursor.is_eof() {
                break;
            }

            if self.state.cursor.peek_str("import") {
                body.push(self.parse_import()?);
            }
            else if self.state.cursor.peek_str("export") {
                body.push(self.parse_export()?);
            }
            else if self.state.cursor.peek_str("const")
                || self.state.cursor.peek_str("let")
                || self.state.cursor.peek_str("var")
            {
                body.push(self.parse_variable_decl()?);
            }
            else if self.state.cursor.peek_str("function") {
                body.push(self.parse_function_decl()?);
            }
            else {
                body.push(self.parse_stmt()?);
            }

            self.state.cursor.skip_whitespace();
        }

        Ok(JsProgram { body, span: self.state.cursor.span_from(start_pos) })
    }

    fn parse_import(&mut self) -> Result<JsStmt> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.consume_str("import");
        self.state.cursor.skip_whitespace();

        let mut specifiers = Vec::new();
        if self.state.cursor.peek() == '{' {
            self.state.cursor.consume();
            while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
                self.state.cursor.skip_whitespace();
                specifiers.push(self.consume_ident()?);
                self.state.cursor.skip_whitespace();
                if self.state.cursor.peek() == ',' {
                    self.state.cursor.consume();
                }
            }
            self.state.cursor.expect('}')?;
        }
        else if self.state.cursor.peek() == '*' {
            self.state.cursor.consume();
            self.state.cursor.skip_whitespace();
            self.state.cursor.expect_str("as")?;
            self.state.cursor.skip_whitespace();
            specifiers.push(self.consume_ident()?);
        }
        else {
            specifiers.push(self.consume_ident()?);
        }

        self.state.cursor.skip_whitespace();
        self.state.cursor.expect_str("from")?;
        self.state.cursor.skip_whitespace();
        let source = self.consume_string()?;

        Ok(JsStmt::Import { source, specifiers, span: self.state.cursor.span_from(start_pos) })
    }

    fn parse_export(&mut self) -> Result<JsStmt> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.consume_str("export");
        self.state.cursor.skip_whitespace();

        if self.state.cursor.peek_str("default") {
            self.state.cursor.consume_str("default");
            self.state.cursor.skip_whitespace();
            let decl = if self.state.cursor.peek_str("function") {
                self.parse_function_decl()?
            }
            else {
                JsStmt::Expr(self.parse_expr()?, self.state.cursor.span_from(start_pos))
            };
            Ok(JsStmt::Export { declaration: Box::new(decl), span: self.state.cursor.span_from(start_pos) })
        }
        else {
            let decl = if self.state.cursor.peek_str("const")
                || self.state.cursor.peek_str("let")
                || self.state.cursor.peek_str("var")
            {
                self.parse_variable_decl()?
            }
            else {
                self.parse_function_decl()?
            };
            Ok(JsStmt::Export { declaration: Box::new(decl), span: self.state.cursor.span_from(start_pos) })
        }
    }

    fn parse_variable_decl(&mut self) -> Result<JsStmt> {
        let start_pos = self.state.cursor.position();
        let kind = if self.state.cursor.peek_str("const") {
            self.state.cursor.consume_str("const");
            "const"
        }
        else if self.state.cursor.peek_str("let") {
            self.state.cursor.consume_str("let");
            "let"
        }
        else {
            self.state.cursor.consume_str("var");
            "var"
        };

        self.state.cursor.skip_whitespace();

        // Handle destructuring like [count, setCount]
        let id = if self.state.cursor.peek() == '[' {
            let start = self.state.cursor.pos;
            self.state.cursor.consume();
            while !self.state.cursor.is_eof() && self.state.cursor.peek() != ']' {
                self.state.cursor.consume();
            }
            self.state.cursor.expect(']')?;
            self.state.cursor.current_str(start).to_string()
        }
        else if self.state.cursor.peek() == '{' {
            let start = self.state.cursor.pos;
            self.state.cursor.consume();
            while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
                self.state.cursor.consume();
            }
            self.state.cursor.expect('}')?;
            self.state.cursor.current_str(start).to_string()
        }
        else {
            self.consume_ident()?
        };

        self.state.cursor.skip_whitespace();
        let init = if self.state.cursor.peek() == '=' {
            self.state.cursor.consume();
            self.state.cursor.skip_whitespace();
            Some(self.parse_expr()?)
        }
        else {
            None
        };

        Ok(JsStmt::VariableDecl { kind: kind.to_string(), id, init, span: self.state.cursor.span_from(start_pos) })
    }

    fn parse_function_decl(&mut self) -> Result<JsStmt> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.consume_str("function");
        self.state.cursor.skip_whitespace();
        let id = if is_alphabetic(self.state.cursor.peek()) { self.consume_ident()? } else { "".to_string() };

        self.state.cursor.skip_whitespace();
        self.state.cursor.expect('(')?;
        let mut params = Vec::new();
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != ')' {
            self.state.cursor.skip_whitespace();
            params.push(self.consume_ident()?);
            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek() == ',' {
                self.state.cursor.consume();
            }
        }
        self.state.cursor.expect(')')?;

        self.state.cursor.skip_whitespace();
        self.state.cursor.expect('{')?;
        let mut body = Vec::new();
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
            body.push(self.parse_stmt()?);
            self.state.cursor.skip_whitespace();
        }
        self.state.cursor.expect('}')?;

        Ok(JsStmt::FunctionDecl { id, params, body, span: self.state.cursor.span_from(start_pos) })
    }

    fn parse_stmt(&mut self) -> Result<JsStmt> {
        let start_pos = self.state.cursor.position();

        // Try to parse as an expression statement if it looks like one
        let c = self.state.cursor.peek();
        if is_alphabetic(c) || c == '(' || c == '[' || c == '{' || c == '"' || c == '\'' || c.is_numeric() {
            let expr = self.parse_expr()?;
            self.state.cursor.skip_whitespace();
            if !self.state.cursor.is_eof() && self.state.cursor.peek() == ';' {
                self.state.cursor.consume();
            }
            return Ok(JsStmt::Expr(expr, self.state.cursor.span_from(start_pos)));
        }

        let start_offset = self.state.cursor.pos;

        // Very basic statement parsing: consume until semicolon or closing brace
        let mut brace_count = 0;
        while !self.state.cursor.is_eof() {
            let c = self.state.cursor.peek();
            if c == '{' {
                brace_count += 1;
            }
            else if c == '}' {
                if brace_count == 0 {
                    break;
                }
                brace_count -= 1;
            }
            else if c == ';' && brace_count == 0 {
                self.state.cursor.consume();
                break;
            }
            self.state.cursor.consume();
        }

        let content = self.state.cursor.current_str(start_offset).trim().to_string();
        let end_pos = self.state.cursor.position();

        Ok(JsStmt::Other(content, Span { start: start_pos, end: end_pos }))
    }

    fn parse_expr(&mut self) -> Result<JsExpr> {
        self.parse_pratt_expr(0)
    }

    fn parse_pratt_expr(&mut self, min_precedence: i32) -> Result<JsExpr> {
        self.state.cursor.skip_whitespace();
        let _start_pos = self.state.cursor.position();
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

            // If it's a binary operator that we handle in LED
            left = self.parse_led(left, &op, precedence)?;
        }

        Ok(left)
    }

    fn parse_nud(&mut self) -> Result<JsExpr> {
        let start_pos = self.state.cursor.position();
        let c = self.state.cursor.peek();

        // Prefix operators
        if c == '!' || (c == '-' && !self.state.cursor.peek_n(1).is_numeric()) {
            let op = self.state.cursor.consume().to_string();
            let argument = self.parse_pratt_expr(7)?; // Unary precedence
            return Ok(JsExpr::Unary { op, argument: Box::new(argument), span: self.state.cursor.span_from(start_pos) });
        }

        if self.state.cursor.peek_str("++") || self.state.cursor.peek_str("--") {
            let start = self.state.cursor.pos;
            self.state.cursor.consume_n(2);
            let op = self.state.cursor.current_str(start).to_string();
            let argument = self.parse_pratt_expr(7)?;
            return Ok(JsExpr::Unary { op, argument: Box::new(argument), span: self.state.cursor.span_from(start_pos) });
        }

        // Literals and Primary
        if c.is_numeric() {
            let start = self.state.cursor.pos;
            while !self.state.cursor.is_eof() && (self.state.cursor.peek().is_numeric() || self.state.cursor.peek() == '.') {
                self.state.cursor.consume();
            }
            let n = self.state.cursor.current_str(start).parse().unwrap_or(0.0);
            Ok(JsExpr::Literal(HxoValue::Number(n), self.state.cursor.span_from(start_pos)))
        }
        else if c == '"' || c == '\'' {
            let s = self.consume_string()?;
            Ok(JsExpr::Literal(HxoValue::String(s), self.state.cursor.span_from(start_pos)))
        }
        else if self.state.cursor.peek_str("true") {
            self.state.cursor.consume_n(4);
            Ok(JsExpr::Literal(HxoValue::Bool(true), self.state.cursor.span_from(start_pos)))
        }
        else if self.state.cursor.peek_str("false") {
            self.state.cursor.consume_n(5);
            Ok(JsExpr::Literal(HxoValue::Bool(false), self.state.cursor.span_from(start_pos)))
        }
        else if self.state.cursor.peek_str("null") {
            self.state.cursor.consume_n(4);
            Ok(JsExpr::Literal(HxoValue::Null, self.state.cursor.span_from(start_pos)))
        }
        else if is_alphabetic(c) {
            let id = self.consume_ident()?;
            Ok(JsExpr::Identifier(id, self.state.cursor.span_from(start_pos)))
        }
        else if c == '(' {
            self.state.cursor.consume();
            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek() == ')' {
                self.state.cursor.consume();
                Ok(JsExpr::Other("()".to_string(), self.state.cursor.span_from(start_pos)))
            }
            else {
                let e = self.parse_expr()?;
                self.state.cursor.expect(')')?;
                Ok(e)
            }
        }
        else if c == '[' {
            self.state.cursor.consume();
            let mut elements = Vec::new();
            while !self.state.cursor.is_eof() && self.state.cursor.peek() != ']' {
                self.state.cursor.skip_whitespace();
                elements.push(self.parse_expr()?);
                self.state.cursor.skip_whitespace();
                if self.state.cursor.peek() == ',' {
                    self.state.cursor.consume();
                }
            }
            self.state.cursor.expect(']')?;
            Ok(JsExpr::Array(elements, self.state.cursor.span_from(start_pos)))
        }
        else if c == '{' {
            self.state.cursor.consume();
            let mut props = std::collections::HashMap::new();
            while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
                self.state.cursor.skip_whitespace();
                let key = if self.state.cursor.peek() == '"' || self.state.cursor.peek() == '\'' {
                    self.consume_string()?
                }
                else {
                    self.consume_ident()?
                };
                self.state.cursor.skip_whitespace();
                self.state.cursor.expect(':')?;
                self.state.cursor.skip_whitespace();
                let val = self.parse_expr()?;
                props.insert(key, val);
                self.state.cursor.skip_whitespace();
                if self.state.cursor.peek() == ',' {
                    self.state.cursor.consume();
                }
            }
            self.state.cursor.expect('}')?;
            Ok(JsExpr::Object(props, self.state.cursor.span_from(start_pos)))
        }
        else if c == '<' {
            self.parse_tsx_element()
        }
        else {
            Err(Error::unexpected_char(c, self.state.cursor.span_at_current()))
        }
    }

    fn parse_led(&mut self, left: JsExpr, op: &str, precedence: i32) -> Result<JsExpr> {
        let start_pos = left.span().start;

        if op == "." {
            self.state.cursor.consume();
            let property = self.consume_ident()?;
            return Ok(JsExpr::Member {
                object: Box::new(left),
                property,
                computed: false,
                span: self.state.cursor.span_from(start_pos),
            });
        }

        if op == "[" {
            self.state.cursor.consume();
            let property_expr = self.parse_expr()?;
            self.state.cursor.expect(']')?;
            // Note: Our JsExpr::Member currently takes property: String.
            // For computed properties, we might need to store the expression or convert it to string if it's a literal.
            // For now, let's treat it as a string if it's a simple literal or identifier.
            let property = match &property_expr {
                JsExpr::Literal(HxoValue::String(s), _) => s.clone(),
                JsExpr::Identifier(id, _) => id.clone(),
                _ => "computed".to_string(), // Placeholder
            };
            return Ok(JsExpr::Member {
                object: Box::new(left),
                property,
                computed: true,
                span: self.state.cursor.span_from(start_pos),
            });
        }

        if op == "(" {
            self.state.cursor.consume();
            let mut args = Vec::new();
            while !self.state.cursor.is_eof() && self.state.cursor.peek() != ')' {
                self.state.cursor.skip_whitespace();
                args.push(self.parse_expr()?);
                self.state.cursor.skip_whitespace();
                if self.state.cursor.peek() == ',' {
                    self.state.cursor.consume();
                }
            }
            self.state.cursor.expect(')')?;
            return Ok(JsExpr::Call { callee: Box::new(left), args, span: self.state.cursor.span_from(start_pos) });
        }

        if op == "=>" {
            self.state.cursor.consume_n(2);
            self.state.cursor.skip_whitespace();
            let body = self.parse_expr()?;
            let params = match left {
                JsExpr::Identifier(id, _) => vec![id],
                JsExpr::Other(s, _) if s == "()" => vec![],
                _ => {
                    return Err(Error::parse_error(
                        "Invalid arrow function parameters".to_string(),
                        self.state.cursor.span_from(start_pos),
                    ));
                }
            };
            return Ok(JsExpr::ArrowFunction { params, body: Box::new(body), span: self.state.cursor.span_from(start_pos) });
        }

        // Binary operators
        self.state.cursor.consume_n(op.chars().count());
        self.state.cursor.skip_whitespace();

        // Right associativity for assignment
        let next_min_precedence = if op == "=" { precedence - 1 } else { precedence };
        let right = self.parse_pratt_expr(next_min_precedence)?;

        Ok(JsExpr::Binary {
            left: Box::new(left),
            op: op.to_string(),
            right: Box::new(right),
            span: self.state.cursor.span_from(start_pos),
        })
    }

    fn get_precedence(&self, op: &str) -> i32 {
        match op {
            "=" => 1,
            "||" => 2,
            "&&" => 3,
            "==" | "!=" | "===" | "!==" => 4,
            "<" | ">" | "<=" | ">=" => 5,
            "+" | "-" => 6,
            "*" | "/" | "%" => 7,
            "." | "(" | "[" | "=>" => 10,
            _ => 0,
        }
    }

    fn peek_operator(&self) -> String {
        let s = &self.state.cursor.source[self.state.cursor.pos..];
        if s.starts_with("===") || s.starts_with("!==") {
            return s[..3].to_string();
        }
        if s.starts_with("==")
            || s.starts_with("!=")
            || s.starts_with("<=")
            || s.starts_with(">=")
            || s.starts_with("&&")
            || s.starts_with("||")
            || s.starts_with("=>")
        {
            return s[..2].to_string();
        }
        let c = s.chars().next().unwrap_or('\0');
        if "+-*/%.()[]=".contains(c) {
            return c.to_string();
        }
        "".to_string()
    }

    fn consume_ident(&mut self) -> Result<String> {
        self.state.cursor.consume_ident()
    }

    fn consume_string(&mut self) -> Result<String> {
        self.state.cursor.consume_string()
    }

    fn parse_tsx_element(&mut self) -> Result<JsExpr> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.expect('<')?;
        let tag = self.state.cursor.consume_while(|c| c.is_alphanumeric() || c == '-');
        let mut attributes = Vec::new();

        self.state.cursor.skip_whitespace();
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '>' && !self.state.cursor.peek_str("/>") {
            let attr_start = self.state.cursor.position();
            let attr_name = self.state.cursor.consume_while(|c| !c.is_whitespace() && c != '=' && c != '>' && c != '/');
            self.state.cursor.skip_whitespace();
            let attr_value = if self.state.cursor.peek() == '=' {
                self.state.cursor.consume();
                self.state.cursor.skip_whitespace();
                if self.state.cursor.peek() == '{' {
                    self.state.cursor.consume();
                    let expr = self.parse_expr()?;
                    self.state.cursor.expect('}')?;
                    Some(expr)
                }
                else {
                    let s = self.consume_string()?;
                    Some(JsExpr::Literal(HxoValue::String(s), self.state.cursor.span_from(attr_start)))
                }
            }
            else {
                None
            };
            attributes.push(TseAttribute { name: attr_name, value: attr_value, span: self.state.cursor.span_from(attr_start) });
            self.state.cursor.skip_whitespace();
        }

        let is_self_closing = if self.state.cursor.peek_str("/>") {
            self.state.cursor.consume_n(2);
            true
        }
        else {
            self.state.cursor.expect('>')?;
            false
        };

        let mut children = Vec::new();
        if !is_self_closing {
            while !self.state.cursor.is_eof() && !self.state.cursor.peek_str("</") {
                if self.state.cursor.peek() == '{' {
                    self.state.cursor.consume();
                    children.push(self.parse_expr()?);
                    self.state.cursor.expect('}')?;
                }
                else if self.state.cursor.peek() == '<' {
                    children.push(self.parse_tsx_element()?);
                }
                else {
                    let start = self.state.cursor.pos;
                    while !self.state.cursor.is_eof() && self.state.cursor.peek() != '<' && self.state.cursor.peek() != '{' {
                        self.state.cursor.consume();
                    }
                    let text = self.state.cursor.current_str(start).to_string();
                    if !text.trim().is_empty() {
                        children.push(JsExpr::Literal(HxoValue::String(text), self.state.cursor.span_from(start_pos)));
                    }
                }
                self.state.cursor.skip_whitespace();
            }
            self.state.cursor.expect_str("</")?;
            self.state.cursor.consume_str(&tag);
            self.state.cursor.expect('>')?;
        }

        Ok(JsExpr::TseElement { tag, attributes, children, span: self.state.cursor.span_from(start_pos) })
    }
}
