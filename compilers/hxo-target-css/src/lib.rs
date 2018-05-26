use hxo_ir::{IRModule, StyleIR};
use hxo_types::{CodeWriter, Result};

pub struct CssWriter {
    inner: CodeWriter,
}

impl CssWriter {
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

    pub fn finish(self) -> String {
        self.inner.finish().0
    }
}

pub struct CssCompiler {}

impl CssCompiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn minify(&self, css: &str) -> String {
        css.trim()
            .split('\n')
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .join("")
            .replace(" {", "{")
            .replace("{ ", "{")
            .replace(" }", "}")
            .replace("} ", "}")
            .replace(": ", ":")
            .replace("; ", ";")
    }
}

pub struct CssBackend {
    pub minify: bool,
}

impl CssBackend {
    pub fn new(minify: bool) -> Self {
        Self { minify }
    }

    pub fn minify(&self, css: &str) -> String {
        // Basic minification for now, can be improved with lightningcss
        css.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    pub fn generate(&self, ir: &IRModule) -> Result<String> {
        let mut writer = CssWriter::new();

        for style in &ir.styles {
            self.generate_style(style, &mut writer)?;
            writer.newline();
        }

        Ok(writer.finish())
    }

    fn generate_style(&self, style: &StyleIR, writer: &mut CssWriter) -> Result<()> {
        if !self.minify {
            if style.scoped {
                writer.write_line(&format!("/* Scoped CSS (lang: {}) */", style.lang));
            }
            else {
                writer.write_line(&format!("/* CSS (lang: {}) */", style.lang));
            }
        }

        if self.minify {
            writer.write(style.code.trim());
        }
        else {
            writer.write_line(&style.code);
        }
        Ok(())
    }
}
