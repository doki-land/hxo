use hxo_ir::IRModule;
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
