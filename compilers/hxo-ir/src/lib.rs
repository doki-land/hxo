use hxo_types::{HxoValue, Span};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsExpr {
    Identifier(String, #[serde(default)] Span),
    Literal(HxoValue, #[serde(default)] Span),
    Binary {
        left: Box<JsExpr>,
        op: String,
        right: Box<JsExpr>,
        #[serde(default)]
        span: Span,
    },
    Call {
        callee: Box<JsExpr>,
        args: Vec<JsExpr>,
        #[serde(default)]
        span: Span,
    },
    Member {
        object: Box<JsExpr>,
        property: String,
        computed: bool,
        #[serde(default)]
        span: Span,
    },
    Array(Vec<JsExpr>, #[serde(default)] Span),
    Object(HashMap<String, JsExpr>, #[serde(default)] Span),
    ArrowFunction {
        params: Vec<String>,
        body: Box<JsExpr>,
        #[serde(default)]
        span: Span,
    },
    TseElement {
        tag: String,
        attributes: Vec<TseAttribute>,
        children: Vec<JsExpr>,
        #[serde(default)]
        span: Span,
    },
    Other(String, #[serde(default)] Span),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TseAttribute {
    pub name: String,
    pub value: Option<JsExpr>,
    #[serde(default)]
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsStmt {
    Expr(JsExpr, #[serde(default)] Span),
    VariableDecl {
        kind: String, // var, let, const
        id: String,
        init: Option<JsExpr>,
        #[serde(default)]
        span: Span,
    },
    Import {
        source: String,
        specifiers: Vec<String>,
        #[serde(default)]
        span: Span,
    },
    Export {
        declaration: Box<JsStmt>,
        #[serde(default)]
        span: Span,
    },
    ExportAll {
        source: String,
        #[serde(default)]
        span: Span,
    },
    ExportNamed {
        source: Option<String>,
        specifiers: Vec<String>,
        #[serde(default)]
        span: Span,
    },
    FunctionDecl {
        id: String,
        params: Vec<String>,
        body: Vec<JsStmt>,
        #[serde(default)]
        span: Span,
    },
    Other(String, #[serde(default)] Span),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsProgram {
    pub body: Vec<JsStmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IRModule {
    pub name: String,
    pub metadata: HashMap<String, HxoValue>,
    pub script: Option<JsProgram>,
    pub script_meta: Option<HxoValue>, // For script analysis results
    pub template: Option<TemplateIR>,
    pub styles: Vec<StyleIR>,
    pub i18n: Option<HashMap<String, HashMap<String, String>>>,
    pub wasm: Vec<Vec<u8>>,
    pub custom_blocks: Vec<CustomBlockIR>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateIR {
    pub nodes: Vec<TemplateNodeIR>,
    #[serde(default)]
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemplateNodeIR {
    Element(ElementIR),
    Text(String, #[serde(default)] Span),
    Interpolation(ExpressionIR),
    Comment(String, #[serde(default)] Span),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementIR {
    pub tag: String,
    pub attributes: Vec<AttributeIR>,
    pub children: Vec<TemplateNodeIR>,
    pub is_static: bool,
    #[serde(default)]
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeIR {
    pub name: String,
    pub value: Option<String>,
    pub value_ast: Option<JsExpr>,
    pub is_directive: bool,
    pub is_dynamic: bool,
    #[serde(default)]
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionIR {
    pub code: String,
    pub ast: Option<JsExpr>,
    #[serde(default)]
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleIR {
    pub code: String,
    pub lang: String,
    pub scoped: bool,
    #[serde(default)]
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomBlockIR {
    pub name: String,
    pub content: String,
    pub attributes: HashMap<String, String>,
    #[serde(default)]
    pub span: Span,
}
