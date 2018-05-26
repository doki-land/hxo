use hxo_ir::{AttributeIR, ElementIR, IRModule, JsExpr, JsProgram, JsStmt, TemplateIR, TemplateNodeIR};
use hxo_target_js::JsBackend;
use hxo_types::{HxoValue, Span};
use std::collections::HashMap;

#[test]
fn test_generate_simple_component() {
    let ir = IRModule {
        name: "MyComponent".to_string(),
        metadata: HashMap::new(),
        script: None,
        script_meta: None,
        template: None,
        styles: vec![],
        i18n: None,
        wasm: vec![],
        custom_blocks: vec![],
        span: Span::default(),
    };
    let backend = JsBackend::new(false, false, None);
    let result = backend.generate(&ir).expect("Failed to generate JS");

    assert!(result.0.contains("export default"));
    assert!(result.0.contains("name: 'MyComponent'"));
}

#[test]
fn test_generate_simple() {
    let ir = IRModule {
        name: "Test".to_string(),
        metadata: HashMap::new(),
        script: Some(JsProgram {
            body: vec![JsStmt::VariableDecl {
                kind: "const".to_string(),
                id: "count".to_string(),
                init: Some(JsExpr::Literal(HxoValue::Number(0.0), Span::default())),
                span: Span::default(),
            }],
            span: Span::default(),
        }),
        script_meta: None,
        template: None,
        styles: Vec::new(),
        i18n: None,
        wasm: Vec::new(),
        custom_blocks: Vec::new(),
        span: Span::default(),
    };

    let backend = JsBackend::new(false, false, None);
    let output = backend.generate(&ir).unwrap();
    // Since no template is used, no core/dom imports should be generated
    assert!(!output.0.contains("import {"));
    assert!(output.0.contains("const count = 0;"));
    assert!(output.0.contains("export default {"));
}

#[test]
fn test_generate_with_template() {
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
                    is_directive: false,
                    is_dynamic: false,
                    span: Span::default(),
                }],
                children: vec![TemplateNodeIR::Text("Hello".to_string(), Span::default())],
                is_static: true,
                span: Span::default(),
            })],
            span: Span::default(),
        }),
        styles: Vec::new(),
        i18n: None,
        wasm: Vec::new(),
        custom_blocks: Vec::new(),
        span: Span::default(),
    };

    let backend = JsBackend::new(false, false, None);
    let output = backend.generate(&ir).unwrap();

    // Should have hoisted the static div
    assert!(output.0.contains("const _hoisted_1 = /*#__PURE__*/ createTextVNode('Hello');"));
    assert!(output.0.contains("const _hoisted_2 = /*#__PURE__*/ h('div', { 'class': 'container' }, ["));
    assert!(output.0.contains("import { createTextVNode, h } from '@hxo/dom';"));
    assert!(output.0.contains("render(ctx) {"));
    assert!(output.0.contains("return _hoisted_2;"));
}
