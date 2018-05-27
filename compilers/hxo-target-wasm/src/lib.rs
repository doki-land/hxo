use hxo_ir::{IRModule, TemplateNodeIR};
use hxo_types::{CodeWriter, Result};
use std::collections::HashMap;

#[derive(Default)]
pub struct WasmWriter {
    inner: CodeWriter,
}

impl WasmWriter {
    pub fn new() -> Self {
        Self { inner: CodeWriter::new() }
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

    pub fn finish(self) -> Vec<u8> {
        self.inner.finish().0.into_bytes()
    }

    pub fn finish_wat(self) -> String {
        self.inner.finish().0
    }
}

pub struct WasmBackend {
    pub debug: bool,
}

impl WasmBackend {
    pub fn new(debug: bool) -> Self {
        Self { debug }
    }

    pub fn generate(&self, ir: &IRModule) -> Result<Vec<u8>> {
        let wat = self.generate_wat(ir)?;
        Ok(wat.into_bytes())
    }

    pub fn generate_wat(&self, ir: &IRModule) -> Result<String> {
        let mut writer = WasmWriter::new();
        let mut strings = HashMap::new();
        let mut string_offset = 0;

        let mut collect_strings = |s: &str| {
            if !strings.contains_key(s) {
                strings.insert(s.to_string(), string_offset);
                string_offset += s.len();
            }
        };

        // Collect strings from template
        if let Some(template) = &ir.template {
            for node in &template.nodes {
                Self::collect_node_strings(node, &mut collect_strings);
            }
        }

        writer.write_line("(module");
        writer.indent();
        writer.write_line(&format!(";; Component: {}", ir.name));

        // Imports
        writer.write_line("(import \"hxo\" \"create_element\" (func $create_element (param i32 i32) (result i32)))");
        writer.write_line("(import \"hxo\" \"create_text\" (func $create_text (param i32 i32) (result i32)))");
        writer.write_line("(import \"hxo\" \"set_attribute\" (func $set_attribute (param i32 i32 i32 i32 i32)))");
        writer.write_line("(import \"hxo\" \"append_child\" (func $append_child (param i32 i32)))");

        // Memory
        let pages = (string_offset / 65536) + 1;
        writer.write_line(&format!("(memory (export \"memory\") {})", pages));

        // Data section
        for (s, offset) in &strings {
            writer.write_line(&format!("(data (i32.const {}) \"{}\")", offset, s));
        }

        // Render function
        writer.write_line("(func (export \"render\") (result i32)");
        writer.indent();

        // Declare locals for different depths
        let max_depth = Self::calculate_max_depth(ir.template.as_ref());
        for i in 0..=max_depth {
            writer.write_line(&format!("(local $el_{} i32)", i));
        }

        if let Some(template) = &ir.template {
            if template.nodes.is_empty() {
                writer.write_line("i32.const 0");
            }
            else if template.nodes.len() == 1 {
                Self::generate_node_wat(&template.nodes[0], &mut writer, &strings, 0);
            }
            else {
                writer.write_line(";; Multiple root nodes not fully supported in WASM yet");
                Self::generate_node_wat(&template.nodes[0], &mut writer, &strings, 0);
            }
        }
        else {
            writer.write_line("i32.const 0");
        }

        writer.dedent();
        writer.write_line(")");

        writer.dedent();
        writer.write_line(")");

        Ok(writer.finish_wat())
    }

    fn calculate_max_depth(template: Option<&hxo_ir::TemplateIR>) -> usize {
        match template {
            Some(t) => t.nodes.iter().map(Self::node_depth).max().unwrap_or(0),
            None => 0,
        }
    }

    fn node_depth(node: &TemplateNodeIR) -> usize {
        match node {
            TemplateNodeIR::Element(el) => 1 + el.children.iter().map(Self::node_depth).max().unwrap_or(0),
            _ => 1,
        }
    }

    fn collect_node_strings(node: &TemplateNodeIR, collect: &mut impl FnMut(&str)) {
        match node {
            TemplateNodeIR::Element(el) => {
                collect(&el.tag);
                for attr in &el.attributes {
                    collect(&attr.name);
                    if let Some(val) = &attr.value {
                        collect(val);
                    }
                }
                for child in &el.children {
                    Self::collect_node_strings(child, collect);
                }
            }
            TemplateNodeIR::Text(text, _) => {
                collect(text);
            }
            TemplateNodeIR::Interpolation(_) => {
                // Interpolations are dynamic, but their static parts could be collected
            }
            TemplateNodeIR::Comment(text, _) => {
                collect(text);
            }
        }
    }

    fn generate_node_wat(node: &TemplateNodeIR, writer: &mut WasmWriter, strings: &HashMap<String, usize>, depth: usize) {
        match node {
            TemplateNodeIR::Element(el) => {
                let tag_offset = strings.get(&el.tag).unwrap();
                let tag_len = el.tag.len();

                writer.write_line(";; Create element");
                writer.write_line(&format!("i32.const {}", tag_offset));
                writer.write_line(&format!("i32.const {}", tag_len));
                writer.write_line("call $create_element");
                writer.write_line(&format!("local.set $el_{}", depth));

                for attr in &el.attributes {
                    let name_offset = strings.get(&attr.name).unwrap();
                    let name_len = attr.name.len();
                    if let Some(val) = &attr.value {
                        let val_offset = strings.get(val).unwrap();
                        let val_len = val.len();

                        writer.write_line(";; Set attribute");
                        writer.write_line(&format!("local.get $el_{}", depth));
                        writer.write_line(&format!("i32.const {}", name_offset));
                        writer.write_line(&format!("i32.const {}", name_len));
                        writer.write_line(&format!("i32.const {}", val_offset));
                        writer.write_line(&format!("i32.const {}", val_len));
                        writer.write_line("call $set_attribute");
                    }
                }

                for child in &el.children {
                    writer.write_line(&format!("local.get $el_{}", depth));
                    Self::generate_node_wat(child, writer, strings, depth + 1);
                    writer.write_line("call $append_child");
                }

                writer.write_line(&format!("local.get $el_{}", depth));
            }
            TemplateNodeIR::Text(text, _) => {
                let offset = strings.get(text).unwrap();
                let len = text.len();
                writer.write_line(&format!("i32.const {}", offset));
                writer.write_line(&format!("i32.const {}", len));
                writer.write_line("call $create_text");
            }
            TemplateNodeIR::Interpolation(expr) => {
                writer.write_line(&format!(";; Interpolation: {}", expr.code));
                writer.write_line("i32.const 0 ;; Placeholder for dynamic text");
            }
            TemplateNodeIR::Comment(text, _) => {
                writer.write_line(&format!(";; Comment: {}", text));
                writer.write_line("i32.const 0 ;; Comments ignored in WASM render for now");
            }
        }
    }
}
