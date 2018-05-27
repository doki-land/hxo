use hxo_ir::TemplateNodeIR;
use hxo_parser_pug::parse;

#[test]
fn test_parse_pug() {
    let pug = "p Hello";
    let nodes = parse(pug).unwrap();

    match &nodes[0] {
        TemplateNodeIR::Element(el) => {
            assert_eq!(el.tag, "p");
            match &el.children[0] {
                TemplateNodeIR::Text(t, _) => assert_eq!(t, "Hello"),
                _ => panic!("Expected text child"),
            }
        }
        _ => panic!("Expected element node"),
    }
}
