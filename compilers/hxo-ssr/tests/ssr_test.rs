use hxo_ir::{AttributeIR, ElementIR, IRModule, TemplateIR, TemplateNodeIR};
use hxo_ssr::SsrBackend;
use hxo_types::Span;
use std::collections::HashMap;

#[test]
fn test_ssr_generation() {
    let backend = SsrBackend::new();
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

    let ssr_js = backend.generate(&ir).unwrap();
    assert!(ssr_js.contains("export function render(ctx)"));
    assert!(ssr_js.contains("html += '<div class=\"container\">';"));
    assert!(ssr_js.contains("html += 'Hello World';"));
    assert!(ssr_js.contains("html += '</div>';"));
}
