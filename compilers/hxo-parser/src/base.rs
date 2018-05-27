pub use hxo_types::{Cursor, Error, ErrorKind, Position};
use std::collections::HashMap;

pub struct ParseState<'a> {
    pub cursor: Cursor<'a>,
}

impl<'a> ParseState<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { cursor: Cursor::new(source) }
    }

    pub fn new_with_pos(source: &'a str, pos: Position) -> Self {
        Self { cursor: Cursor::with_position(source, pos) }
    }

    pub fn with_cursor(cursor: Cursor<'a>) -> Self {
        Self { cursor }
    }

    pub fn unexpected_char(&self) -> Error {
        Error::new(ErrorKind::UnexpectedChar { character: self.cursor.peek(), span: self.cursor.span_at_current() })
    }

    pub fn expected_char(&self, expected: char) -> Error {
        Error::new(ErrorKind::ExpectedChar { expected, found: self.cursor.peek(), span: self.cursor.span_at_current() })
    }

    pub fn expected_string(&self, expected: &str) -> Error {
        Error::new(ErrorKind::ExpectedString {
            expected: expected.to_string(),
            found: self.cursor.peek().to_string(),
            span: self.cursor.span_at_current(),
        })
    }

    pub fn error(&self, message: impl Into<String>) -> Error {
        Error::new(ErrorKind::Parse { message: message.into(), span: self.cursor.span_at_current() })
    }

    pub fn parse_tag_attributes(&mut self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        self.cursor.skip_whitespace();
        while !self.cursor.is_eof() && self.cursor.peek() != '>' {
            let start = self.cursor.pos;
            while !self.cursor.is_eof()
                && !self.cursor.peek().is_whitespace()
                && self.cursor.peek() != '='
                && self.cursor.peek() != '>'
            {
                self.cursor.consume();
            }
            let key = self.cursor.current_str(start).to_string();
            self.cursor.skip_whitespace();
            if self.cursor.peek() == '=' {
                self.cursor.consume();
                self.cursor.skip_whitespace();
                let quote = self.cursor.peek();
                if quote == '"' || quote == '\'' {
                    self.cursor.consume();
                    let val_start = self.cursor.pos;
                    while !self.cursor.is_eof() && self.cursor.peek() != quote {
                        self.cursor.consume();
                    }
                    let val = self.cursor.current_str(val_start).to_string();
                    let _ = self.cursor.consume(); // consume quote
                    attrs.insert(key, val);
                }
                else {
                    let val_start = self.cursor.pos;
                    while !self.cursor.is_eof() && !self.cursor.peek().is_whitespace() && self.cursor.peek() != '>' {
                        self.cursor.consume();
                    }
                    let val = self.cursor.current_str(val_start).to_string();
                    attrs.insert(key, val);
                }
            }
            else {
                attrs.insert(key, "true".to_string());
            }
            self.cursor.skip_whitespace();
        }
        attrs
    }
}
