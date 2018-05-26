use hxo_parser_scss::{ScssParserOptions, compile};

#[test]
fn test_compile_basic_scss() {
    let scss = "$color: red; .test { color: $color; }";
    let options = ScssParserOptions::default();
    let result = compile(scss, &options).expect("Should compile SCSS");
    assert!(result.contains("color: red"));
    assert!(result.contains(".test"));
}

#[test]
fn test_compile_nested_scss() {
    let scss = ".parent { .child { color: blue; } }";
    let options = ScssParserOptions::default();
    let result = compile(scss, &options).expect("Should compile nested SCSS");
    assert!(result.contains(".parent .child"));
    assert!(result.contains("color: blue"));
}

#[test]
fn test_compile_comments_scss() {
    let scss = "
        // This is a comment
        $color: red; /* Multi-line
                        comment */
        .test {
            color: $color; // Inline comment
        }
    ";
    let options = ScssParserOptions::default();
    let result = compile(scss, &options).expect("Should compile SCSS with comments");
    assert!(result.contains("color: red"));
    assert!(result.contains(".test"));
}
