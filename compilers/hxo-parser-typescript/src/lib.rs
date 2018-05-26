use hxo_ir::{JsExpr, JsProgram, JsStmt, TseAttribute};
use hxo_parser::{ParseState, ScriptSubParser};
use hxo_types::{Error, HxoValue, Result, Span};

pub struct JsTsParser;

pub fn parse_expression(source: &str) -> Result<JsExpr> {
    let mut parser = ExpressionParser::new(source);
    parser.parse()
}

pub fn parse_script(source: &str, is_ts: bool) -> Result<JsProgram> {
    let mut state = ParseState::new(source);
    let mut parser = JsTsParserImpl { state: &mut state };
    parser.parse(is_ts)
}

pub struct ExpressionParser<'a> {
    state: ParseState<'a>,
}

impl<'a> ExpressionParser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { state: ParseState::new(source) }
    }

    pub fn parse(&mut self) -> Result<JsExpr> {
        let mut parser = JsTsParserImpl { state: &mut self.state };
        parser.parse_expr()
    }
}

impl ScriptSubParser for JsTsParser {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<JsProgram> {
        let mut parser = JsTsParserImpl { state };
        parser.parse(lang == "ts")
    }
}

struct JsTsParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
}

impl<'a, 'b> JsTsParserImpl<'a, 'b> {
    pub fn parse(&mut self, _is_ts: bool) -> Result<JsProgram> {
        let start_pos = self.state.cursor.position();
        let body = self.parse_statements()?;
        let end_pos = self.state.cursor.position();

        Ok(JsProgram { body, span: Span { start: start_pos, end: end_pos } })
    }

    fn parse_statements(&mut self) -> Result<Vec<JsStmt>> {
        let mut body = Vec::new();
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
            self.state.cursor.skip_whitespace();
            if self.state.cursor.is_eof() || self.state.cursor.peek() == '}' {
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
        Ok(body)
    }

    fn parse_import(&mut self) -> Result<JsStmt> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.expect_str("import")?;
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
        else {
            specifiers.push(self.consume_ident()?);
        }

        self.state.cursor.skip_whitespace();
        self.state.cursor.expect_str("from")?;
        self.state.cursor.skip_whitespace();
        let source = self.consume_string()?;
        if self.state.cursor.peek() == ';' {
            self.state.cursor.consume();
        }

        Ok(JsStmt::Import { source, specifiers, span: self.state.cursor.span_from(start_pos) })
    }

    fn parse_export(&mut self) -> Result<JsStmt> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.expect_str("export")?;
        self.state.cursor.skip_whitespace();

        if self.state.cursor.peek() == '*' {
            self.state.cursor.consume();
            self.state.cursor.skip_whitespace();
            self.state.cursor.expect_str("from")?;
            self.state.cursor.skip_whitespace();
            let source = self.consume_string()?;
            if self.state.cursor.peek() == ';' {
                self.state.cursor.consume();
            }
            return Ok(JsStmt::ExportAll { source, span: self.state.cursor.span_from(start_pos) });
        }

        if self.state.cursor.peek() == '{' {
            self.state.cursor.consume();
            let mut specifiers = Vec::new();
            while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
                self.state.cursor.skip_whitespace();
                specifiers.push(self.consume_ident()?);
                self.state.cursor.skip_whitespace();
                if self.state.cursor.peek() == ',' {
                    self.state.cursor.consume();
                }
            }
            self.state.cursor.expect('}')?;
            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek_str("from") {
                self.state.cursor.expect_str("from")?;
                self.state.cursor.skip_whitespace();
                let source = self.consume_string()?;
                if self.state.cursor.peek() == ';' {
                    self.state.cursor.consume();
                }
                return Ok(JsStmt::ExportNamed {
                    source: Some(source),
                    specifiers,
                    span: self.state.cursor.span_from(start_pos),
                });
            }

