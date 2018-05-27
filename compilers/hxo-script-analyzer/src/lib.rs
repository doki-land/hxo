use hxo_ir::{JsExpr, JsProgram, JsStmt};
use hxo_types::Result;
use std::collections::HashSet;

pub struct ScriptMetadata {
    pub signals: HashSet<String>,
    pub props: HashSet<String>,
    pub emits: HashSet<String>,
}

#[derive(Default)]
pub struct ScriptAnalyzer;

impl ScriptAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, program: &JsProgram) -> Result<ScriptMetadata> {
        let mut signals = HashSet::new();
        let props = HashSet::new();
        let emits = HashSet::new();

        for stmt in &program.body {
            if let JsStmt::VariableDecl { id, init, .. } = stmt {
                // Handle destructuring like [count, setCount] = createSignal(0)
                if id.starts_with('[') && id.ends_with(']') {
                    let content = &id[1..id.len() - 1];
                    let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
                    if let Some(JsExpr::Call { callee, .. }) = init {
                        if let JsExpr::Identifier(name, _) = &**callee {
                            if name == "createSignal" && !parts.is_empty() {
                                signals.insert(parts[0].to_string());
                            }
                        }
                    }
                }
                else if id.starts_with('{') && id.ends_with('}') {
                    // Handle object destructuring if needed
                }
                else {
                    // Regular variable
                    if let Some(JsExpr::Call { callee, .. }) = init {
                        if let JsExpr::Identifier(name, _) = &**callee {
                            if name == "createComputed" || name == "createSignal" {
                                signals.insert(id.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(ScriptMetadata { signals, props, emits })
    }
}
