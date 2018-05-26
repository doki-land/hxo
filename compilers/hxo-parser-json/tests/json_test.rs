use hxo_parser_json::parse;
use hxo_types::{HxoValue, Position};

fn default_pos() -> Position {
    Position { line: 1, column: 1, offset: 0 }
}

#[test]
fn test_parse_simple_router() {
    let json = r#"{
  "routes": [
    {
      "path": "/",
      "component": "./views/Home.hxo",
      "name": "home"
    },
    {
      "path": "/about",
      "component": "./views/About.hxo"
    }
  ],
  "mode": "history"
}"#;
    let config = parse(json, default_pos()).expect("Failed to parse JSON");
    assert_eq!(config.routes.len(), 2);
    assert_eq!(config.routes[0].path, "/");
    assert_eq!(config.routes[0].component, "./views/Home.hxo");
    assert_eq!(config.routes[0].name.as_deref(), Some("home"));
    assert_eq!(config.mode, "history");

    // Check spans
    assert!(config.span.start.offset < config.span.end.offset);
}

#[test]
fn test_parse_nested_router() {
    let json = r#"{
  "routes": [
    {
      "path": "/user",
      "component": "./layouts/UserLayout.hxo",
      "children": [
        {
          "path": "profile",
          "component": "./views/UserProfile.hxo"
        }
      ]
    }
  ]
}"#;
    let config = parse(json, default_pos()).expect("Failed to parse nested JSON");
    assert_eq!(config.routes.len(), 1);
    let user_route = &config.routes[0];
    assert_eq!(user_route.children.as_ref().unwrap().len(), 1);
    assert_eq!(user_route.children.as_ref().unwrap()[0].path, "profile");
}

#[test]
fn test_parse_with_meta() {
    let json = r#"{
  "routes": [
    {
      "path": "/admin",
      "component": "./views/Admin.hxo",
      "meta": {
        "requiresAuth": true,
        "roles": ["admin", "editor"]
      }
    }
  ]
}"#;
    let config = parse(json, default_pos()).expect("Failed to parse JSON with meta");
    assert_eq!(config.routes.len(), 1);
    let meta = config.routes[0].meta.as_ref().unwrap();
    if let HxoValue::Object(obj) = meta {
        assert_eq!(obj.get("requiresAuth"), Some(&HxoValue::Bool(true)));
        if let Some(HxoValue::Array(roles)) = obj.get("roles") {
            assert_eq!(roles.len(), 2);
            assert_eq!(roles[0], HxoValue::String("admin".to_string()));
        }
        else {
            panic!("Expected roles array");
        }
    }
    else {
        panic!("Expected meta object");
    }
}
