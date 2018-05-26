use hxo_parser_tailwind::StyleEngine;
use hxo_types::{Position, Span};

#[test]
fn test_parse_classes() {
    let mut engine = StyleEngine::new();
    let span = Span { start: Position { line: 1, column: 1, offset: 0 }, end: Position { line: 1, column: 1, offset: 0 } };
    engine.parse_classes("m-4 p-2 rounded-lg bg-blue text-white", span).unwrap();

    let css = engine.generate_css();
    assert!(css.contains(".m-4"));
    assert!(css.contains("margin: 1rem;"));
    assert!(css.contains(".p-2"));
    assert!(css.contains("padding: 0.5rem;"));
    assert!(css.contains(".rounded-lg"));
    assert!(css.contains("border-radius: 0.5rem;"));
    assert!(css.contains(".bg-blue"));
    assert!(css.contains("background-color: #0000ff;"));
    assert!(css.contains(".text-white"));
    assert!(css.contains("color: #ffffff;"));
}

#[test]
fn test_position_tracking() {
    let mut engine = StyleEngine::new();
    let span = Span { start: Position { line: 1, column: 10, offset: 10 }, end: Position { line: 1, column: 20, offset: 20 } };
    engine.parse_classes("m-4 text-red", span).unwrap();

    assert_eq!(engine.rules.len(), 2);

    // m-4
    assert_eq!(engine.rules[0].span.start.column, 10);
    assert_eq!(engine.rules[0].span.end.column, 13);

    // text-red
    // current_column += class_len + 1 = 10 + 3 + 1 = 14
    assert_eq!(engine.rules[1].span.start.column, 14);
    assert_eq!(engine.rules[1].span.end.column, 22);
}
