use hxo_ir::{IRModule, TemplateNodeIR};
use hxo_source_map::{SourceMap, SourceMapBuilder};
use hxo_types::{CodeWriter, Position, Result, Span};

pub struct MapWriter {
    inner: CodeWriter,
    builder: SourceMapBuilder,
    current_source: Option<String>,
}

impl MapWriter {
    pub fn new() -> Self {
        Self { inner: CodeWriter::new(), builder: SourceMapBuilder::new(), current_source: None }
    }

    pub fn set_source(&mut self, source: String) {
        self.current_source = Some(source);
    }

    pub fn add_mapping(&mut self, original: Option<Position>, name: Option<String>) {
        if let Some(original) = original {
            let generated = self.inner.position();
            self.builder.add_mapping(generated, original, self.current_source.clone(), name);
        }
    }

    pub fn add_span_mapping(&mut self, span: Span, name: Option<String>) {
        if !span.is_unknown() {
            self.add_mapping(Some(span.start), name);
        }
    }

    pub fn write(&mut self, text: &str) {
        self.inner.write(text);
    }

    pub fn write_line(&mut self, text: &str) {
        self.inner.write_line(text);
    }

    pub fn newline(&mut self) {
        self.inner.newline();
    }

    pub fn indent(&mut self) {
        self.inner.indent();
    }

    pub fn dedent(&mut self) {
        self.inner.dedent();
    }

    pub fn finish(self) -> SourceMap {
        self.builder.finish()
    }
}

pub struct MapBackend;

impl MapBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, ir: &IRModule) -> Result<SourceMap> {
        let mut writer = MapWriter::new();
        writer.set_source(format!("{}.hxo", ir.name));

        // Simulate JS generation to create mappings
        // This should mirror JsBackend's structure

        // 1. Imports
        writer.add_mapping(None, None); // Start of file
        writer.write_line("import { createSignal, createEffect, Fragment } from '@hxo/core';");
        writer.write_line("import { h, createTextVNode } from '@hxo/core/dom';");
        writer.newline();

        // 2. Component
        writer.write("export default {");
        writer.newline();
        writer.indent();

        writer.write(&format!("name: '{}',", ir.name));
        writer.newline();

        // 3. Setup
        writer.write("setup(props) {");
        writer.newline();
        writer.indent();

        if let Some(_script) = &ir.script {
            // In a real implementation, we'd iterate over script statements
            // and add mappings for each. For now, we add a general mapping.
            writer.add_mapping(None, Some("setup".to_string()));
            writer.write_line("// script content...");
        }

        writer.write_line("return {};");
        writer.dedent();
        writer.write_line("},");

        // 4. Render
        writer.write("render(ctx) {");
        writer.newline();
        writer.indent();
        writer.write("return ");

        if let Some(template) = &ir.template {
            self.generate_template(&template.nodes, &mut writer, ir);
        }
        else {
            writer.write("null");
        }

        writer.write_line(";");
        writer.dedent();
        writer.write_line("}");

        writer.dedent();
        writer.write_line("};");

        Ok(writer.finish())
    }

    fn generate_template(&self, nodes: &[TemplateNodeIR], writer: &mut MapWriter, ir: &IRModule) {
        if nodes.is_empty() {
            writer.write("null");
        }
        else if nodes.len() == 1 {
            self.generate_node(&nodes[0], writer, ir);
        }
        else {
            writer.write("h(Fragment, null, [");
            writer.newline();
            writer.indent();
            for node in nodes {
                self.generate_node(node, writer, ir);
                writer.write_line(",");
            }
            writer.dedent();
            writer.write("])");
        }
    }

    fn generate_node(&self, node: &TemplateNodeIR, writer: &mut MapWriter, ir: &IRModule) {
        match node {
            TemplateNodeIR::Element(el) => {
                writer.add_span_mapping(el.span, Some(el.tag.clone()));
                writer.write(&format!("h('{}', {{", el.tag));

                // Attributes
                for (i, attr) in el.attributes.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    writer.add_span_mapping(attr.span, Some(attr.name.clone()));
                    writer.write(&format!("'{}': ", attr.name));
                    if let Some(val) = &attr.value {
                        writer.write(&format!("'{}'", val));
                    }
                    else {
                        writer.write("true");
                    }
                }

                writer.write("}, [");
                if !el.children.is_empty() {
                    writer.newline();
                    writer.indent();
                    for child in &el.children {
                        self.generate_node(child, writer, ir);
                        writer.write_line(",");
                    }
                    writer.dedent();
                }
                writer.write("])");
            }
            TemplateNodeIR::Text(text, span) => {
                writer.add_span_mapping(*span, None);
                writer.write(&format!("createTextVNode('{}')", text.trim()));
            }
            TemplateNodeIR::Interpolation(expr) => {
                writer.add_span_mapping(expr.span, None);
                writer.write(&format!("createTextVNode(ctx.{})", expr.code));
            }
            TemplateNodeIR::Comment(_, span) => {
                writer.add_span_mapping(*span, None);
            }
        }
    }
}
