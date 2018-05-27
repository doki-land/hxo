use hxo_parser::{Parser, ParserRegistry};
use hxo_parser_markdown::{MarkdownParser, parse};
use std::sync::Arc;

#[test]
fn test_parse_markdown() {
    let md = "# Hello\n\nThis is **bold** and *italic* and `code`";
    let res = parse(md).unwrap();
    assert!(res.contains("<h1>Hello</h1>"));
    assert!(res.contains("<strong>bold</strong>"));
    assert!(res.contains("<em>italic</em>"));
    assert!(res.contains("<code>code</code>"));
}

#[test]
fn test_parse_markdown_links_images() {
    let md = "[Link](https://hxo.dev) and ![Image](src.png)";
    let html = parse(md).expect("Should parse MD links/images");
    assert!(html.contains("<a href=\"https://hxo.dev\">Link</a>"));
    assert!(html.contains("<img src=\"src.png\" alt=\"Image\">"));
}

#[test]
fn test_markdown_integration() {
    let source = r#"
<template lang="markdown">
# Hello Markdown
</template>
"#;
    let mut registry = ParserRegistry::new();
    registry.register_template_parser("markdown", Arc::new(MarkdownParser));
    let registry = Arc::new(registry);

    let mut parser = Parser::new("Test".to_string(), source, registry);
    let parsed = parser.parse_all().expect("Failed to parse");

    let template = parsed.template.expect("Should have template");
    assert_eq!(template.nodes.len(), 1);

    if let hxo_ir::TemplateNodeIR::Element(el) = &template.nodes[0] {
        assert_eq!(el.tag, "h1");
        if let hxo_ir::TemplateNodeIR::Text(text, _) = &el.children[0] {
            assert_eq!(text, "Hello Markdown");
        }
        else {
            panic!("Expected text child");
        }
    }
    else {
        panic!("Expected h1 element");
    }
}
