use hxo_parser::{MetadataSubParser, ParseState};
use hxo_types::{HxoValue, Result};
use std::collections::HashMap;

pub struct PropertiesParser;

impl MetadataSubParser for PropertiesParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<HxoValue> {
        let mut parser = PropertiesParserImpl { state };
        parser.parse_properties()
    }
}

struct PropertiesParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
}

impl<'a, 'b> PropertiesParserImpl<'a, 'b> {
    pub fn parse_properties(&mut self) -> Result<HxoValue> {
        let mut map = HashMap::new();
        
        while !self.state.cursor.is_eof() {
            self.state.cursor.skip_whitespace();
            if self.state.cursor.is_eof() {
                break;
            }

            // Skip comments
            let peek = self.state.cursor.peek();
            if peek == '#' || peek == '!' {
                self.state.cursor.consume_while(|c| c != '\n');
                continue;
            }

            // Parse key
            let key = self.state.cursor.consume_while(|c| !c.is_whitespace() && c != '=' && c != ':');
            if key.is_empty() {
                self.state.cursor.consume();
                continue;
            }

            self.state.cursor.skip_whitespace();
            let separator = self.state.cursor.peek();
            if separator == '=' || separator == ':' {
                self.state.cursor.consume();
            }
            
            self.state.cursor.skip_whitespace();
            
            // Parse value (until end of line)
            let value = self.state.cursor.consume_while(|c| c != '\n').trim().to_string();
            
            // Handle simple nesting with dots if needed? 
            // For now just flat map as per properties spec
            map.insert(key, HxoValue::String(value));
        }
        
        Ok(HxoValue::Object(map))
    }
}
