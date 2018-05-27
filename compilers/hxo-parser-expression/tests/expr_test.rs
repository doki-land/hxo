use hxo_ir::JsExpr;
use hxo_parser_expression::parse_expression;

#[test]
fn test_parse_complex_expr() {
    let expr = parse_expression("a + b * c").unwrap();
    if let JsExpr::Binary { op, .. } = expr {
        assert_eq!(op, "+");
    }
    else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_parse_member_call() {
    let expr = parse_expression("console.log('hello')").unwrap();
    if let JsExpr::Call { .. } = expr {
        // OK
    }
    else {
        panic!("Expected call expression");
    }
}

#[test]
fn test_parse_arrow() {
    let expr = parse_expression("(x) => x + 1").unwrap();
    if let JsExpr::ArrowFunction { .. } = expr {
        // OK
    }
    else {
        panic!("Expected arrow function");
    }
}

#[test]
fn test_parse_tsx() {
    let expr = parse_expression("<div class=\"container\">{title}</div>").unwrap();
    if let JsExpr::TseElement { tag, attributes, children, .. } = expr {
        assert_eq!(tag, "div");
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].name, "class");
        assert_eq!(children.len(), 1);
    }
    else {
        panic!("Expected TSX element");
    }
}
