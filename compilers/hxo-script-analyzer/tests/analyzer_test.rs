use hxo_ir::{JsExpr, JsProgram, JsStmt};
use hxo_script_analyzer::ScriptAnalyzer;
use hxo_types::{HxoValue, Span};

#[test]
fn test_analyze_signals() {
    let program = JsProgram {
        body: vec![
            JsStmt::VariableDecl {
                kind: "const".to_string(),
                id: "[count, setCount]".to_string(),
                init: Some(JsExpr::Call {
                    callee: Box::new(JsExpr::Identifier("createSignal".to_string(), Span::default())),
                    args: vec![JsExpr::Literal(HxoValue::Number(0.0), Span::default())],
                    span: Span::default(),
                }),
                span: Span::default(),
            },
            JsStmt::VariableDecl {
                kind: "const".to_string(),
                id: "doubleCount".to_string(),
                init: Some(JsExpr::Call {
                    callee: Box::new(JsExpr::Identifier("createComputed".to_string(), Span::default())),
                    args: vec![],
                    span: Span::default(),
                }),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };

    let analyzer = ScriptAnalyzer::new();
    let meta = analyzer.analyze(&program).unwrap();

    assert!(meta.signals.contains("count"));
    assert!(meta.signals.contains("doubleCount"));
    assert_eq!(meta.signals.len(), 2);
}
