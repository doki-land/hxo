use hxo_ir::{AttributeIR, ElementIR, TemplateNodeIR};
use hxo_parser::{ParseState, TemplateParser};
use hxo_types::{Result, Span};

pub struct PugParser;

pub fn parse(source: &str) -> Result<Vec<TemplateNodeIR>> {
    let mut state = ParseState::new(source);
    PugParser.parse(&mut state, "pug")
}

impl TemplateParser for PugParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<Vec<TemplateNodeIR>> {
        let mut parser = PugParserImpl { state };
        parser.parse()
    }
}

struct PugParserImpl<'a, 'b> {
    state: &'a mut ParseState<'b>,
}

impl<'a, 'b> PugParserImpl<'a, 'b> {
    pub fn parse(&mut self) -> Result<Vec<TemplateNodeIR>> {
        let source = self.state.cursor.source;
        let mut nodes = Vec::new();
        let mut stack: Vec<(usize, ElementIR)> = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            let indent = line.len() - trimmed.len();
            let (node, is_element) = self.parse_line(trimmed)?;

            if is_element {
                let el = match node {
                    TemplateNodeIR::Element(e) => e,
                    _ => unreachable!(),
                };

                while let Some((stack_indent, _)) = stack.last() {
                    if *stack_indent >= indent {
                        let (_, completed_el) = stack.pop().unwrap();
                        if let Some((_, parent_el)) = stack.last_mut() {
                            parent_el.children.push(TemplateNodeIR::Element(completed_el));
                        }
                        else {
                            nodes.push(TemplateNodeIR::Element(completed_el));
                        }
                    }
                    else {
                        break;
                    }
                }
                stack.push((indent, el));
            }
            else {
                // For non-elements (text, comments), add to current parent or root
                if let Some((_, parent_el)) = stack.last_mut() {
                    parent_el.children.push(node);
                }
                else {
                    nodes.push(node);
                }
            }
        }

        // Pop remaining elements from stack
        while let Some((_, completed_el)) = stack.pop() {
            if let Some((_, parent_el)) = stack.last_mut() {
                parent_el.children.push(TemplateNodeIR::Element(completed_el));
            }
            else {
                nodes.push(TemplateNodeIR::Element(completed_el));
            }
        }

        Ok(nodes)
    }

    fn parse_line(&self, line: &str) -> Result<(TemplateNodeIR, bool)> {
        // Simple Pug line parser: tagname#id.class1.class2(attr=val) text
        let mut cursor = 0;
        let bytes = line.as_bytes();

        // 1. Tag name
        let mut tag_name = String::new();
        while cursor < bytes.len() && (bytes[cursor] as char).is_alphanumeric() {
            tag_name.push(bytes[cursor] as char);
            cursor += 1;
        }

        if tag_name.is_empty() {
            // Check for #id or .class without tag (defaults to div)
            if cursor < bytes.len() && (bytes[cursor] == b'#' || bytes[cursor] == b'.') {
                tag_name = "div".to_string();
            }
            else {
                // Plain text
                return Ok((TemplateNodeIR::Text(line.to_string(), Span::unknown()), false));
            }
        }

        let mut attributes = Vec::new();

        // 2. ID and classes
        while cursor < bytes.len() {
            match bytes[cursor] {
                b'#' => {
                    cursor += 1;
                    let mut id = String::new();
                    while cursor < bytes.len() && ((bytes[cursor] as char).is_alphanumeric() || bytes[cursor] == b'-') {
                        id.push(bytes[cursor] as char);
                        cursor += 1;
                    }
                    attributes.push(AttributeIR {
                        name: "id".to_string(),
                        value: Some(id),
                        value_ast: None,
                        is_directive: false,
                        is_dynamic: false,
                        span: Span::unknown(),
                    });
                }
                b'.' => {
                    cursor += 1;
                    let mut class = String::new();
                    while cursor < bytes.len() && ((bytes[cursor] as char).is_alphanumeric() || bytes[cursor] == b'-') {
                        class.push(bytes[cursor] as char);
                        cursor += 1;
                    }
                    attributes.push(AttributeIR {
                        name: "class".to_string(),
                        value: Some(class),
                        value_ast: None,
                        is_directive: false,
                        is_dynamic: false,
                        span: Span::unknown(),
                    });
                }
                _ => break,
            }
        }

        // 3. Attributes in parentheses
        if cursor < bytes.len() && bytes[cursor] == b'(' {
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor] != b')' {
                // Parse attr=val
                let mut name = String::new();
                while cursor < bytes.len()
                    && bytes[cursor] != b'='
                    && bytes[cursor] != b')'
                    && !(bytes[cursor] as char).is_whitespace()
                {
                    name.push(bytes[cursor] as char);
                    cursor += 1;
                }

                let mut value = None;
                let mut value_ast = None;
                let mut is_dynamic = false;
                let mut is_directive = false;

                if cursor < bytes.len() && bytes[cursor] == b'=' {
                    cursor += 1;
                    let mut val_str = String::new();
                    if cursor < bytes.len() && (bytes[cursor] == b'"' || bytes[cursor] == b'\'') {
                        let quote = bytes[cursor];
                        cursor += 1;
                        while cursor < bytes.len() && bytes[cursor] != quote {
                            val_str.push(bytes[cursor] as char);
                            cursor += 1;
                        }
                        cursor += 1;
                    }
                    else {
                        while cursor < bytes.len()
                            && bytes[cursor] != b','
                            && bytes[cursor] != b')'
                            && !(bytes[cursor] as char).is_whitespace()
                        {
                            val_str.push(bytes[cursor] as char);
                            cursor += 1;
                        }
                    }

                    if name.starts_with(':') || name.starts_with('@') {
                        is_dynamic = true;
                        is_directive = true;
                        value_ast = hxo_parser_expression::parse_expression(&val_str).ok();
                    }
                    value = Some(val_str);
                }

                attributes.push(AttributeIR { name, value, value_ast, is_directive, is_dynamic, span: Span::unknown() });

                if cursor < bytes.len() && bytes[cursor] == b',' {
                    cursor += 1;
                }
                while cursor < bytes.len() && (bytes[cursor] as char).is_whitespace() {
                    cursor += 1;
                }
            }
            if cursor < bytes.len() && bytes[cursor] == b')' {
                cursor += 1;
            }
        }

        // 4. Remaining text
        while cursor < bytes.len() && (bytes[cursor] as char).is_whitespace() {
            cursor += 1;
        }

        let children = if cursor < bytes.len() {
            vec![TemplateNodeIR::Text(line[cursor..].to_string(), Span::unknown())]
        }
        else {
            Vec::new()
        };

        Ok((
            TemplateNodeIR::Element(ElementIR { tag: tag_name, attributes, children, is_static: false, span: Span::unknown() }),
            true,
        ))
    }
}
