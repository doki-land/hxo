use hxo_source_map::SourceMapBuilder;
use hxo_types::Position;

#[test]
fn test_source_map_builder() {
    let mut builder = SourceMapBuilder::new();

    let gen_pos = Position { line: 1, column: 1, offset: 0 };
    let orig_pos = Position { line: 1, column: 1, offset: 0 };
    let source_file = Some("test.hxo".to_string());
    let name = Some("test".to_string());

    builder.add_mapping(gen_pos, orig_pos, source_file.clone(), name.clone());

    let map = builder.finish();

    assert_eq!(map.sources.len(), 1);
    assert_eq!(map.sources[0], "test.hxo");
    assert_eq!(map.names.len(), 1);
    assert_eq!(map.names[0], "test");
    assert_eq!(map.mappings.len(), 1);
}
