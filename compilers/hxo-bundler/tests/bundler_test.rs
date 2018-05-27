use hxo_bundler::Bundler;
use hxo_ir::IRModule;
use hxo_types::Span;
use std::collections::HashMap;

#[test]
fn test_bundler_basic() {
    let mut bundler = Bundler::new();
    let modules = vec![IRModule {
        name: "Test".to_string(),
        metadata: HashMap::new(),
        script: None,
        script_meta: None,
        template: None,
        styles: Vec::new(),
        i18n: None,
        wasm: Vec::new(),
        custom_blocks: Vec::new(),
        span: Span::default(),
    }];

    bundler.analyze_all(&modules);
    let runtime = bundler.generate_custom_runtime();
    assert!(runtime.contains("HXO Custom Runtime"));
}
