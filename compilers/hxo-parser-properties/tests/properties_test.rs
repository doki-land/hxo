use hxo_parser_properties::PropertiesParser;
use hxo_parser::{MetadataSubParser, ParseState};
use hxo_types::HxoValue;

#[test]
fn test_parse_properties_basic() {
    let source = r#"
        hello = Hello, world!
        welcome : Welcome to HXO
        app.name = My App
    "#;
    let parser = PropertiesParser;
    let mut state = ParseState::new(source);
    let result = parser.parse(&mut state, "properties").unwrap();
    
    if let HxoValue::Object(map) = result {
        assert_eq!(map.get("hello").unwrap().as_str().unwrap(), "Hello, world!");
        assert_eq!(map.get("welcome").unwrap().as_str().unwrap(), "Welcome to HXO");
        assert_eq!(map.get("app.name").unwrap().as_str().unwrap(), "My App");
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_parse_properties_comments() {
    let source = r#"
        # This is a comment
        ! This is also a comment
        key = value
    "#;
    let parser = PropertiesParser;
    let mut state = ParseState::new(source);
    let result = parser.parse(&mut state, "properties").unwrap();
    
    if let HxoValue::Object(map) = result {
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("key").unwrap().as_str().unwrap(), "value");
    } else {
        panic!("Expected object");
    }
}
