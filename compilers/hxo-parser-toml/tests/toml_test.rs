use hxo_parser::{MetadataParser, ParseState};
use hxo_parser_toml::TomlParser;
use hxo_types::HxoValue;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
struct Config {
    name: String,
    version: String,
}

#[test]
fn test_parse_toml_to_type() {
    let parser = TomlParser::new();
    let toml = r#"
        name = "hxo"
        version = "0.1.0"
    "#;
    let config: Config = parser.parse_to_type(toml).unwrap();
    assert_eq!(config.name, "hxo");
    assert_eq!(config.version, "0.1.0");
}

#[test]
fn test_parse_toml_as_metadata() {
    let parser = TomlParser::new();
    let source = r#"
        [en]
        hello = "Hello"
        [zh]
        hello = "你好"
    "#;
    let mut state = ParseState::new(source);
    let val = parser.parse(&mut state, "toml").unwrap();

    if let HxoValue::Object(map) = val {
        let en_val = map.get("en").unwrap();
        if let HxoValue::Object(en) = en_val {
            assert_eq!(en.get("hello").unwrap().as_str().unwrap(), "Hello");
        }
        else {
            panic!("Expected object for 'en'");
        }

        let zh_val = map.get("zh").unwrap();
        if let HxoValue::Object(zh) = zh_val {
            assert_eq!(zh.get("hello").unwrap().as_str().unwrap(), "你好");
        }
        else {
            panic!("Expected object for 'zh'");
        }
    }
    else {
        panic!("Expected object");
    }
}
