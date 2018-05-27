use hxo_ir::TemplateNodeIR;
use hxo_parser::{ParseState, TemplateParser};
use hxo_types::Result;

pub struct MarkdownParser;

impl TemplateParser for MarkdownParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<Vec<TemplateNodeIR>> {
        let mut impl_parser = MarkdownParserImpl::with_state(state);
        let html = impl_parser.parse_to_html()?;
        hxo_parser_template::parse(html.trim())
    }
}

struct MarkdownParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
}

impl<'a, 'b> MarkdownParserImpl<'a, 'b> {
    pub fn with_state(state: &'a mut ParseState<'b>) -> Self {
        Self { state }
    }

    pub fn parse_to_html(&mut self) -> Result<String> {
        let mut html = String::new();
        while !self.state.cursor.is_eof() {
            if self.state.cursor.peek() == '#' {
                html.push_str(&self.parse_heading()?);
            }
            else if self.state.cursor.peek() == '-' || self.state.cursor.peek() == '*' {
                if self.state.cursor.peek_n(1).is_whitespace() {
                    html.push_str(&self.parse_list()?);
                }
                else {
                    html.push_str(&self.parse_paragraph()?);
                }
            }
            else if self.state.cursor.peek() == '\n' {
                self.state.cursor.consume();
            }
            else {
                html.push_str(&self.parse_paragraph()?);
            }
        }
        Ok(html)
    }

    fn parse_heading(&mut self) -> Result<String> {
        let mut level = 0;
        while self.state.cursor.peek() == '#' && level < 6 {
            self.state.cursor.consume();
            level += 1;
        }
        self.state.cursor.skip_whitespace();
        let content = self.parse_inline()?;
        Ok(format!("<h{}>{}</h{}>\n", level, content, level))
    }

    fn parse_list(&mut self) -> Result<String> {
        let mut html = String::from("<ul>\n");
        while !self.state.cursor.is_eof() && (self.state.cursor.peek() == '-' || self.state.cursor.peek() == '*') {
            self.state.cursor.consume(); // - or *
            self.state.cursor.skip_whitespace();
            let content = self.parse_inline()?;
            html.push_str(&format!("  <li>{}</li>\n", content));
            self.state.cursor.skip_whitespace();
        }
        html.push_str("</ul>\n");
        Ok(html)
    }

    fn parse_paragraph(&mut self) -> Result<String> {
        let content = self.parse_inline()?;
        if content.is_empty() { Ok(String::new()) } else { Ok(format!("<p>{}</p>\n", content)) }
    }

    fn parse_inline(&mut self) -> Result<String> {
        let mut inline = String::new();
        while !self.state.cursor.is_eof() && self.state.cursor.peek() != '\n' {
            let c = self.state.cursor.peek();
            match c {
                '*' => {
                    if self.state.cursor.peek_str("**") {
                        self.state.cursor.consume_n(2);
                        let content = self.consume_until("**")?;
                        self.state.cursor.consume_n(2);
                        inline.push_str(&format!("<strong>{}</strong>", content));
                    }
                    else {
                        self.state.cursor.consume();
                        let content = self.consume_until("*")?;
                        self.state.cursor.consume();
                        inline.push_str(&format!("<em>{}</em>", content));
                    }
                }
                '_' => {
                    if self.state.cursor.peek_str("__") {
                        self.state.cursor.consume_n(2);
                        let content = self.consume_until("__")?;
                        self.state.cursor.consume_n(2);
                        inline.push_str(&format!("<strong>{}</strong>", content));
                    }
                    else {
                        self.state.cursor.consume();
                        let content = self.consume_until("_")?;
                        self.state.cursor.consume();
                        inline.push_str(&format!("<em>{}</em>", content));
                    }
                }
                '[' => {
                    self.state.cursor.consume();
                    let text = self.consume_until("]")?;
                    self.state.cursor.consume();
                    if self.state.cursor.peek() == '(' {
                        self.state.cursor.consume();
                        let url = self.consume_until(")")?;
                        self.state.cursor.consume();
                        inline.push_str(&format!("<a href=\"{}\">{}</a>", url, text));
                    }
                    else {
                        inline.push('[');
                        inline.push_str(&text);
                        inline.push(']');
                    }
                }
                '!' => {
                    if self.state.cursor.peek_n(1) == '[' {
                        self.state.cursor.consume_n(2);
                        let alt = self.consume_until("]")?;
                        self.state.cursor.consume();
                        if self.state.cursor.peek() == '(' {
                            self.state.cursor.consume();
                            let src = self.consume_until(")")?;
                            self.state.cursor.consume();
                            inline.push_str(&format!("<img src=\"{}\" alt=\"{}\">", src, alt));
                        }
                        else {
                            inline.push_str("![");
                            inline.push_str(&alt);
                            inline.push(']');
                        }
                    }
                    else {
                        inline.push(self.state.cursor.consume());
                    }
                }
                '`' => {
                    self.state.cursor.consume();
                    let content = self.consume_until("`")?;
                    self.state.cursor.consume();
                    inline.push_str(&format!("<code>{}</code>", content));
                }
                _ => {
                    inline.push(self.state.cursor.consume());
                }
            }
        }
        if !self.state.cursor.is_eof() {
            self.state.cursor.consume(); // \n
        }
        Ok(inline.trim().to_string())
    }

    fn consume_until(&mut self, s: &str) -> Result<String> {
        let start = self.state.cursor.pos;
        while !self.state.cursor.is_eof() && !self.state.cursor.peek_str(s) {
            self.state.cursor.consume();
        }
        Ok(self.state.cursor.current_str(start).to_string())
    }
}

pub fn parse(source: &str) -> Result<String> {
    let mut state = ParseState::new(source);
    let mut parser = MarkdownParserImpl::with_state(&mut state);
    parser.parse_to_html()
}
