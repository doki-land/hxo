use hxo_parser::{Cursor, MetadataParser, ParseState};
use hxo_types::{Error, HxoValue, Position, Result, Route, RouterConfig, Span};
use std::collections::HashMap;

pub struct JsonParser;

impl MetadataParser for JsonParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<HxoValue> {
        let mut parser = JsonParserImpl { state };
        parser.parse_json()
    }
}

struct JsonParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
}

impl<'a, 'b> JsonParserImpl<'a, 'b> {
    pub fn parse_json(&mut self) -> Result<HxoValue> {
        self.state.cursor.skip_whitespace();
        let val = self.parse_value()?;
        self.state.cursor.skip_whitespace();
        Ok(val)
    }

    fn parse_value(&mut self) -> Result<HxoValue> {
        self.state.cursor.skip_whitespace();
        match self.state.cursor.peek() {
            '{' => self.parse_object(),
            '[' => self.parse_array(),
            '"' => Ok(HxoValue::String(self.consume_string()?)),
            't' | 'f' => self.parse_bool(),
            'n' => self.parse_null(),
            '0'..='9' | '-' => self.parse_number(),
            _ => Err(self.state.unexpected_char()),
        }
    }

    fn parse_object(&mut self) -> Result<HxoValue> {
        self.state.cursor.expect('{')?;
        let mut map = HashMap::new();
        self.state.cursor.skip_whitespace();
        while self.state.cursor.peek() != '}' {
            let key = self.consume_string()?;
            self.state.cursor.skip_whitespace();
            self.state.cursor.expect(':')?;
            let val = self.parse_value()?;
            map.insert(key, val);
            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek() == ',' {
                self.state.cursor.consume();
                self.state.cursor.skip_whitespace();
            }
        }
        self.state.cursor.expect('}')?;
        Ok(HxoValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<HxoValue> {
        self.state.cursor.expect('[')?;
        let mut arr = Vec::new();
        self.state.cursor.skip_whitespace();
        while self.state.cursor.peek() != ']' {
            arr.push(self.parse_value()?);
            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek() == ',' {
                self.state.cursor.consume();
                self.state.cursor.skip_whitespace();
            }
        }
        self.state.cursor.expect(']')?;
        Ok(HxoValue::Array(arr))
    }

    fn parse_bool(&mut self) -> Result<HxoValue> {
        if self.state.cursor.consume_str("true") {
            Ok(HxoValue::Bool(true))
        }
        else if self.state.cursor.consume_str("false") {
            Ok(HxoValue::Bool(false))
        }
        else {
            Err(Error::expected_one_of(
                vec!["true".to_string(), "false".to_string()],
                self.state.cursor.peek().to_string(),
                self.state.cursor.span_at_current(),
            ))
        }
    }

    fn parse_null(&mut self) -> Result<HxoValue> {
        if self.state.cursor.consume_str("null") {
            Ok(HxoValue::Null)
        }
        else {
            Err(Error::expected_string(
                "null".to_string(),
                self.state.cursor.peek().to_string(),
                self.state.cursor.span_at_current(),
            ))
        }
    }

    fn parse_number(&mut self) -> Result<HxoValue> {
        let start = self.state.cursor.pos;
        if self.state.cursor.peek() == '-' {
            self.state.cursor.consume();
        }
        while self.state.cursor.peek().is_ascii_digit() {
            self.state.cursor.consume();
        }
        if self.state.cursor.peek() == '.' {
            self.state.cursor.consume();
            while self.state.cursor.peek().is_ascii_digit() {
                self.state.cursor.consume();
            }
        }
        let s = self.state.cursor.current_str(start);
        match s.parse::<f64>() {
            Ok(n) => Ok(HxoValue::Number(n)),
            Err(_) => Err(Error::parse_float_error(
                s.to_string(),
                self.state.cursor.span_from(Position { line: 1, column: 1, offset: start as u32 }),
            )),
        }
    }

    pub fn parse(&mut self) -> Result<RouterConfig> {
        self.state.cursor.skip_whitespace();
        let config = self.parse_router_config()?;
        self.state.cursor.skip_whitespace();
        if !self.state.cursor.is_eof() {
            return Err(Error::trailing_content(self.state.cursor.span_at_current()));
        }
        Ok(config)
    }

    fn parse_router_config(&mut self) -> Result<RouterConfig> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.expect('{')?;
        self.state.cursor.skip_whitespace();

        let mut routes = Vec::new();
        let mut mode = "history".to_string();
        let mut base = None;

        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
            let key = self.consume_string()?;
            self.state.cursor.skip_whitespace();
            self.state.cursor.expect(':')?;
            self.state.cursor.skip_whitespace();

            match key.as_str() {
                "routes" => {
                    routes = self.parse_routes()?;
                }
                "mode" => {
                    mode = self.consume_string()?;
                }
                "base" => {
                    base = Some(self.consume_string()?);
                }
                _ => {
                    self.skip_value()?;
                }
            }

            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek() == ',' {
                self.state.cursor.consume();
                self.state.cursor.skip_whitespace();
            }
        }

        self.state.cursor.expect('}')?;
        let end_pos = self.state.cursor.position();

        Ok(RouterConfig { routes, mode, base, span: Span { start: start_pos, end: end_pos } })
    }

