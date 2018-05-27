use hxo_ir::{IRModule, TemplateNodeIR};
use hxo_target_js::JsWriter;
use hxo_types::Result;
use std::collections::HashSet;

pub struct SsrBackend {
    pub runtime_path: String,
}

impl SsrBackend {
    pub fn new() -> Self {
        Self { runtime_path: "@hxo".to_string() }
    }
}

impl Default for SsrBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SsrBackend {
    pub fn generate(&self, ir: &IRModule) -> Result<String> {
        let mut writer = JsWriter::new();
        let mut used_core = HashSet::new();

        // 1. Generate SSR Function Body
        let mut body_writer = JsWriter::new();
        self.generate_ssr_body(ir, &mut body_writer, &mut used_core)?;

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

    fn generate_ssr_body(&self, ir: &IRModule, writer: &mut JsWriter, _used_core: &mut HashSet<String>) -> Result<()> {
        writer.write_block("export function render(ctx)", |writer| {
            writer.write("let html = '';");
            writer.newline();

            if let Some(template) = &ir.template {
                let mut node_index = 0;
                for node in &template.nodes {
                    Self::generate_node_ssr(node, writer, &mut node_index);
                }
            }

            writer.write_line("return html;");
        });
        Ok(())
    }

    fn generate_node_ssr(node: &TemplateNodeIR, writer: &mut JsWriter, node_index: &mut usize) {
        match node {
            TemplateNodeIR::Element(el) => {
                let current_index = *node_index;
                *node_index += 1;

                let mut start_tag = format!("html += '<{}", el.tag);

                // Add data-hxo-id for non-static elements or elements with dynamic content
                if !el.is_static {
                    start_tag.push_str(&format!(" data-hxo-id=\"{}\"", current_index));
                }

                for attr in &el.attributes {
                    if !attr.is_directive {
                        if attr.is_dynamic {
                            start_tag.push_str(&format!(
                                " {}=\"' + ({}) + '\"",
                                attr.name,
                                attr.value.as_deref().unwrap_or("")
                            ));
                        }
                        else {
                            match &attr.value {
                                Some(v) => start_tag.push_str(&format!(" {}=\"{}\"", attr.name, v)),
                                None => start_tag.push_str(&format!(" {}", attr.name)),
                            }
                        }
                    }
                }
                start_tag.push_str(">';");
                writer.write_line(&start_tag);

                for child in &el.children {
                    Self::generate_node_ssr(child, writer, node_index);
                }

                writer.write_line(&format!("html += '</{}>';", el.tag));
            }
            TemplateNodeIR::Text(text, _) => {
                *node_index += 1;
                writer.write_line(&format!("html += '{}';", text.replace("'", "\\'")));
            }
            TemplateNodeIR::Interpolation(expr) => {
                let current_index = *node_index;
                *node_index += 1;
                // Wrap interpolation in a span with ID for hydration
                writer
                    .write_line(&format!("html += '<span data-hxo-id=\"{}\">' + ({}) + '</span>';", current_index, expr.code));
            }
            TemplateNodeIR::Comment(comment, _) => {
                *node_index += 1;
                writer.write_line(&format!("html += '<!-- {} -->';", comment.replace("'", "\\'")));
            }
        }
    }
}
