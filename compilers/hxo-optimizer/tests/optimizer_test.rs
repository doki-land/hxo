use hxo_ir::{ElementIR, ExpressionIR, IRModule, JsExpr, TemplateIR, TemplateNodeIR};
use hxo_optimizer::Optimizer;
use hxo_types::{HxoValue, Span};
use std::collections::HashMap;

#[test]
fn test_optimize_i18n() {
    let mut messages = HashMap::new();
    messages.insert("hello".to_string(), "你好".to_string());

    let mut ir = IRModule {
        name: "Test".to_string(),
        metadata: HashMap::new(),
        script: None,
        script_meta: None,
        template: Some(TemplateIR {
            nodes: vec![TemplateNodeIR::Interpolation(ExpressionIR {
                code: "$t('hello')".to_string(),
                ast: Some(JsExpr::Call {
                    callee: Box::new(JsExpr::Identifier("$t".to_string(), Span::unknown())),
                    args: vec![JsExpr::Literal(HxoValue::String("hello".to_string()), Span::unknown())],
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            })],
            span: Span::unknown(),
        }),
        styles: vec![],
        i18n: None,
        wasm: vec![],
        custom_blocks: vec![],
        span: Span::unknown(),
    };

    let optimizer = Optimizer::new();
    optimizer.optimize_i18n(&mut ir, &messages);

    if let Some(template) = &ir.template {
        if let TemplateNodeIR::Interpolation(expr) = &template.nodes[0] {
            assert_eq!(expr.code, "'你好'");
            if let Some(JsExpr::Literal(HxoValue::String(s), _)) = &expr.ast {
                assert_eq!(s, "你好");
            }
            else {
                panic!("Expected literal string in AST");
            }
        }
        else {
            panic!("Expected interpolation node");
        }
    }
    else {
        panic!("Expected template");
    }
}

#[test]
fn test_optimize_static_element() {
    let mut ir = IRModule {
        name: "Test".to_string(),
        metadata: HashMap::new(),
        script: None,
        script_meta: None,
        template: Some(TemplateIR {
            nodes: vec![TemplateNodeIR::Element(ElementIR {
                tag: "div".to_string(),
                attributes: vec![],
                children: vec![TemplateNodeIR::Text("Static".to_string(), Span::unknown())],
                is_static: false, // Initially false
                span: Span::unknown(),
            })],
            span: Span::unknown(),
        }),
        styles: vec![],
        i18n: None,
        wasm: vec![],
        custom_blocks: vec![],
        span: Span::unknown(),
    };

    let mut optimizer = Optimizer::new();
    optimizer.optimize(&mut ir, None, false);

    if let Some(template) = &ir.template {
        if let TemplateNodeIR::Element(el) = &template.nodes[0] {
            assert!(el.is_static, "Div with static text should be optimized to is_static: true");
        }
        else {
            panic!("Expected element node");
        }
    }
    else {
        panic!("Expected template");
    }
}
