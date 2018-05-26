use hxo_parser_less::compile;

#[test]
fn test_less_basic() {
    let source = r#"
        @primary-color: #4D926F;
        #header {
          color: @primary-color;
        }
        h2 {
          color: @primary-color;
        }
    "#;
    let css = compile(source).unwrap();
    assert!(css.contains("color: #4D926F;"));
    assert!(css.contains("#header {"));
    assert!(css.contains("h2 {"));
}

#[test]
fn test_parse_comments_less() {
    let less = "
        // This is a comment
        @color: red; /* Multi-line
                        comment */
        .test {
            color: @color; // Inline comment
        }
    ";
    let css = compile(less).expect("Should parse LESS with comments");
    assert!(css.contains("color: red;"));
    assert!(css.contains(".test {"));
}

#[test]
fn test_parse_nested_less() {
    let less = ".parent { .child { color: blue; } }";
    let css = compile(less).expect("Should parse nested LESS");
    assert!(css.contains(".parent .child {"));
    assert!(css.contains("color: blue;"));
}
