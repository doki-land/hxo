use hxo_ir::{AttributeIR, ElementIR, IRModule, TemplateIR, TemplateNodeIR};
use hxo_target_wasm::WasmBackend;
use hxo_types::Span;

#[test]
fn test_wasm_backend_generation() {
    let ir = IRModule {
        name: "TestComponent".to_string(),
        metadata: std::collections::HashMap::new(),
        script: None,
        script_meta: None,
        template: None,
        styles: vec![],
        i18n: None,
        wasm: vec![],
        custom_blocks: vec![],
        span: Span::default(),
    };

    let backend = WasmBackend::new(true);
    let result = backend.generate_wat(&ir);

    assert!(result.is_ok());
    let wat = result.unwrap();
    assert!(wat.contains("(module"));
    assert!(wat.contains("TestComponent"));
}

#[test]
fn test_wasm_backend_complex_template() {
    let ir = IRModule {
        name: "ComplexComponent".to_string(),
        metadata: std::collections::HashMap::new(),
        script: None,
        script_meta: None,
        template: Some(TemplateIR {
            nodes: vec![TemplateNodeIR::Element(ElementIR {
                tag: "div".to_string(),
                attributes: vec![AttributeIR {
                    name: "class".to_string(),
                    value: Some("container".to_string()),
                    value_ast: None,
                    is_directive: false,
                    is_dynamic: false,
                    span: Span::default(),
                }],
                children: vec![TemplateNodeIR::Text("Hello WASM".to_string(), Span::default())],
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

    let backend = WasmBackend::new(true);
    let result = backend.generate_wat(&ir);

    assert!(result.is_ok());
    let wat = result.unwrap();

    // Check for imports
    assert!(wat.contains("(import \"hxo\" \"create_element\""));
    assert!(wat.contains("(import \"hxo\" \"set_attribute\""));
    assert!(wat.contains("(import \"hxo\" \"create_text\""));
    assert!(wat.contains("(import \"hxo\" \"append_child\""));

    // Check for data section (strings)
    assert!(wat.contains("\"div\""));
    assert!(wat.contains("\"class\""));
    assert!(wat.contains("\"container\""));
    assert!(wat.contains("\"Hello WASM\""));

    // Check for calls in render function
    assert!(wat.contains("call $create_element"));
    assert!(wat.contains("call $set_attribute"));
    assert!(wat.contains("call $create_text"));
    assert!(wat.contains("call $append_child"));
}
