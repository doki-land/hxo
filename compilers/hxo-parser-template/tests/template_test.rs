use hxo_ir::TemplateNodeIR;
use hxo_parser_template::parse;

#[test]
fn test_parse_basic() {
    let source = r#"<div class="container">Hello {{ name }}!</div>"#;
    let nodes = parse(source).unwrap();
    assert_eq!(nodes.len(), 1);
    if let TemplateNodeIR::Element(el) = &nodes[0] {
        assert_eq!(el.tag, "div");
        assert_eq!(el.attributes.iter().find(|a| a.name == "class").unwrap().value.as_ref().unwrap(), "container");
        assert_eq!(el.children.len(), 3);
        if let TemplateNodeIR::Interpolation(expr) = &el.children[1] {
            assert_eq!(expr.code, "name");
        }
        else {
            panic!("Expected interpolation");
        }
    }
}

#[test]
fn test_parse_void_elements() {
    let source = r#"<div><br><img src="test.png"></div>"#;
    let nodes = parse(source).unwrap();
    assert_eq!(nodes.len(), 1);
    if let TemplateNodeIR::Element(el) = &nodes[0] {
        assert_eq!(el.children.len(), 2);
        if let TemplateNodeIR::Element(br) = &el.children[0] {
            assert_eq!(br.tag, "br");
            assert!(br.children.is_empty());
        }
    }
}

#[test]
fn test_parse_comment() {
    let source = r#"<div><!-- this is a comment --></div>"#;
    let nodes = parse(source).unwrap();
    if let TemplateNodeIR::Element(el) = &nodes[0] {
        if let TemplateNodeIR::Comment(c, _) = &el.children[0] {
            assert_eq!(c, " this is a comment ");
        }
    }
}

#[test]
fn test_parse_script_style() {
    let source = r#"
        <script>
            console.log("hello");
            if (a < b) { doSomething(); }
        </script>
        <style>
            .test { color: red; }
        </style>
    "#;
    let nodes = parse(source).unwrap();
    // Filter out whitespace text nodes
    let nodes: Vec<_> =
        nodes.into_iter().filter(|n| if let TemplateNodeIR::Text(t, _) = n { !t.trim().is_empty() } else { true }).collect();

    assert_eq!(nodes.len(), 2);
    if let TemplateNodeIR::Element(script) = &nodes[0] {
        assert_eq!(script.tag, "script");
        if let TemplateNodeIR::Text(t, _) = &script.children[0] {
            assert!(t.contains("console.log"));
        }
    }
    if let TemplateNodeIR::Element(style) = &nodes[1] {
        assert_eq!(style.tag, "style");
        if let TemplateNodeIR::Text(t, _) = &style.children[0] {
            assert!(t.contains(".test"));
        }
    }
}

#[test]
fn test_parse_unquoted_attr() {
    let source = r#"<div class=container id=main></div>"#;
    let nodes = parse(source).unwrap();
    assert_eq!(nodes.len(), 1);
    if let TemplateNodeIR::Element(el) = &nodes[0] {
        assert_eq!(el.attributes.iter().find(|a| a.name == "class").unwrap().value.as_ref().unwrap(), "container");
        assert_eq!(el.attributes.iter().find(|a| a.name == "id").unwrap().value.as_ref().unwrap(), "main");
    }
}
