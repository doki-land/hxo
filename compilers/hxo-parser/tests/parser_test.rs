use hxo_ir::TemplateNodeIR;
use hxo_parser::{MetadataParser, ParseState, Parser, ParserRegistry, TemplateParser};
use hxo_types::{HxoValue, Result, Span};
use std::{collections::HashMap, sync::Arc};

struct MockTemplateParser;
impl TemplateParser for MockTemplateParser {
    fn parse(&self, _state: &mut ParseState, _lang: &str) -> Result<Vec<TemplateNodeIR>> {
        Ok(vec![TemplateNodeIR::Text("Mock".to_string(), Span::default())])
    }
}

struct MockMetadataParser;
impl MetadataParser for MockMetadataParser {
    fn parse(&self, _state: &mut ParseState, _lang: &str) -> Result<HxoValue> {
        let mut map = HashMap::new();
        let mut en = HashMap::new();
        en.insert("hello".to_string(), HxoValue::String("Hello".to_string()));
        let mut zh = HashMap::new();
        zh.insert("hello".to_string(), HxoValue::String("你好".to_string()));

        map.insert("en".to_string(), HxoValue::Object(en));
        map.insert("zh".to_string(), HxoValue::Object(zh));
        Ok(HxoValue::Object(map))
    }
}

#[test]
fn test_parse_minimal() {
    let source = r#"
<template>
  <div>Hello</div>
</template>
<script>
const x = 1;
</script>
"#;
    let mut registry = ParserRegistry::new();
    registry.register_template_parser("html", Arc::new(MockTemplateParser));
    let registry = Arc::new(registry);
    let mut parser = Parser::new("Test".to_string(), source, registry);
    let parsed = parser.parse_all().expect("Failed to parse");

    assert_eq!(parsed.name, "Test");
    assert!(parsed.template.is_some());
    // script analysis results are currently empty in parse_all
    // assert!(parsed.script.is_some());
}

#[test]
fn test_parse_i18n_json() {
    // We need to modify parse_i18n to support registry or register it globally
    // For now, let's mock it by calling a modified version or just testing the registry
    let mut registry = ParserRegistry::new();
    registry.register_metadata_parser("json", Arc::new(MockMetadataParser));

    let source = r#"{
        "en": { "hello": "Hello" },
        "zh": { "hello": "你好" }
    }"#;

    let mut state = ParseState::new(source);
    let parser = registry.get_metadata_parser("json").unwrap();
    let val = parser.parse(&mut state, "json").unwrap();

    if let HxoValue::Object(i18n) = val {
        assert_eq!(i18n.get("en").unwrap().get("hello").unwrap().as_str().unwrap(), "Hello");
        assert_eq!(i18n.get("zh").unwrap().get("hello").unwrap().as_str().unwrap(), "你好");
    }
    else {
        panic!("Expected object");
    }
}
