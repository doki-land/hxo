use hxo_parser_css::compile;

#[test]
fn test_parse_css() {
    let css = ".test { color: red; }";
    let res = compile(css).unwrap();
    assert!(res.contains("color: red"));
}
