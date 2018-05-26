use hxo_ir::{AttributeIR, ElementIR, ExpressionIR, TemplateNodeIR};
use hxo_parser::{ParseState, TemplateSubParser};
use hxo_types::{Result, Span, is_void_element};

pub struct TemplateParser;

pub fn parse(source: &str) -> Result<Vec<TemplateNodeIR>> {
    let mut state = ParseState::new(source);
    let parser = TemplateParser;
    parser.parse(&mut state)
}

impl TemplateSubParser for TemplateParser {
    fn parse(&self, state: &mut ParseState) -> Result<Vec<TemplateNodeIR>> {
        let mut parser = TemplateParserImpl { state };
        parser.parse()
    }
}

struct TemplateParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
}

impl<'a, 'b> TemplateParserImpl<'a, 'b> {
    pub fn parse(&mut self) -> Result<Vec<TemplateNodeIR>> {
        let mut nodes = Vec::new();
        while !self.state.cursor.is_eof() {
            if self.state.cursor.peek_str("{{") {
                nodes.push(self.parse_interpolation()?);
            }
            else if self.state.cursor.peek() == '<' {
                if self.state.cursor.peek_str("<!--") {
                    nodes.push(self.parse_comment()?);
                }
                else if self.state.cursor.peek_str("</") {
                    break; // Closing tag, handle in parent
                }
                else {
                    nodes.push(self.parse_element()?);
                }
            }
            else {
                nodes.push(self.parse_text()?);
            }
        }
        Ok(nodes)
    }

    fn parse_element(&mut self) -> Result<TemplateNodeIR> {
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
                let peek = self.state.cursor.peek();
                if peek == '"' || peek == '\'' {
                    Some(self.state.cursor.consume_string()?)
                }
                else {
                    Some(self.state.cursor.consume_while(|c| !c.is_whitespace() && c != '>' && c != '/'))
                }
            }
            else {
                None
            };

            let is_directive = attr_name.starts_with(':') || attr_name.starts_with('@') || attr_name.starts_with("v-");

            let is_dynamic = is_directive || attr_name == "class" || attr_name == "style"; // class/style can be dynamic in some contexts, but let's keep it simple

            let attr_end = self.state.cursor.position();
            attributes.push(AttributeIR {
                name: attr_name,
                value: attr_value,
                value_ast: None,
                is_directive,
                is_dynamic,
                span: Span { start: attr_start, end: attr_end },
            });
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
        if !is_self_closing && !is_void_element(&tag) {
            if tag == "script" || tag == "style" {
                let start = self.state.cursor.pos;
                let end_tag = format!("</{}>", tag);
                while !self.state.cursor.is_eof() && !self.state.cursor.peek_str(&end_tag) {
                    self.state.cursor.consume();
                }
                let content = self.state.cursor.current_str(start).to_string();
                if !content.is_empty() {
                    let text_span = self.state.cursor.span_from(start_pos);
                    children.push(TemplateNodeIR::Text(content, text_span));
                }
                self.state.cursor.expect_str(&end_tag)?;
            }
            else {
                children = self.parse()?;
                self.state.cursor.expect_str(&format!("</{}>", tag))?;
            }
        }

        let end_pos = self.state.cursor.position();

        Ok(TemplateNodeIR::Element(ElementIR {
            tag,
            attributes,
            children,
            is_static: false, // Default to false, optimizer will handle it
            span: Span { start: start_pos, end: end_pos },
        }))
    }

    fn parse_interpolation(&mut self) -> Result<TemplateNodeIR> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.expect_str("{{")?;
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && !self.state.cursor.peek_str("}}") {
            self.state.cursor.consume();
        }
        let content = self.state.cursor.current_str(start).trim().to_string();
        self.state.cursor.expect_str("}}")?;
        let end_pos = self.state.cursor.position();

        Ok(TemplateNodeIR::Interpolation(ExpressionIR {
            code: content,
            ast: None,
            span: Span { start: start_pos, end: end_pos },
        }))
    }

    fn parse_text(&mut self) -> Result<TemplateNodeIR> {
        let start_pos = self.state.cursor.position();
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '<' && !self.state.cursor.peek_str("{{") {
            self.state.cursor.consume();
        }
        let end_pos = self.state.cursor.position();
        Ok(TemplateNodeIR::Text(self.state.cursor.current_str(start).to_string(), Span { start: start_pos, end: end_pos }))
    }

    fn parse_comment(&mut self) -> Result<TemplateNodeIR> {
        let start_pos = self.state.cursor.position();
        self.state.cursor.expect_str("<!--")?;
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && !self.state.cursor.peek_str("-->") {
            self.state.cursor.consume();
        }
        let content = self.state.cursor.current_str(start).to_string();
        self.state.cursor.expect_str("-->")?;
        let end_pos = self.state.cursor.position();
        Ok(TemplateNodeIR::Comment(content, Span { start: start_pos, end: end_pos }))
    }
}
