use hxo_parser::{MetadataParser, ParseState};
use hxo_types::{HxoValue, Result, Route, RouterConfig, Span};
use std::collections::HashMap;

pub struct YamlParser;

impl MetadataParser for YamlParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<HxoValue> {
        let mut parser = YamlParserImpl { state };
        parser.parse_yaml()
    }
}

pub fn parse_router(source: &str) -> Result<RouterConfig> {
    let value = parse(source)?;
    match value {
        HxoValue::Object(map) => {
            let routes_val = map
                .get("routes")
                .ok_or_else(|| hxo_types::Error::parse_error("Missing routes".to_string(), Span::default()))?;
            let routes = parse_routes(routes_val)?;
            let mode = map.get("mode").and_then(|v| v.as_str()).unwrap_or("hash").to_string();
            let base = map.get("base").and_then(|v| v.as_str()).map(|s| s.to_string());

            Ok(RouterConfig { routes, mode, base, span: Span::default() })
        }
        _ => Err(hxo_types::Error::parse_error("Expected object for router config".to_string(), Span::default())),
    }
}

fn parse_routes(value: &HxoValue) -> Result<Vec<Route>> {
    match value {
        HxoValue::Array(arr) => {
            let mut routes = Vec::new();
            for item in arr {
                if let HxoValue::Object(map) = item {
                    let path = map.get("path").and_then(|v| v.as_str()).unwrap_or("/").to_string();
                    let component = map.get("component").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let name = map.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let redirect = map.get("redirect").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let children =
                        if let Some(children_val) = map.get("children") { Some(parse_routes(children_val)?) } else { None };

                    routes.push(Route { path, component, name, redirect, children, meta: None, span: Span::default() });
                }
            }
            Ok(routes)
        }
        _ => Err(hxo_types::Error::parse_error("Expected array for routes".to_string(), Span::default())),
    }
}

struct YamlParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
}

impl<'a, 'b> YamlParserImpl<'a, 'b> {
    pub fn parse_yaml(&mut self) -> Result<HxoValue> {
        self.state.cursor.skip_whitespace();
        if self.state.cursor.is_eof() {
            return Ok(HxoValue::Null);
        }

        let first = self.state.cursor.peek();
        if first == '-' {
            self.parse_list()
        }
        else if first == '{' {
            self.parse_json_object()
        }
        else if first == '[' {
            self.parse_json_array()
        }
        else {
            // Check if it's a map
            let indent = self.get_indent();
            let line = self.peek_line();
            if line.contains(':') { self.parse_map(indent) } else { self.parse_scalar() }
        }
    }

    fn parse_map(&mut self, indent: usize) -> Result<HxoValue> {
        let mut map = HashMap::new();
        while !self.state.cursor.is_eof() {
            self.state.cursor.skip_spaces();
            if self.state.cursor.is_eof() || self.state.cursor.peek() == '\n' {
                if !self.state.cursor.is_eof() {
                    self.state.cursor.consume(); // \n
                }
                continue;
            }

            let current_indent = self.get_indent();
            if current_indent < indent {
                break;
            }

            if self.state.cursor.peek() == '#' {
                self.skip_line();
                continue;
            }

            // If we hit a '-' at the same indent as the current map, it's likely we're inside a list item
            if self.state.cursor.peek() == '-' && current_indent == indent {
                break;
            }

            let key = self.consume_key()?;
            if key.is_empty() {
                break;
            }

            self.state.cursor.skip_spaces();
            if self.state.cursor.peek() == ':' {
                self.state.cursor.consume();
                self.state.cursor.skip_spaces();

                let value = if self.state.cursor.peek() == '\n' || self.state.cursor.is_eof() {
                    self.state.cursor.skip_whitespace();
                    let next_indent = self.get_indent();
                    if self.state.cursor.peek() == '-' {
                        self.parse_list()?
                    }
                    else if next_indent > current_indent {
                        self.parse_map(next_indent)?
                    }
                    else {
                        HxoValue::Null
                    }
                }
                else if self.state.cursor.peek() == '-' {
                    self.parse_list()?
                }
                else {
                    let line = self.peek_line();
                    if line.contains(':')
                        && !line.starts_with('"')
                        && !line.starts_with('\'')
                        && !line.starts_with('{')
                        && !line.starts_with('[')
                    {
                        let next_indent = self.get_indent();
                        self.parse_map(next_indent)?
                    }
                    else {
                        self.parse_scalar()?
                    }
                };
                map.insert(key, value);
            }
            else {
                break;
            }
        }
        Ok(HxoValue::Object(map))
    }

