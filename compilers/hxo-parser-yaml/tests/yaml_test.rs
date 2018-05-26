use hxo_parser_yaml::{parse_yaml, parse_router};
use hxo_types::HxoValue;

#[test]
fn test_parse_basic_yaml() {
    let yaml = "
        name: test
        version: 1.0
        active: true
        meta:
            author: hxo
            tags:
                - rust
                - yaml
    ";
    let result = parse_yaml(yaml).expect("Should parse YAML");
    if let HxoValue::Object(map) = result {
        assert_eq!(map.get("name").and_then(|v| v.as_str()), Some("test"));
        assert_eq!(map.get("active").and_then(|v| v.as_bool()), Some(true));
        if let Some(HxoValue::Object(meta)) = map.get("meta") {
            assert_eq!(meta.get("author").and_then(|v| v.as_str()), Some("hxo"));
            if let Some(HxoValue::Array(tags)) = meta.get("tags") {
                assert_eq!(tags.len(), 2);
                assert_eq!(tags[0].as_str(), Some("rust"));
            }
            else {
                panic!("tags should be an array");
            }
        }
        else {
            panic!("meta should be an object");
        }
    }
    else {
        panic!("result should be an object");
    }
}

#[test]
fn test_parse_simple_router() {
    let yaml = r#"
routes:
  - path: /
    component: ./views/Home.hxo
    name: home
  - path: /about
    component: ./views/About.hxo
mode: history
"#;
    let config = parse_router(yaml).expect("Failed to parse YAML");
    assert_eq!(config.routes.len(), 2);
    assert_eq!(config.routes[0].path, "/");
    assert_eq!(config.routes[0].component, "./views/Home.hxo");
    assert_eq!(config.routes[0].name.as_deref(), Some("home"));
    assert_eq!(config.mode, "history");
}

#[test]
fn test_parse_nested_router() {
    let yaml = r#"
routes:
  - path: /user
    component: ./layouts/UserLayout.hxo
    children:
      - path: profile
        component: ./views/UserProfile.hxo
      - path: settings
        component: ./views/UserSettings.hxo
"#;
    let config = parse_router(yaml).expect("Failed to parse nested YAML");
    assert_eq!(config.routes.len(), 1);
    let user_route = &config.routes[0];
    assert_eq!(user_route.children.as_ref().unwrap().len(), 2);
    assert_eq!(user_route.children.as_ref().unwrap()[0].path, "profile");
}
