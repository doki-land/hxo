use hxo_ir::{CustomBlockIR, IRModule, StyleIR, TemplateIR};
use hxo_types::{HxoValue, Result, Span};
use std::{collections::HashMap, sync::Arc};

mod base;
mod registry;

pub use base::ParseState;
pub use hxo_types::Cursor;
pub use registry::{MetadataSubParser, ParserRegistry, ScriptSubParser, StyleSubParser, TemplateSubParser};

pub struct Parser<'a> {
    name: String,
    state: ParseState<'a>,
    registry: Arc<ParserRegistry>,
}

impl<'a> Parser<'a> {
    pub fn new(name: String, source: &'a str, registry: Arc<ParserRegistry>) -> Self {
        Self { name, state: ParseState::new(source), registry }
    }

    pub fn parse_all(&mut self) -> Result<IRModule> {
        let mut template_nodes = Vec::new();
        let mut script = None;
        let mut ir_styles = Vec::new();
        let mut metadata = HashMap::new();
        let mut custom_blocks = Vec::new();

        let state = &mut self.state;

        while !state.cursor.is_eof() {
            state.cursor.skip_whitespace();
            if state.cursor.peek_str("<template") {
                state.cursor.consume_str("<template");
                let _attrs = state.parse_tag_attributes();
                state.cursor.expect('>')?;

                let start_pos = state.cursor.position();
                let start_offset = state.cursor.pos;
                while !state.cursor.is_eof() && !state.cursor.peek_str("</template>") {
                    state.cursor.consume();
                }
                let content = &state.cursor.source[start_offset..state.cursor.pos];
                state.cursor.consume_str("</template>");

                if let Some(template_parser) = self.registry.get_template_parser() {
                    let mut sub_state = ParseState::with_cursor(Cursor::with_sliced_source(content, start_pos));
                    template_nodes.extend(template_parser.parse(&mut sub_state)?);
                }
            }
            else if state.cursor.peek_str("<script") {
                state.cursor.consume_str("<script");
                let attrs = state.parse_tag_attributes();
                state.cursor.expect('>')?;
                let lang = attrs.get("lang").cloned().unwrap_or_else(|| "js".to_string());

                let start_pos = state.cursor.position();
                let start_offset = state.cursor.pos;
                while !state.cursor.is_eof() && !state.cursor.peek_str("</script>") {
                    state.cursor.consume();
                }
                let content = &state.cursor.source[start_offset..state.cursor.pos];
                state.cursor.consume_str("</script>");

                if let Some(script_parser) = self.registry.get_script_parser(&lang) {
                    let mut sub_state = ParseState::with_cursor(Cursor::with_sliced_source(content, start_pos));
                    script = Some(script_parser.parse(&mut sub_state, &lang)?);
                }
            }
            else if state.cursor.peek_str("<style") {
                state.cursor.consume_str("<style");
                let attrs = state.parse_tag_attributes();
                state.cursor.expect('>')?;
                let lang = attrs.get("lang").cloned().unwrap_or_else(|| "css".to_string());
                let scoped = attrs.contains_key("scoped");

                let start_pos = state.cursor.position();
                let start_offset = state.cursor.pos;
                while !state.cursor.is_eof() && !state.cursor.peek_str("</style>") {
                    state.cursor.consume();
                }
                let content = &state.cursor.source[start_offset..state.cursor.pos];
                let span = state.cursor.span_from(start_pos);
                state.cursor.consume_str("</style>");

                if let Some(style_parser) = self.registry.get_style_parser(&lang) {
                    let mut sub_state = ParseState::with_cursor(Cursor::with_sliced_source(content, start_pos));
                    if let Ok(code) = style_parser.parse(&mut sub_state, &lang) {
                        ir_styles.push(StyleIR { code, lang, scoped, span });
                    }
                }
            }
            else if state.cursor.peek_str("<metadata") {
                state.cursor.consume_str("<metadata");
                let attrs = state.parse_tag_attributes();
                state.cursor.expect('>')?;
                let lang = attrs.get("lang").cloned().unwrap_or_else(|| "yaml".to_string());

                let start_pos = state.cursor.position();
                let start_offset = state.cursor.pos;
                while !state.cursor.is_eof() && !state.cursor.peek_str("</metadata>") {
                    state.cursor.consume();
                }
                let content = &state.cursor.source[start_offset..state.cursor.pos];
                state.cursor.consume_str("</metadata>");

                if let Some(metadata_parser) = self.registry.get_metadata_parser(&lang) {
                    let mut sub_state = ParseState::with_cursor(Cursor::with_sliced_source(content, start_pos));
                    if let Ok(val) = metadata_parser.parse(&mut sub_state, &lang) {
                        if let HxoValue::Object(map) = val {
                            metadata.extend(map);
                        }
                    }
                }
            }
            else if state.cursor.peek_str("<") {
                // Handle custom blocks
                state.cursor.consume(); // consume '<'
                let block_name = state.cursor.consume_while(|c| c.is_alphanumeric() || c == '-');
                let attrs = state.parse_tag_attributes();
                state.cursor.expect('>')?;

                let start_pos = state.cursor.position();
                let start_offset = state.cursor.pos;
                let end_tag = format!("</{}>", block_name);
                while !state.cursor.is_eof() && !state.cursor.peek_str(&end_tag) {
                    state.cursor.consume();
                }
                let content = state.cursor.current_str(start_offset).to_string();
                let span = state.cursor.span_from(start_pos);
                state.cursor.consume_str(&end_tag);

                custom_blocks.push(CustomBlockIR { name: block_name, content, attributes: attrs, span });
            }
            else {
                state.cursor.consume();
            }
        }

        Ok(IRModule {
            name: self.name.clone(),
            metadata,
            script,
            script_meta: None,
            template: if template_nodes.is_empty() { None } else { Some(TemplateIR { nodes: template_nodes, span: Span::default() }) },
            styles: ir_styles,
            i18n: None,
            wasm: Vec::new(),
            custom_blocks,
            span: Span::default(),
        })
    }
}

pub fn parse_i18n(source: &str, lang: &str) -> Result<HashMap<String, HxoValue>> {
    let registry = ParserRegistry::new();
    if let Some(parser) = registry.get_metadata_parser(lang) {
        let mut state = ParseState::new(source);
        let val = parser.parse(&mut state, lang)?;
        if let HxoValue::Object(map) = val {
            return Ok(map);
        }
    }
    Err(hxo_types::Error::parse_error(format!("Unsupported i18n lang: {}", lang), Span::default()))
}
