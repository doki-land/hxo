use hxo_parser_markdown::parse;

#[test]
fn test_parse_markdown() {
    let md = "# Hello\n\nThis is **bold** and *italic* and `code`";
    let res = parse(md).unwrap();
    assert!(res.contains("<h1>Hello</h1>"));
    assert!(res.contains("<strong>bold</strong>"));
    assert!(res.contains("<em>italic</em>"));
    assert!(res.contains("<code>code</code>"));
}

#[test]
fn test_parse_markdown_links_images() {
    let md = "[Link](https://hxo.dev) and ![Image](src.png)";
    let html = parse(md).expect("Should parse MD links/images");
    assert!(html.contains("<a href=\"https://hxo.dev\">Link</a>"));
    assert!(html.contains("<img src=\"src.png\" alt=\"Image\">"));
}