    fn parse_list(&mut self) -> Result<HxoValue> {
        let mut list = Vec::new();
        let indent = self.get_indent();

        while !self.state.cursor.is_eof() {
            self.state.cursor.skip_spaces();
            if self.state.cursor.is_eof() || self.state.cursor.peek() == '\n' {
                if !self.state.cursor.is_eof() {
                    self.state.cursor.consume(); // \n
                }
                continue;
            }

            let current_indent = self.get_indent();
            if current_indent < indent {
                break;
            }

            if self.state.cursor.peek() == '-' {
                self.state.cursor.consume(); // -
                self.state.cursor.skip_spaces();

                let value = if self.state.cursor.peek() == '\n' || self.state.cursor.is_eof() {
                    self.state.cursor.skip_whitespace();
                    let next_indent = self.get_indent();
                    if next_indent > current_indent { self.parse_map(next_indent)? } else { HxoValue::Null }
                }
                else {
                    let line = self.peek_line();
                    if line.contains(':')
                        && !line.starts_with('"')
                        && !line.starts_with('\'')
                        && !line.starts_with('{')
                        && !line.starts_with('[')
                    {
                        let next_indent = self.get_indent();
                        self.parse_map(next_indent)?
                    }
                    else {
                        self.parse_scalar()?
                    }
                };
                list.push(value);
            }
            else {
                break;
            }
        }
        Ok(HxoValue::Array(list))
    }

    fn parse_scalar(&mut self) -> Result<HxoValue> {
        let s = self.consume_value()?;
        if s == "true" {
            Ok(HxoValue::Bool(true))
        }
        else if s == "false" {
            Ok(HxoValue::Bool(false))
        }
        else if s == "null" || s == "~" {
            Ok(HxoValue::Null)
        }
        else if let Ok(n) = s.parse::<f64>() {
            Ok(HxoValue::Number(n))
        }
        else {
            // Handle quoted strings
            if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
                Ok(HxoValue::String(s[1..s.len() - 1].to_string()))
            }
            else {
                Ok(HxoValue::String(s.to_string()))
            }
        }
    }

    fn get_indent(&self) -> usize {
        let mut pos = self.state.cursor.pos;
        let mut count = 0;
        while pos > 0 {
            pos -= 1;
            let c = self.state.cursor.source.as_bytes()[pos] as char;
            if c == '\n' {
                break;
            }
            if c == ' ' {
                count += 1;
            }
            else {
                count = 0;
            }
        }
        count
    }

    fn peek_line(&self) -> String {
        let mut pos = self.state.cursor.pos;
        let start = pos;
        while pos < self.state.cursor.source.len() && self.state.cursor.source.as_bytes()[pos] != b'\n' {
            pos += 1;
        }
        self.state.cursor.source[start..pos].to_string()
    }

    fn parse_json_object(&mut self) -> Result<HxoValue> {
        let start = self.state.cursor.pos;
        let mut depth = 0;
        while !self.state.cursor.is_eof() {
            let c = self.state.cursor.peek();
            if c == '{' {
                depth += 1;
            }
            else if c == '}' {
                depth -= 1;
            }
            self.state.cursor.consume();
            if depth == 0 {
                break;
            }
        }
        Ok(HxoValue::String(self.state.cursor.source[start..self.state.cursor.pos].to_string()))
    }

    fn parse_json_array(&mut self) -> Result<HxoValue> {
        let start = self.state.cursor.pos;
        let mut depth = 0;
        while !self.state.cursor.is_eof() {
            let c = self.state.cursor.peek();
            if c == '[' {
                depth += 1;
            }
            else if c == ']' {
                depth -= 1;
            }
            self.state.cursor.consume();
            if depth == 0 {
                break;
            }
        }
        Ok(HxoValue::String(self.state.cursor.source[start..self.state.cursor.pos].to_string()))
    }

    fn consume_key(&mut self) -> Result<String> {
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != ':' && !self.state.cursor.peek().is_whitespace() {
            self.state.cursor.consume();
        }
        Ok(self.state.cursor.source[start..self.state.cursor.pos].to_string())
    }

    fn consume_value(&mut self) -> Result<String> {
        self.state.cursor.skip_spaces();
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '\n' && self.state.cursor.peek() != '#' {
            self.state.cursor.consume();
        }
        Ok(self.state.cursor.source[start..self.state.cursor.pos].trim().to_string())
    }

    fn skip_line(&mut self) {
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '\n' {
            self.state.cursor.consume();
        }
        if !self.state.cursor.is_eof() {
            self.state.cursor.consume();
        }
    }
}

pub fn parse(source: &str) -> Result<HxoValue> {
    let mut state = ParseState::new(source);
    let mut parser = YamlParserImpl { state: &mut state };
    parser.parse_yaml()
}

pub fn parse_yaml(source: &str) -> Result<HxoValue> {
    let mut state = ParseState::new(source);
    let mut parser = YamlParserImpl { state: &mut state };
    parser.parse_yaml()
}
