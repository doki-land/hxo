use hxo_parser_stylus::compile;

#[test]
fn test_stylus_basic() {
    let stylus = r#"
body
  color red
  margin 10px
"#;
    let expected = "body {\n  color: red;\n  margin: 10px;\n}";
    let result = compile(stylus).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_stylus_nesting() {
    let stylus = r#"
.container
  width 100%
  .header
    color #333
"#;
    let expected = ".container {\n  width: 100%;\n}\n.container .header {\n  color: #333;\n}";
    let result = compile(stylus).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_stylus_variables() {
    let stylus = r#"
main-color = #fff
body
  background main-color
"#;
    let expected = "body {\n  background: #fff;\n}";
    let result = compile(stylus).unwrap();
    assert_eq!(result, expected);
}
