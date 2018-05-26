use hxo_types::{HxoBlock, HxoFile};
use std::collections::HashMap;

#[test]
fn test_hxo_file_structure() {
    let mut attributes = HashMap::new();
    attributes.insert("lang".to_string(), "ts".to_string());

    let block = HxoBlock {
        name: "script".to_string(),
        attributes,
        content: "const x = 1;".to_string(),
        span: Default::default(),
        content_span: Default::default(),
    };

    let file = HxoFile { blocks: vec![block], span: Default::default() };

    assert_eq!(file.blocks.len(), 1);
    assert_eq!(file.blocks[0].name, "script");
    assert_eq!(file.blocks[0].attributes.get("lang").unwrap(), "ts");
}
