use hxo_ir::IRModule;
use hxo_types::{CodeWriter, Result};

pub struct DtsWriter {
    inner: CodeWriter,
}

impl DtsWriter {
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

    pub fn write_interface<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.inner.write(&format!("export interface {} {{", name));
        self.inner.newline();
        self.inner.indent();
        f(self);
        self.inner.dedent();
        self.inner.write_line("}");
    }

    pub fn write_type_alias(&mut self, name: &str, value: &str) {
        self.inner.write_line(&format!("export type {} = {};", name, value));
    }

    pub fn write_import(&mut self, names: &[&str], source: &str) {
        self.inner.write_line(&format!("import {{ {} }} from '{}';", names.join(", "), source));
    }

    pub fn write_declare_const(&mut self, name: &str, type_name: &str) {
        self.inner.write_line(&format!("declare const {}: {};", name, type_name));
    }

    pub fn write_export_default(&mut self, name: &str) {
        self.inner.write_line(&format!("export default {};", name));
    }

    pub fn finish(self) -> String {
        self.inner.finish().0
    }
}

pub struct DtsBackend;

impl DtsBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, ir: &IRModule) -> Result<String> {
        let mut writer = DtsWriter::new();

        // 1. Extract metadata from script_meta
        let mut props = Vec::new();
        let mut emits = Vec::new();
        let mut signals = Vec::new();

        if let Some(meta) = &ir.script_meta {
            if let Some(props_val) = meta.get("props").and_then(|v| v.as_array()) {
                for p in props_val {
                    if let Some(s) = p.as_str() {
                        props.push(s.to_string());
                    }
                }
            }
            if let Some(emits_val) = meta.get("emits").and_then(|v| v.as_array()) {
                for e in emits_val {
                    if let Some(s) = e.as_str() {
                        emits.push(s.to_string());
                    }
                }
            }
            if let Some(signals_val) = meta.get("signals").and_then(|v| v.as_array()) {
                for s in signals_val {
                    if let Some(s_str) = s.as_str() {
                        signals.push(s_str.to_string());
                    }
                }
            }
        }

        // 2. Generate Types
        writer.write_import(&["VNode"], "@hxo/core");
        writer.newline();

        // Props interface
        writer.write_interface("Props", |writer| {
            if props.is_empty() {
                writer.write_line("[key: string]: any;");
            }
            else {
                for prop in &props {
                    writer.write_line(&format!("{}: any;", prop));
                }
            }
        });
        writer.newline();

        // Emits type (simplified)
        if !emits.is_empty() {
            let emits_type = emits.iter().map(|e| format!("'{}'", e)).collect::<Vec<_>>().join(" | ");
            writer.write_type_alias("Emits", &emits_type);
            writer.newline();
        }

        // Component Instance (the 'this' or 'ctx' in render)
        writer.write_interface("ComponentInstance", |writer| {
            for signal in &signals {
                writer.write_line(&format!("{}: any;", signal));
            }
            // Add props to instance as well
            for prop in &props {
                writer.write_line(&format!("{}: any;", prop));
            }
            writer.write_line("$props: Props;");
            if !emits.is_empty() {
                writer.write_line("$emit: (event: Emits, ...args: any[]) => void;");
            }
        });
        writer.newline();

        // Component definition
        writer.write_line("declare const component: {");
        writer.indent();
        writer.write_line(&format!("name: '{}';", ir.name));
        writer.write_line("setup(props: Props): ComponentInstance;");
        writer.write_line("render(ctx: ComponentInstance): VNode;");
        writer.dedent();
        writer.write_line("};");
        writer.newline();

        writer.write_export_default("component");

        Ok(writer.finish())
    }
}
