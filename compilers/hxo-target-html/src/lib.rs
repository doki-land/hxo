use hxo_ir::{AttributeIR, ElementIR, IRModule, TemplateNodeIR};
use hxo_types::{CodeWriter, Result};

pub struct HtmlWriter {
    inner: CodeWriter,
}

impl HtmlWriter {
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

    /// 写入带有属性的标签开始: <tag attr1="val1">
    pub fn write_tag_start(&mut self, tag: &str, attributes: &[(&str, Option<&str>)]) {
        self.inner.write("<");
        self.inner.write(tag);
        for (name, value) in attributes {
            self.inner.write(" ");
            self.inner.write(name);
            if let Some(val) = value {
                self.inner.write("=\"");
                self.inner.write(val);
                self.inner.write("\"");
            }
        }
        self.inner.write(">");
    }

    /// 写入自闭合标签: <tag attr1="val1" />
    pub fn write_self_closing_tag(&mut self, tag: &str, attributes: &[(&str, Option<&str>)]) {
        self.inner.write("<");
        self.inner.write(tag);
        for (name, value) in attributes {
            self.inner.write(" ");
            self.inner.write(name);
            if let Some(val) = value {
                self.inner.write("=\"");
                self.inner.write(val);
                self.inner.write("\"");
            }
        }
        self.inner.write(" />");
    }

    /// 写入结束标签: </tag>
    pub fn write_tag_end(&mut self, tag: &str) {
        self.inner.write("</");
        self.inner.write(tag);
        self.inner.write(">");
    }

    /// 写入注释: <!-- content -->
    pub fn write_comment(&mut self, content: &str) {
        self.inner.write("<!-- ");
        self.inner.write(content);
        self.inner.write(" -->");
    }

    pub fn write_node(&mut self, node: &TemplateNodeIR) {
        match node {
            TemplateNodeIR::Element(el) => self.write_element(el),
            TemplateNodeIR::Text(text, _) => self.write(text),
            TemplateNodeIR::Interpolation(expr) => {
                self.write("{{ ");
                self.write(&expr.code);
                self.write(" }}");
            }
            TemplateNodeIR::Comment(comment, _) => {
                self.write("<!-- ");
                self.write(comment);
                self.write(" -->");
            }
        }
    }

    pub fn write_element(&mut self, el: &ElementIR) {
        if el.children.is_empty() && hxo_types::is_void_element(&el.tag) {
            self.inner.write("<");
            self.inner.write(&el.tag);
            for attr in &el.attributes {
                self.write_attribute(attr);
            }
            self.inner.write(" />");
        }
        else {
            self.inner.write("<");
            self.inner.write(&el.tag);
            for attr in &el.attributes {
                self.write_attribute(attr);
            }
            self.inner.write(">");

            if el.children.len() == 1 {
                let child = &el.children[0];
                match child {
                    TemplateNodeIR::Text(text, _) => self.write(text),
                    TemplateNodeIR::Interpolation(expr) => {
                        self.write("{{ ");
                        self.write(&expr.code);
                        self.write(" }}");
                    }
                    _ => {
                        self.indent();
                        self.newline();
                        self.write_node(child);
                        self.dedent();
                        self.newline();
                    }
                }
            }
            else if !el.children.is_empty() {
                self.indent();
                self.newline();
                for child in &el.children {
                    self.write_node(child);
                    self.newline();
                }
                self.dedent();
            }
            self.write_tag_end(&el.tag);
        }
    }

    fn write_attribute(&mut self, attr: &AttributeIR) {
        self.write(" ");
        if attr.is_directive {
            self.write("v-");
        }
        if attr.is_dynamic {
            self.write(":");
        }
        self.write(&attr.name);
        if let Some(value) = &attr.value {
            self.write("=\"");
            self.write(value);
            self.write("\"");
        }
    }

    pub fn finish(self) -> String {
        self.inner.finish().0
    }
}

pub struct HtmlBackend;

impl HtmlBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, ir: &IRModule) -> Result<String> {
        let mut writer = HtmlWriter::new();

        if let Some(template) = &ir.template {
            for node in &template.nodes {
                writer.write_node(node);
                writer.newline();
            }
        }

        Ok(writer.finish())
    }
}
