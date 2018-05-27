use hxo_ir::{IRModule, TemplateNodeIR};
use hxo_target_js::JsWriter;
use hxo_types::Result;
use std::collections::HashSet;

pub struct HydrateBackend {
    pub runtime_path: String,
}

impl HydrateBackend {
    pub fn new() -> Self {
        Self { runtime_path: "@hxo".to_string() }
    }
}

impl Default for HydrateBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl HydrateBackend {
    pub fn generate(&self, ir: &IRModule) -> Result<String> {
        let mut writer = JsWriter::new();
        let mut used_core = HashSet::new();

        // 1. Generate Hydrate Function Body
        let mut body_writer = JsWriter::new();
        Self::generate_hydrate_body(ir, &mut body_writer, &mut used_core)?;

        // 2. Generate Imports
        if !used_core.is_empty() {
            let mut imports: Vec<_> = used_core.into_iter().collect();
            imports.sort();
            writer.write_line(&format!("import {{ {} }} from '{}/core';", imports.join(", "), self.runtime_path));
            writer.newline();
        }

        // 3. Append Body
        writer.append(body_writer);

        Ok(writer.finish().0)
    }

    fn generate_hydrate_body(ir: &IRModule, writer: &mut JsWriter, used_core: &mut HashSet<String>) -> Result<()> {
        writer.write_block("export function hydrate(root, ctx)", |writer| {
            if let Some(template) = &ir.template {
                let mut node_index = 0;
                for node in &template.nodes {
                    Self::generate_node_hydrate(node, writer, &mut node_index, used_core);
                }
            }
        });
        Ok(())
    }

    fn generate_node_hydrate(
        node: &TemplateNodeIR,
        writer: &mut JsWriter,
        node_index: &mut usize,
        used_core: &mut HashSet<String>,
    ) {
        match node {
            TemplateNodeIR::Element(el) => {
                let current_index = *node_index;
                *node_index += 1;

                // Only generate hydration code if the element is dynamic
                if !el.is_static {
                    let el_var = format!("el{}", current_index);
                    writer
                        .write_line(&format!("const {} = root.querySelector('[data-hxo-id=\"{}\"]');", el_var, current_index));

                    // Handle event listeners (directives starting with @)
                    for attr in &el.attributes {
                        if attr.is_directive && attr.name.starts_with('@') {
                            let event_name = &attr.name[1..];
                            if let Some(value) = &attr.value {
                                writer.write_line(&format!("{}.addEventListener('{}', () => {});", el_var, event_name, value));
                            }
                        }
                    }

                    // Handle dynamic children
                    for child in &el.children {
                        Self::generate_node_hydrate(child, writer, node_index, used_core);
                    }
                }
            }
            TemplateNodeIR::Interpolation(expr) => {
                let current_index = *node_index;
                *node_index += 1;

                used_core.insert("createEffect".to_string());
                writer.write_line(&format!(
                    "const text{} = root.querySelector('[data-hxo-id=\"{}\"]');",
                    current_index, current_index
                ));
                writer.write_block("createEffect(() =>", |writer| {
                    writer.write_line(&format!("text{}.textContent = {};", current_index, expr.code));
                });
                writer.write_line(");");
            }
            _ => {
                // Static text and comments don't need hydration
                *node_index += 1;
            }
        }
    }
}
