use hxo_parser_tailwind::StyleEngine;
use hxo_types::Span;

#[test]
fn test_tailwind_engine() {
    let mut engine = StyleEngine::new();
    let source = "p-4 m-2 text-center flex items-center";
    engine.parse_classes(source, Span::unknown()).unwrap();

    let css = engine.generate_css();

    assert!(css.contains(".p-4"));
    assert!(css.contains("padding: 1rem;"));
    assert!(css.contains(".m-2"));
    assert!(css.contains("margin: 0.5rem;"));
    assert!(css.contains(".text-center"));
    assert!(css.contains("text-align: center;"));
    assert!(css.contains(".flex"));
    assert!(css.contains("display: flex;"));
    assert!(css.contains(".items-center"));
    assert!(css.contains("align-items: center;"));
}

#[test]
fn test_add_style() {
    let mut engine = StyleEngine::new();
    engine.add_style("p-4");
    engine.add_style("m-2");

    let css = engine.generate_css();
    assert!(css.contains(".p-4"));
    assert!(css.contains("padding: 1rem;"));
    assert!(css.contains(".m-2"));
    assert!(css.contains("margin: 0.5rem;"));
}

#[test]
fn test_add_styles() {
    let mut engine = StyleEngine::new();
    engine.add_styles("p-4 m-2");

    let css = engine.generate_css();
    assert!(css.contains(".p-4"));
    assert!(css.contains(".m-2"));
}
