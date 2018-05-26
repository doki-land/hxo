use hxo_parser::{MetadataSubParser, ParseState};
use hxo_types::{HxoValue, Result, Span};

pub struct TomlParser;

impl TomlParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_to_type<T>(&self, content: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        toml::from_str(content)
            .map_err(|e| hxo_types::Error::external_error("TOML".to_string(), e.to_string(), hxo_types::Span::unknown()))
    }
}

impl MetadataSubParser for TomlParser {
    fn parse(&self, state: &mut ParseState, _lang: &str) -> Result<HxoValue> {
        let content = state.cursor.source;
        let value: serde_json::Value = toml::from_str(content)
            .map_err(|e| hxo_types::Error::external_error("TOML".to_string(), e.to_string(), Span::unknown()))?;
        
        Ok(convert_json_to_hxo(value))
    }
}

fn convert_json_to_hxo(value: serde_json::Value) -> HxoValue {
    match value {
        serde_json::Value::Null => HxoValue::Null,
        serde_json::Value::Bool(b) => HxoValue::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                HxoValue::Number(f)
            } else {
                HxoValue::Number(0.0)
            }
        }
        serde_json::Value::String(s) => HxoValue::String(s),
        serde_json::Value::Array(a) => HxoValue::Array(a.into_iter().map(convert_json_to_hxo).collect()),
        serde_json::Value::Object(o) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in o {
                map.insert(k, convert_json_to_hxo(v));
            }
            HxoValue::Object(map)
        }
    }
}
