use hxo_parser_sass::{SassParserOptions, compile};

#[test]
fn test_compile_basic_sass() {
    let sass = "$color: red\n.test\n  color: $color";
    let options = SassParserOptions::default();
    let result = compile(sass, &options).expect("Should compile SASS");
    assert!(result.contains("color: red"));
    assert!(result.contains(".test"));
}

#[test]
fn test_compile_nested_sass() {
    let sass = ".parent\n  color: blue\n  .child\n    color: red";
    let options = SassParserOptions::default();
    let result = compile(sass, &options).expect("Should compile SASS");
    assert!(result.contains(".parent {"));
    assert!(result.contains("color: blue;"));
    assert!(result.contains(".parent .child {"));
    assert!(result.contains("color: red;"));
}
