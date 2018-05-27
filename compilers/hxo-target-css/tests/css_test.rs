use hxo_ir::{IRModule, StyleIR};
use hxo_target_css::CssBackend;
use hxo_types::Span;
use std::collections::HashMap;

#[test]
fn test_generate_css() {
    let mut ir = IRModule {
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
    ir.styles.push(StyleIR {
        lang: "css".to_string(),
        code: ".test { color: red; }".to_string(),
        scoped: false,
        span: Span::default(),
    });

    let backend = CssBackend::new(false);
    let result = backend.generate(&ir).expect("Failed to generate CSS");

    assert!(result.contains(".test { color: red; }"));
    assert!(result.contains("/* CSS (lang: css) */"));
}

#[test]
fn test_generate_scoped_css() {
    let mut ir = IRModule {
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
    ir.styles.push(StyleIR {
        lang: "css".to_string(),
        code: ".test { color: blue; }".to_string(),
        scoped: true,
        span: Span::default(),
    });

    let backend = CssBackend::new(false);
    let result = backend.generate(&ir).expect("Failed to generate CSS");

    assert!(result.contains(".test { color: blue; }"));
    assert!(result.contains("/* Scoped CSS (lang: css) */"));
}
