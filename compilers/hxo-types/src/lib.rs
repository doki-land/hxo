pub mod errors;

pub use errors::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn unknown() -> Self {
        Self { start: Position::unknown(), end: Position::unknown() }
    }

    pub fn is_unknown(&self) -> bool {
        self.start.is_unknown() && self.end.is_unknown()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Position {
    pub line: u32,
    pub column: u32,
    pub offset: u32,
}

impl Position {
    pub fn unknown() -> Self {
        Self { line: 0, column: 0, offset: 0 }
    }

    pub fn is_unknown(&self) -> bool {
        self.line == 0 && self.column == 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HxoFile {
    pub blocks: Vec<HxoBlock>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HxoBlock {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub content: String,
    pub span: Span,
    pub content_span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HxoValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<HxoValue>),
    Object(HashMap<String, HxoValue>),
    /// 响应式信号引用
    Signal(String),
    /// 二进制数据 (如 WASM)
    Binary(Vec<u8>),
    /// 源码片段或表达式代码
    Raw(String),
    /// 跨节点引用或 ID
    Ref(String),
}

impl HxoValue {
    pub fn get(&self, key: &str) -> Option<&HxoValue> {
        match self {
            HxoValue::Object(map) => map.get(key),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<HxoValue>> {
        match self {
            HxoValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub routes: Vec<Route>,
    pub mode: String,
    pub base: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub path: String,
    pub component: String,
    pub name: Option<String>,
    pub redirect: Option<String>,
    pub children: Option<Vec<Route>>,
    pub meta: Option<HxoValue>,
    pub span: Span,
}

pub fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

pub fn is_pos_in_span(pos: Position, span: Span) -> bool {
    if span.is_unknown() {
        return false;
    }

    if pos.line < span.start.line || pos.line > span.end.line {
        return false;
    }

    if pos.line == span.start.line && pos.column < span.start.column {
        return false;
    }

    if pos.line == span.end.line && pos.column > span.end.column {
        return false;
    }

    true
}

pub fn is_alphabetic(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

pub fn is_alphanumeric(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

impl HxoValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            HxoValue::String(s) => Some(s),
            HxoValue::Signal(s) => Some(s),
            HxoValue::Raw(s) => Some(s),
            HxoValue::Ref(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            HxoValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            HxoValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            HxoValue::Binary(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, HxoValue>> {
        match self {
            HxoValue::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn is_json_compatible(&self) -> bool {
        match self {
            HxoValue::Null
            | HxoValue::Bool(_)
            | HxoValue::Number(_)
            | HxoValue::String(_)
            | HxoValue::Array(_)
            | HxoValue::Object(_) => true,
            HxoValue::Signal(_) | HxoValue::Binary(_) | HxoValue::Raw(_) | HxoValue::Ref(_) => false,
        }
    }
}

pub struct Cursor<'a> {
    pub source: &'a str,
    pub pos: usize,
    pub line: usize,
    pub column: usize,
    pub base_offset: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, pos: 0, line: 1, column: 1, base_offset: 0 }
    }

    pub fn with_position(source: &'a str, pos: Position) -> Self {
        Self { source, pos: pos.offset as usize, line: pos.line as usize, column: pos.column as usize, base_offset: 0 }
    }

    pub fn with_sliced_source(source: &'a str, pos: Position) -> Self {
        Self { source, pos: 0, line: pos.line as usize, column: pos.column as usize, base_offset: pos.offset as usize }
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    pub fn peek(&self) -> char {
        self.source[self.pos..].chars().next().unwrap_or('\0')
    }

    pub fn peek_n(&self, n: usize) -> char {
        self.source[self.pos..].chars().nth(n).unwrap_or('\0')
    }

    pub fn peek_str(&self, s: &str) -> bool {
        self.source[self.pos..].starts_with(s)
    }

    pub fn consume(&mut self) -> char {
        let c = self.peek();
        if c == '\0' {
            return c;
        }
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        }
        else {
            self.column += c.len_utf16();
        }
        c
    }

    pub fn consume_n(&mut self, n: usize) {
        for _ in 0..n {
            self.consume();
        }
    }

    pub fn consume_str(&mut self, s: &str) -> bool {
        if self.peek_str(s) {
            self.consume_n(s.chars().count());
            true
        }
        else {
            false
        }
    }

    pub fn consume_while<F>(&mut self, f: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let start = self.pos;
        while !self.is_eof() && f(self.peek()) {
            self.consume();
        }
        self.current_str(start).to_string()
    }

    pub fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.peek().is_whitespace() {
            self.consume();
        }
    }

    pub fn skip_spaces(&mut self) {
        while !self.is_eof() && (self.peek() == ' ' || self.peek() == '\t') {
            self.consume();
        }
    }

    pub fn expect(&mut self, expected: char) -> Result<()> {
        if self.peek() == expected {
            self.consume();
            Ok(())
        }
        else {
            Err(Error::expected_char(expected, self.peek(), self.span_at_current()))
        }
    }

    pub fn expect_str(&mut self, expected: &str) -> Result<()> {
        if self.peek_str(expected) {
            self.consume_n(expected.chars().count());
            Ok(())
        }
        else {
            Err(Error::expected_string(expected.to_string(), self.peek().to_string(), self.span_at_current()))
        }
    }

    pub fn position(&self) -> Position {
        Position { line: self.line as u32, column: self.column as u32, offset: (self.base_offset + self.pos) as u32 }
    }

    pub fn current_str(&self, start: usize) -> &str {
        &self.source[start..self.pos]
    }

    pub fn span_at_current(&self) -> Span {
        let start = self.position();
        let mut end = start;
        let c = self.peek();
        if c != '\0' {
            end.column += c.len_utf16() as u32;
            end.offset += c.len_utf8() as u32;
        }
        Span { start, end }
    }

    pub fn span_from(&self, start: Position) -> Span {
        Span { start, end: self.position() }
    }

    pub fn consume_ident(&mut self) -> Result<String> {
        let start = self.pos;
        if !is_alphabetic(self.peek()) {
            return Err(Error::parse_error("Expected identifier".to_string(), self.span_at_current()));
        }
        while !self.is_eof() && is_alphanumeric(self.peek()) {
            self.consume();
        }
        Ok(self.current_str(start).to_string())
    }

    pub fn consume_string(&mut self) -> Result<String> {
        let quote = self.peek();
        if quote != '"' && quote != '\'' {
            return Err(Error::parse_error("Expected string".to_string(), self.span_at_current()));
        }
        self.consume();
        let start = self.pos;
        while !self.is_eof() && self.peek() != quote {
            self.consume();
        }
        let s = self.current_str(start).to_string();
        self.expect(quote)?;
        Ok(s)
    }

    pub fn span_at_pos(&self, pos: Position) -> Span {
        Span { start: pos, end: pos }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CodeWriter {
    buffer: String,
    indent_level: usize,
    mappings: Vec<(Position, Span)>,
    current_pos: Position,
}

impl CodeWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, text: &str) {
        if self.buffer.is_empty() || self.buffer.ends_with('\n') {
            let indent = "  ".repeat(self.indent_level);
            self.buffer.push_str(&indent);
            self.current_pos.column += indent.len() as u32;
            self.current_pos.offset += indent.len() as u32;
        }

        self.buffer.push_str(text);
        let lines: Vec<&str> = text.split('\n').collect();
        if lines.len() > 1 {
            self.current_pos.line += (lines.len() - 1) as u32;
            self.current_pos.column = lines.last().unwrap().len() as u32;
        }
        else {
            self.current_pos.column += text.len() as u32;
        }
        self.current_pos.offset += text.len() as u32;
    }

    pub fn write_with_span(&mut self, text: &str, span: Span) {
        if !span.is_unknown() {
            self.mappings.push((self.current_pos, span));
        }
        self.write(text);
    }

    pub fn write_line(&mut self, text: &str) {
        self.write(text);
        self.newline();
    }

    pub fn write_line_with_span(&mut self, text: &str, span: Span) {
        self.write_with_span(text, span);
        self.newline();
    }

    pub fn newline(&mut self) {
        self.buffer.push('\n');
        self.current_pos.line += 1;
        self.current_pos.column = 0;
        self.current_pos.offset += 1;
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn position(&self) -> Position {
        self.current_pos
    }

    pub fn append(&mut self, other: CodeWriter) {
        let (other_buf, other_mappings) = other.finish();
        for (mut pos, span) in other_mappings {
            pos.line += self.current_pos.line;
            if pos.line == self.current_pos.line {
                pos.column += self.current_pos.column;
            }
            pos.offset += self.current_pos.offset;
            self.mappings.push((pos, span));
        }
        self.write(&other_buf);
    }

    pub fn finish(self) -> (String, Vec<(Position, Span)>) {
        (self.buffer, self.mappings)
    }
}
