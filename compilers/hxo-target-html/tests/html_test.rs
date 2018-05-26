use hxo_ir::{AttributeIR, ElementIR, IRModule, TemplateIR, TemplateNodeIR};
use hxo_target_html::HtmlBackend;
use hxo_types::Span;
use std::collections::HashMap;

#[test]
fn test_generate_simple_html() {
    let backend = HtmlBackend::new();
    let ir = IRModule {
        name: "Test".to_string(),
        metadata: HashMap::new(),
        script: None,
        script_meta: None,
        template: Some(TemplateIR {
            nodes: vec![TemplateNodeIR::Element(ElementIR {
                tag: "div".to_string(),
                attributes: vec![AttributeIR {
                    name: "class".to_string(),
                    value: Some("container".to_string()),
                    value_ast: None,
                    is_dynamic: false,
                    is_directive: false,
                    span: Span::default(),
                }],
                children: vec![TemplateNodeIR::Text("Hello World".to_string(), Span::default())],
                is_static: true,
                span: Span::default(),
            })],
            span: Span::default(),
        }),
        styles: vec![],
        i18n: None,
        wasm: vec![],
        custom_blocks: vec![],
        span: Span::default(),
    };

    let html = backend.generate(&ir).unwrap();
    assert!(html.contains("<div class=\"container\">"));
    assert!(html.contains("Hello World"));
    assert!(html.contains("</div>"));
}

#[test]
fn test_self_closing_tags() {
    let backend = HtmlBackend::new();
    let ir = IRModule {
        name: "Test".to_string(),
        metadata: HashMap::new(),
        script: None,
        script_meta: None,
        template: Some(TemplateIR {
            nodes: vec![TemplateNodeIR::Element(ElementIR {
                tag: "img".to_string(),
                attributes: vec![AttributeIR {
                    name: "src".to_string(),
                    value: Some("logo.png".to_string()),
                    value_ast: None,
                    is_dynamic: false,
                    is_directive: false,
                    span: Span::default(),
                }],
                children: vec![],
                is_static: true,
                span: Span::default(),
            })],
            span: Span::default(),
        }),
        styles: vec![],
        i18n: None,
        wasm: vec![],
        custom_blocks: vec![],
        span: Span::default(),
    };

    let html = backend.generate(&ir).unwrap();
    assert!(html.contains("<img src=\"logo.png\" />"));
}
