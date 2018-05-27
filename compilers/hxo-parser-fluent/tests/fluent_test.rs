use hxo_parser::{MetadataParser, ParseState};
use hxo_parser_fluent::FluentParser;
use hxo_types::HxoValue;

#[test]
fn test_parse_fluent_basic() {
    let source = r#"
hello = Hello, world!
welcome = Welcome to HXO
    "#;
    let parser = FluentParser;
    let mut state = ParseState::new(source);
    let result = parser.parse(&mut state, "fluent").unwrap();

    if let HxoValue::Object(map) = result {
        assert_eq!(map.get("hello").unwrap().as_str().unwrap(), "Hello, world!");
        assert_eq!(map.get("welcome").unwrap().as_str().unwrap(), "Welcome to HXO");
    }
    else {
        panic!("Expected object");
    }
}

#[test]
fn test_parse_fluent_with_attributes() {
    let source = r#"
login = Login
    .aria-label = Login to your account
    .title = Please login
    "#;
    let parser = FluentParser;
    let mut state = ParseState::new(source);
    let result = parser.parse(&mut state, "fluent").unwrap();

    if let HxoValue::Object(map) = result {
        let login = map.get("login").unwrap();
        if let HxoValue::Object(login_map) = login {
            assert_eq!(login_map.get("val").unwrap().as_str().unwrap(), "Login");
            assert_eq!(login_map.get("aria-label").unwrap().as_str().unwrap(), "Login to your account");
            assert_eq!(login_map.get("title").unwrap().as_str().unwrap(), "Please login");
        }
        else {
            panic!("Expected object for login");
        }
    }
    else {
        panic!("Expected object");
    }
}

#[test]
fn test_parse_fluent_comments() {
    let source = r#"
# This is a comment
hello = Hello
# Another comment
world = World
    "#;
    let parser = FluentParser;
    let mut state = ParseState::new(source);
    let result = parser.parse(&mut state, "fluent").unwrap();

    if let HxoValue::Object(map) = result {
        assert_eq!(map.get("hello").unwrap().as_str().unwrap(), "Hello");
        assert_eq!(map.get("world").unwrap().as_str().unwrap(), "World");
    }
    else {
        panic!("Expected object");
    }
}