    fn parse_routes(&mut self) -> Result<Vec<Route>> {
        self.state.cursor.expect('[')?;
        let mut routes = Vec::new();

        while !self.state.cursor.is_eof() && self.state.cursor.peek() != ']' {
            self.state.cursor.skip_whitespace();
            routes.push(self.parse_route()?);
            self.state.cursor.skip_whitespace();

            if self.state.cursor.peek() == ',' {
                self.state.cursor.consume();
                self.state.cursor.skip_whitespace();
            }
            else if self.state.cursor.peek() != ']' {
                return Err(Error::expected_one_of(
                    vec![",".to_string(), "]".to_string()],
                    self.state.cursor.peek().to_string(),
                    self.state.cursor.span_at_current(),
                ));
            }
        }

        self.state.cursor.expect(']')?;
        Ok(routes)
    }

    fn parse_route(&mut self) -> Result<Route> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.expect('{')?;

        let mut path = String::new();
        let mut component = String::new();
        let mut name = None;
        let mut redirect = None;
        let mut children = None;
        let mut meta = None;

        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
            self.state.cursor.skip_whitespace();
            let key = self.consume_string()?;
            self.state.cursor.skip_whitespace();
            self.state.cursor.expect(':')?;
            self.state.cursor.skip_whitespace();

            match key.as_str() {
                "path" => path = self.consume_string()?,
                "component" => component = self.consume_string()?,
                "name" => name = Some(self.consume_string()?),
                "redirect" => redirect = Some(self.consume_string()?),
                "children" => children = Some(self.parse_routes()?),
                "meta" => meta = Some(self.parse_json_value()?),
                _ => self.skip_value()?,
            }

            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek() == ',' {
                self.state.cursor.consume();
                self.state.cursor.skip_whitespace();
            }
            else if self.state.cursor.peek() != '}' {
                break;
            }
        }

        self.state.cursor.expect('}')?;
        let end_pos = self.state.cursor.position();

        Ok(Route { path, component, name, redirect, children, meta, span: Span { start: start_pos, end: end_pos } })
    }

    fn parse_json_value(&mut self) -> Result<HxoValue> {
        self.state.cursor.skip_whitespace();
        let c = self.state.cursor.peek();
        match c {
            '{' => {
                self.state.cursor.consume();
                let mut map = HashMap::new();
                while !self.state.cursor.is_eof() && self.state.cursor.peek() != '}' {
                    self.state.cursor.skip_whitespace();
                    let key = self.consume_string()?;
                    self.state.cursor.skip_whitespace();
                    self.state.cursor.expect(':')?;
                    let val = self.parse_json_value()?;
                    map.insert(key, val);
                    self.state.cursor.skip_whitespace();
                    if self.state.cursor.peek() == ',' {
                        self.state.cursor.consume();
                    }
                }
                self.state.cursor.expect('}')?;
                Ok(HxoValue::Object(map))
            }
            '[' => {
                self.state.cursor.consume();
                let mut vec = Vec::new();
                while !self.state.cursor.is_eof() && self.state.cursor.peek() != ']' {
                    vec.push(self.parse_json_value()?);
                    self.state.cursor.skip_whitespace();
                    if self.state.cursor.peek() == ',' {
                        self.state.cursor.consume();
                    }
                }
                self.state.cursor.expect(']')?;
                Ok(HxoValue::Array(vec))
            }
            '"' => Ok(HxoValue::String(self.consume_string()?)),
            't' => {
                self.state.cursor.expect_str("true")?;
                Ok(HxoValue::Bool(true))
            }
            'f' => {
                self.state.cursor.expect_str("false")?;
                Ok(HxoValue::Bool(false))
            }
            'n' => {
                self.state.cursor.expect_str("null")?;
                Ok(HxoValue::Null)
            }
            c if c.is_ascii_digit() || c == '-' => {
                let start = self.state.cursor.pos;
                while !self.state.cursor.is_eof()
                    && (self.state.cursor.peek().is_ascii_digit()
                        || self.state.cursor.peek() == '.'
                        || self.state.cursor.peek() == '-'
                        || self.state.cursor.peek() == 'e'
                        || self.state.cursor.peek() == 'E')
                {
                    self.state.cursor.consume();
                }
                let s = self.state.cursor.current_str(start);
                let n = s.parse::<f64>().map_err(|_| {
                    Error::parse_float_error(
                        s.to_string(),
                        self.state.cursor.span_from(Position { line: 1, column: 1, offset: start as u32 }), // Simplified
                    )
                })?;
                Ok(HxoValue::Number(n))
            }
            _ => Err(Error::unexpected_char(c, self.state.cursor.span_at_current())),
        }
    }

    fn skip_value(&mut self) -> Result<()> {
        self.parse_json_value()?;
        Ok(())
    }

    fn consume_string(&mut self) -> Result<String> {
        self.state.cursor.expect('"')?;
        let mut s = String::new();
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '"' {
            if self.state.cursor.peek() == '\\' {
                self.state.cursor.consume();
                let escaped = self.state.cursor.consume();
                match escaped {
                    '"' => s.push('"'),
                    '\\' => s.push('\\'),
                    '/' => s.push('/'),
                    'b' => s.push('\x08'),
                    'f' => s.push('\x0c'),
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    'u' => {
                        // Handle unicode escape
                        s.push(self.state.cursor.consume());
                    }
                    _ => s.push(escaped),
                }
            }
            else {
                s.push(self.state.cursor.consume());
            }
        }
        self.state.cursor.expect('"')?;
        Ok(s)
    }
}

pub fn parse(source: &str, start_pos: Position) -> Result<RouterConfig> {
    let mut state = ParseState::with_cursor(Cursor::with_sliced_source(source, start_pos));
    let mut parser = JsonParserImpl { state: &mut state };
    parser.parse()
}
