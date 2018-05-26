use hxo_ir::IRModule;
use hxo_target_dts::DtsBackend;
use hxo_types::{HxoValue, Span};
use std::collections::HashMap;

#[test]
fn test_generate_dts() {
    let backend = DtsBackend::new();
    let mut script_meta = HashMap::new();
    script_meta.insert("props".to_string(), HxoValue::Array(vec![HxoValue::String("count".to_string())]));
    script_meta.insert("emits".to_string(), HxoValue::Array(vec![HxoValue::String("change".to_string())]));

    let ir = IRModule {
        name: "Counter".to_string(),
        metadata: HashMap::new(),
        script: None,
        script_meta: Some(HxoValue::Object(script_meta)),
        template: None,
        styles: vec![],
        i18n: None,
        wasm: vec![],
        custom_blocks: vec![],
        span: Span::default(),
    };

    let dts = backend.generate(&ir).unwrap();
    assert!(dts.contains("export interface CounterProps {"));
    assert!(dts.contains("count?: any;"));
    assert!(dts.contains("export interface CounterEmits {"));
    assert!(dts.contains("(e: 'change', ...args: any[]): void;"));
}
