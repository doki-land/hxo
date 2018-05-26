use hxo_parser_pug::parse;

#[test]
fn test_parse_pug() {
    let pug = "p Hello";
    let html = parse(pug).unwrap();
    assert!(html.contains("<!-- Converted from Pug -->"));
    assert!(html.contains("<div>p Hello</div>"));
}
