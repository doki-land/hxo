use hxo_parser::{MetadataSubParser, ParseState};
use hxo_types::{HxoValue, Result};
use std::collections::HashMap;

pub struct FluentParser;

impl MetadataSubParser for FluentParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<HxoValue> {
        let mut parser = FluentParserImpl { state };
        parser.parse_fluent()
    }
}

struct FluentParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
}

impl<'a, 'b> FluentParserImpl<'a, 'b> {
    pub fn parse_fluent(&mut self) -> Result<HxoValue> {
        let mut messages = HashMap::new();
        
        while !self.state.cursor.is_eof() {
            self.state.cursor.skip_whitespace();
            if self.state.cursor.is_eof() {
                break;
            }

            // Skip comments
            if self.state.cursor.peek() == '#' {
                self.state.cursor.consume_while(|c| c != '\n');
                continue;
            }

            // Parse ID
            let id = self.state.cursor.consume_while(|c| c.is_alphanumeric() || c == '-' || c == '_');
            if id.is_empty() {
                // If we can't parse an ID, just skip one char to avoid infinite loop
                self.state.cursor.consume();
                continue;
            }

            self.state.cursor.skip_whitespace();
            if self.state.cursor.peek() == '=' {
                self.state.cursor.expect('=')?;
                self.state.cursor.skip_whitespace();
                
                // Parse pattern (simple string for now)
                let value = self.state.cursor.consume_while(|c| c != '\n').trim().to_string();
                
                let mut message_map = HashMap::new();
                message_map.insert("val".to_string(), HxoValue::String(value));
                
                // Parse attributes
                self.state.cursor.skip_whitespace();
                while self.state.cursor.peek() == '.' {
                    self.state.cursor.expect('.')?;
                    let attr_name = self.state.cursor.consume_while(|c| c.is_alphanumeric() || c == '-');
                    self.state.cursor.skip_whitespace();
                    self.state.cursor.expect('=')?;
                    self.state.cursor.skip_whitespace();
                    let attr_value = self.state.cursor.consume_while(|c| c != '\n').trim().to_string();
                    message_map.insert(attr_name, HxoValue::String(attr_value));
                    self.state.cursor.skip_whitespace();
                }
                
                // If there's only 'val', we can simplify it to just the string
                if message_map.len() == 1 {
                    messages.insert(id, HxoValue::String(message_map.remove("val").unwrap().as_str().unwrap().to_string()));
                } else {
                    messages.insert(id, HxoValue::Object(message_map));
                }
            }
        }
        
        Ok(HxoValue::Object(messages))
    }
}
