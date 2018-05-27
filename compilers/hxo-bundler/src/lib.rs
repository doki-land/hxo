use hxo_ir::{IRModule, TemplateNodeIR};
use hxo_types::Result;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct FeatureSet {
    pub has_signals: bool,
    pub has_effects: bool,
    pub has_vdom: bool,
    pub has_ssr: bool,
    pub used_core_functions: HashSet<String>,
    pub used_dom_functions: HashSet<String>,
}

pub struct Bundler {
    pub feature_set: FeatureSet,
}

impl Bundler {
    pub fn new() -> Self {
        Self { feature_set: FeatureSet::default() }
    }
}

impl Default for Bundler {
    fn default() -> Self {
        Self::new()
    }
}

impl Bundler {
    pub fn analyze_all(&mut self, modules: &[IRModule]) {
        for module in modules {
            self.analyze_module(module);
        }
    }

    fn analyze_module(&mut self, module: &IRModule) {
        // 1. Analyze template for VDOM usage
        if let Some(template) = &module.template {
            if !template.nodes.is_empty() {
                self.feature_set.has_vdom = true;
                self.feature_set.used_dom_functions.insert("h".to_string());
            }
            for node in &template.nodes {
                self.analyze_node(node);
            }
        }

        // 2. Analyze script for reactivity
        if let Some(script) = &module.script {
            for _stmt in &script.body {
                // In a real implementation, we would analyze the AST more deeply
                // Here we just check for common reactivity function calls in stringified form or similar
                // For now, we'll assume any module with a script might use reactivity
                self.feature_set.has_signals = true;
                self.feature_set.has_effects = true;
            }
        }
    }

    fn analyze_node(&mut self, node: &TemplateNodeIR) {
        match node {
            TemplateNodeIR::Element(el) => {
                for child in &el.children {
                    self.analyze_node(child);
                }
            }
            TemplateNodeIR::Interpolation(_) => {
                self.feature_set.has_effects = true;
                self.feature_set.used_core_functions.insert("createEffect".to_string());
            }
            _ => {}
        }
    }

    pub fn generate_custom_runtime(&self) -> String {
        let mut runtime = String::new();
        runtime.push_str("// HXO Custom Runtime (Optimized)\n\n");

        if self.feature_set.has_signals || self.feature_set.has_effects {
            runtime.push_str("// Reactivity System\n");
            // In a real scenario, we'd read the actual JS files.
            // Here we provide a minimal implementation as a demonstration.
            runtime.push_str("export function createSignal(v) { /* ... */ }\n");
            runtime.push_str("export function createEffect(fn) { /* ... */ }\n");
        }

        if self.feature_set.has_vdom {
            runtime.push_str("\n// VDOM System\n");
            runtime.push_str("export function h(tag, props, children) { /* ... */ }\n");
        }

        runtime
    }

    pub fn bundle_all(&self, modules: &[IRModule]) -> Result<String> {
        let mut output = String::new();

        // 1. Generate the minimal runtime
        output.push_str(&self.generate_custom_runtime());
        output.push_str("\n\n// Components\n");

        // 2. Generate each component and link it to the custom runtime
        // In a real bundler, this would be more complex (module system, etc.)
        for module in modules {
            output.push_str(&format!("\n// Component: {}\n", module.name));
            // We would call the compiler here with a flag to use the internal runtime
        }

        Ok(output)
    }
}
