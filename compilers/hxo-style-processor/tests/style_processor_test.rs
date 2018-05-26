use hxo_style_processor::StyleProcessor;

#[test]
fn test_process_style() {
    let processor = StyleProcessor::new();
    let css = ".test { color: red; }";
    let processed = processor.process(css).unwrap();

    assert!(processed.contains(".test { color: red; }"));
    assert!(processed.contains("/* Processed by HXO Style Processor */"));
}