            if self.state.cursor.peek() == ';' {
                self.state.cursor.consume();
            }
            return Ok(JsStmt::ExportNamed { source: None, specifiers, span: self.state.cursor.span_from(start_pos) });
        }

        let declaration =
            if self.state.cursor.peek_str("const") || self.state.cursor.peek_str("let") || self.state.cursor.peek_str("var") {
                Box::new(self.parse_variable_decl()?)
            }
            else if self.state.cursor.peek_str("function") {
                Box::new(self.parse_function_decl()?)
            }
            else {
                let s_start = self.state.cursor.position();
                Box::new(JsStmt::Other(self.consume_until_semicolon()?, self.state.cursor.span_from(s_start)))
            };
        Ok(JsStmt::Export { declaration, span: self.state.cursor.span_from(start_pos) })
    }

    fn parse_variable_decl(&mut self) -> Result<JsStmt> {
        let start_pos = self.state.cursor.position();
        let kind = if self.state.cursor.peek_str("const") {
            self.state.cursor.expect_str("const")?;
            "const"
        }
        else if self.state.cursor.peek_str("let") {
            self.state.cursor.expect_str("let")?;
            "let"
        }
        else {
            self.state.cursor.expect_str("var")?;
            "var"
        }
        .to_string();

        self.state.cursor.skip_whitespace();
        let id = self.consume_pattern()?;

        // Handle TS type annotation if any
        if self.state.cursor.peek() == ':' {
            self.state.cursor.consume();
            self.consume_type_annotation()?;
        }

        self.state.cursor.skip_whitespace();
        let init = if self.state.cursor.peek() == '=' {
            self.state.cursor.consume();
            self.state.cursor.skip_whitespace();
            Some(self.parse_expr()?)
        }
        else {
            None
        };

        if self.state.cursor.peek() == ';' {
            self.state.cursor.consume();
        }

        Ok(JsStmt::VariableDecl { kind, id, init, span: self.state.cursor.span_from(start_pos) })
    }

    fn parse_function_decl(&mut self) -> Result<JsStmt> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.expect_str("function")?;
        self.state.cursor.skip_whitespace();
        let id = self.consume_ident()?;
        self.state.cursor.skip_whitespace();

        // Handle TS type parameters
        if self.state.cursor.peek() == '<' {
            self.consume_type_parameters()?;
            self.state.cursor.skip_whitespace();
        }

        self.state.cursor.expect('(')?;
        let mut params = Vec::new();
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != ')' {
            self.state.cursor.skip_whitespace();
            params.push(self.consume_ident()?);
            // Handle TS type
            if self.state.cursor.peek() == ':' {
                self.state.cursor.consume();
                self.consume_type_annotation()?;
            }
            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek() == ',' {
                self.state.cursor.consume();
            }
        }
        self.state.cursor.expect(')')?;

        // Handle TS return type
        if self.state.cursor.peek() == ':' {
            self.state.cursor.consume();
            self.consume_type_annotation()?;
        }

        self.state.cursor.skip_whitespace();
        self.state.cursor.expect('{')?;
        let body = self.parse_statements()?;
        self.state.cursor.expect('}')?;
        let end_pos = self.state.cursor.position();

        Ok(JsStmt::FunctionDecl { id, params, body, span: Span { start: start_pos, end: end_pos } })
    }

    fn parse_stmt(&mut self) -> Result<JsStmt> {
        let start_pos = self.state.cursor.position();
        if self.state.cursor.peek() == '{' {
            self.state.cursor.consume();
            let mut depth = 1;
            let start = self.state.cursor.pos;
            while !self.state.cursor.is_eof() && depth > 0 {
                if self.state.cursor.peek() == '{' {
                    depth += 1;
                }
                else if self.state.cursor.peek() == '}' {
                    depth -= 1;
                }
                if depth > 0 {
                    self.state.cursor.consume();
                }
            }
            let content = self.state.cursor.current_str(start).to_string();
            self.state.cursor.expect('}')?;
            let end_pos = self.state.cursor.position();
            return Ok(JsStmt::Other(content, Span { start: start_pos, end: end_pos }));
        }

        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != ';' && self.state.cursor.peek() != '\n' {
            self.state.cursor.consume();
        }
        let content = self.state.cursor.current_str(start).to_string();
        if self.state.cursor.peek() == ';' {
            self.state.cursor.consume();
        }
        let end_pos = self.state.cursor.position();
        Ok(JsStmt::Other(content, Span { start: start_pos, end: end_pos }))
    }

    fn parse_expr(&mut self) -> Result<JsExpr> {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Result<JsExpr> {
        let start_pos = self.state.cursor.position();
        let left = self.parse_binary_expr(0)?;
        self.state.cursor.skip_whitespace();
        if self.state.cursor.peek() == '=' && self.state.cursor.peek_n(1) != '=' {
            self.state.cursor.consume();
            self.state.cursor.skip_whitespace();
            let right = self.parse_assignment_expr()?;
            let end_pos = self.state.cursor.position();
            return Ok(JsExpr::Binary {
                left: Box::new(left),
                op: "=".to_string(),
                right: Box::new(right),
                span: Span { start: start_pos, end: end_pos },
            });
        }
        Ok(left)
    }

    fn get_precedence(&self, op: &str) -> i32 {
        match op {
            "||" => 1,
            "&&" => 2,
            "==" | "!=" | "===" | "!==" => 3,
            "<" | ">" | "<=" | ">=" => 4,
            "+" | "-" => 5,
            "*" | "/" | "%" => 6,
            _ => 0,
        }
    }

    fn parse_binary_expr(&mut self, min_precedence: i32) -> Result<JsExpr> {
        let start_pos = self.state.cursor.position();
        let mut left = self.parse_primary_expr()?;

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
            self.state.cursor.skip_whitespace();
            let right = self.parse_binary_expr(precedence)?;
            let end_pos = self.state.cursor.position();
            left = JsExpr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: Span { start: start_pos, end: end_pos },
            };
        }

        Ok(left)
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
        {
            return s[..2].to_string();
        }
        let c = self.state.cursor.peek();
        if "+-*/%<>=!".contains(c) {
            return c.to_string();
        }
        "".to_string()
    }

    fn parse_primary_expr(&mut self) -> Result<JsExpr> {
        let start_pos = self.state.cursor.position();
        let mut expr = if self.state.cursor.peek().is_numeric() {
            let start = self.state.cursor.pos;
            while !self.state.cursor.is_eof() && (self.state.cursor.peek().is_numeric() || self.state.cursor.peek() == '.') {
                self.state.cursor.consume();
            }
            let n = self.state.cursor.current_str(start).parse().unwrap_or(0.0);
            JsExpr::Literal(HxoValue::Number(n), self.state.cursor.span_from(start_pos))
        }
        else if self.state.cursor.peek() == '"' || self.state.cursor.peek() == '\'' {
            let s = self.consume_string()?;
            JsExpr::Literal(HxoValue::String(s), self.state.cursor.span_from(start_pos))
        }
        else if self.state.cursor.peek_str("true") {
            self.state.cursor.consume_n(4);
            JsExpr::Literal(HxoValue::Bool(true), self.state.cursor.span_from(start_pos))
        }
        else if self.state.cursor.peek_str("false") {
            self.state.cursor.consume_n(5);
            JsExpr::Literal(HxoValue::Bool(false), self.state.cursor.span_from(start_pos))
        }
        else if self.state.cursor.peek_str("null") {
            self.state.cursor.consume_n(4);
            JsExpr::Literal(HxoValue::Null, self.state.cursor.span_from(start_pos))
        }
        else if hxo_types::is_alphabetic(self.state.cursor.peek()) {
            let id = self.consume_ident()?;
            JsExpr::Identifier(id, self.state.cursor.span_from(start_pos))
        }
        else if self.state.cursor.peek() == '(' {
            self.state.cursor.consume();
            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek() == ')' {
                self.state.cursor.consume();
                JsExpr::Other("()".to_string(), self.state.cursor.span_from(start_pos))
            }
            else {
                let e = self.parse_expr()?;
                self.state.cursor.expect(')')?;
                e
            }
        }
        else if self.state.cursor.peek() == '[' {
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
            JsExpr::Array(elements, self.state.cursor.span_from(start_pos))
        }
        else if self.state.cursor.peek() == '{' {
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
            JsExpr::Object(props, self.state.cursor.span_from(start_pos))
        }
        else if self.state.cursor.peek() == '<' {
            self.parse_tsx_element()?
        }
        else {
            return Err(Error::unexpected_char(self.state.cursor.peek(), self.state.cursor.span_at_current()));
        };

        // Post-primary: Member access, Call, Arrow function
        loop {
            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek() == '.' {
                self.state.cursor.consume();
                let prop = self.consume_ident()?;
                expr = JsExpr::Member {
                    object: Box::new(expr),
                    property: prop,
                    computed: false,
                    span: self.state.cursor.span_from(start_pos),
                };
            }
            else if self.state.cursor.peek() == '(' {
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
                expr = JsExpr::Call { callee: Box::new(expr), args, span: self.state.cursor.span_from(start_pos) };
            }
            else if self.state.cursor.peek_str("=>") {
                if let JsExpr::Identifier(id, _span) = expr {
                    self.state.cursor.consume_n(2);
                    self.state.cursor.skip_whitespace();
                    let body = self.parse_expr()?;
                    expr = JsExpr::ArrowFunction {
                        params: vec![id],
                        body: Box::new(body),
                        span: self.state.cursor.span_from(start_pos),
                    };
                }
                else if let JsExpr::Other(s, _span) = &expr {
                    if s == "()" {
                        self.state.cursor.consume_n(2);
                        self.state.cursor.skip_whitespace();
                        let body = self.parse_expr()?;
                        expr = JsExpr::ArrowFunction {
                            params: vec![],
                            body: Box::new(body),
                            span: self.state.cursor.span_from(start_pos),
                        };
                    }
                    else {
                        break;
                    }
                }
                else {
                    break;
                }
            }
            else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_tsx_element(&mut self) -> Result<JsExpr> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.expect('<')?;
        let tag = self.consume_ident()?;
        let mut attributes = Vec::new();

        loop {
            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek() == '>' || self.state.cursor.peek_str("/>") {
                break;
            }
            let attr_start = self.state.cursor.position();
            let name = self.consume_ident()?;
            self.state.cursor.skip_whitespace();
            let value = if self.state.cursor.peek() == '=' {
                self.state.cursor.consume();
                self.state.cursor.skip_whitespace();
                if self.state.cursor.peek() == '{' {
                    self.state.cursor.consume();
                    let expr = self.parse_expr()?;
                    self.state.cursor.expect('}')?;
                    Some(expr)
                }
                else {
                    let s_start = self.state.cursor.position();
                    Some(JsExpr::Literal(HxoValue::String(self.consume_string()?), self.state.cursor.span_from(s_start)))
                }
            }
            else {
                None
            };
            attributes.push(TseAttribute { name, value, span: self.state.cursor.span_from(attr_start) });
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
                    let s_start = self.state.cursor.position();
                    let mut text = String::new();
                    while !self.state.cursor.is_eof() && self.state.cursor.peek() != '<' && self.state.cursor.peek() != '{' {
                        text.push(self.state.cursor.consume());
                    }
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        children.push(JsExpr::Literal(HxoValue::String(text), self.state.cursor.span_from(s_start)));
                    }
                }
                self.state.cursor.skip_whitespace();
            }
            self.state.cursor.expect_str("</")?;
            self.state.cursor.expect_str(&tag)?;
            self.state.cursor.expect('>')?;
        }

        Ok(JsExpr::TseElement { tag, attributes, children, span: self.state.cursor.span_from(start_pos) })
    }

    fn consume_ident(&mut self) -> Result<String> {
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && hxo_types::is_alphanumeric(self.state.cursor.peek()) {
            self.state.cursor.consume();
        }
        Ok(self.state.cursor.current_str(start).to_string())
    }

    fn consume_pattern(&mut self) -> Result<String> {
        let start = self.state.cursor.pos;
        let first = self.state.cursor.peek();
        if first == '[' || first == '{' {
            let open = first;
            let close = if open == '[' { ']' } else { '}' };
            let mut depth = 0;
            while !self.state.cursor.is_eof() {
                let c = self.state.cursor.peek();
                if c == open {
                    depth += 1;
                }
                else if c == close {
                    depth -= 1;
                }
                self.state.cursor.consume();
                if depth == 0 {
                    break;
                }
            }
            Ok(self.state.cursor.current_str(start).to_string())
        }
        else {
            self.consume_ident()
        }
    }

    fn consume_string(&mut self) -> Result<String> {
        let quote = self.state.cursor.consume();
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != quote {
            self.state.cursor.consume();
        }
        let s = self.state.cursor.current_str(start).to_string();
        self.state.cursor.expect(quote)?;
        Ok(s)
    }

    fn consume_until_semicolon(&mut self) -> Result<String> {
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != ';' {
            self.state.cursor.consume();
        }
        let s = self.state.cursor.current_str(start).to_string();
        if self.state.cursor.peek() == ';' {
            self.state.cursor.consume();
        }
        Ok(s)
    }

    fn consume_type_parameters(&mut self) -> Result<()> {
        self.state.cursor.expect('<')?;
        let mut depth = 1;
        while !self.state.cursor.is_eof() && depth > 0 {
            let c = self.state.cursor.peek();
            if c == '<' {
                depth += 1;
            }
            else if c == '>' {
                depth -= 1;
            }
            self.state.cursor.consume();
        }
        Ok(())
    }

    fn consume_type_annotation(&mut self) -> Result<()> {
        self.state.cursor.skip_whitespace();
        let mut depth = 0;
        while !self.state.cursor.is_eof() {
            let c = self.state.cursor.peek();
            if c == '<' || c == '[' || c == '(' {
                depth += 1;
                self.state.cursor.consume();
            }
            else if c == '>' || c == ']' || c == ')' {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                self.state.cursor.consume();
            }
            else if depth > 0 {
                self.state.cursor.consume();
            }
            else if hxo_types::is_alphanumeric(c)
                || c == ' '
                || c == '|'
                || c == '&'
                || c == ','
                || c == '.'
                || c == '_'
                || c == '$'
            {
                self.state.cursor.consume();
            }
            else {
                break;
            }
        }
        Ok(())
    }
}
