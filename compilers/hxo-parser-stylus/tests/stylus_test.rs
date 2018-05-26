use hxo_parser_stylus::compile;

#[test]
fn test_stylus_not_implemented() {
    let result = compile("body { color: red }");
    assert!(result.is_err());
}
