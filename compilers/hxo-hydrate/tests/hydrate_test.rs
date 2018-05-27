use hxo_hydrate::HydrateBackend;
use hxo_ir::{AttributeIR, ElementIR, IRModule, TemplateIR, TemplateNodeIR};
use hxo_types::Span;
use std::collections::HashMap;

#[test]
fn test_hydrate_generation() {
    let backend = HydrateBackend::new();
    let ir = IRModule {
        name: "Test".to_string(),
        metadata: HashMap::new(),
        script: None,
        script_meta: None,
        template: Some(TemplateIR {
            nodes: vec![TemplateNodeIR::Element(ElementIR {
                tag: "button".to_string(),
                attributes: vec![AttributeIR {
                    name: "@click".to_string(),
                    value: Some("handleClick".to_string()),
                    value_ast: None,
                    is_dynamic: false,
                    is_directive: true,
                    span: Span::default(),
                }],
                children: vec![TemplateNodeIR::Text("Click me".to_string(), Span::default())],
                is_static: false,
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

    let hydrate_js = backend.generate(&ir).unwrap();
    assert!(hydrate_js.contains("export function hydrate(root, ctx)"));
    assert!(hydrate_js.contains("const el0 = root.querySelector('[data-hxo-id=\"0\"]');"));
    assert!(hydrate_js.contains("el0.addEventListener('click', () => handleClick);"));
}
