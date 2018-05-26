use hxo_ir::IRModule;
use hxo_target_map::MapBackend;
use hxo_types::Span;
use std::collections::HashMap;

#[test]
fn test_generate_source_map() {
    let backend = MapBackend::new();
    let ir = IRModule {
        name: "Test".to_string(),
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

    let map = backend.generate(&ir).unwrap();
    assert!(!map.sources.is_empty());
    assert_eq!(map.sources[0], "Test.hxo");
    assert!(!map.mappings.is_empty());
}
