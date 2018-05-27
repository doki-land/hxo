use hxo_optimizer::Optimizer;
use hxo_parser::{Parser, ParserRegistry};
use hxo_source_map::SourceMap;
use hxo_types::Result;
use std::sync::Arc;

pub mod codegen;

use crate::codegen::JsBackend;
use hxo_hydrate::HydrateBackend;
use hxo_ssr::SsrBackend;

pub struct CompileResult {
    pub code: String,
    pub css: String,
    pub source_map: Option<SourceMap>,
}

pub struct Compiler {
    pub registry: Arc<ParserRegistry>,
    pub last_css: String,
}

#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    pub ssr: bool,
    pub hydrate: bool,
    pub minify: bool,
    pub is_prod: bool,
    pub target: Option<String>,
    pub scope_id: Option<String>,
    pub i18n_locale: Option<String>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        let mut registry = ParserRegistry::new();

        // Register default parsers
        registry.register_template_parser("hxo", Arc::new(hxo_parser_template::TemplateParser));
        registry.register_template_parser("html", Arc::new(hxo_parser_template::TemplateParser));
        registry.register_template_parser("pug", Arc::new(hxo_parser_pug::PugParser));
        registry.register_template_parser("markdown", Arc::new(hxo_parser_markdown::MarkdownParser));
        registry.register_template_parser("md", Arc::new(hxo_parser_markdown::MarkdownParser));

        let ts_parser = Arc::new(hxo_parser_expression::ExprParser);
        registry.register_script_parser("ts", ts_parser.clone());
        registry.register_script_parser("typescript", ts_parser.clone());
        registry.register_script_parser("js", ts_parser.clone());
        registry.register_script_parser("javascript", ts_parser);

        let yaml_parser = Arc::new(hxo_parser_yaml::YamlParser);
        registry.register_metadata_parser("yaml", yaml_parser.clone());
        registry.register_metadata_parser("yml", yaml_parser);

        let json_parser = Arc::new(hxo_parser_json::JsonParser);
        registry.register_metadata_parser("json", json_parser);

        let toml_parser = Arc::new(hxo_parser_toml::TomlParser);
        registry.register_metadata_parser("toml", toml_parser);

        let props_parser = Arc::new(hxo_parser_properties::PropertiesParser);
        registry.register_metadata_parser("properties", props_parser);

        let fluent_parser = Arc::new(hxo_parser_fluent::FluentParser);
        registry.register_metadata_parser("fluent", fluent_parser.clone());
        registry.register_metadata_parser("ftl", fluent_parser);

        let css_parser = Arc::new(hxo_parser_css::CssParser);
        registry.register_style_parser("css", css_parser);

        let scss_parser = Arc::new(hxo_parser_scss::ScssParser);
        registry.register_style_parser("scss", scss_parser);

        let sass_parser = Arc::new(hxo_parser_sass::SassParser);
        registry.register_style_parser("sass", sass_parser);

        let less_parser = Arc::new(hxo_parser_less::LessParser);
        registry.register_style_parser("less", less_parser);

        let stylus_parser = Arc::new(hxo_parser_stylus::StylusParser);
        registry.register_style_parser("stylus", stylus_parser);

        let tailwind_parser = Arc::new(hxo_parser_tailwind::TailwindParser);
        registry.register_style_parser("tailwind", tailwind_parser);

        Self { registry: Arc::new(registry), last_css: String::new() }
    }

    pub fn compile(&mut self, name: &str, source: &str) -> Result<CompileResult> {
        self.compile_with_options(name, source, CompileOptions::default())
    }

    pub fn compile_with_options(&mut self, name: &str, source: &str, mut options: CompileOptions) -> Result<CompileResult> {
        // 1. Parse source to IR (Now includes Script Analysis inside)
        let mut parser = Parser::new(name.to_string(), source, self.registry.clone());
        let mut ir = parser.parse_all()?;

        // 2. Optimize & Transform IR
        let mut optimizer = Optimizer::new();

        // Handle scope ID in optimizer/transformer
        let has_scoped_style = ir.styles.iter().any(|s| s.scoped);
        if has_scoped_style && options.scope_id.is_none() {
            options.scope_id = Some(optimizer.generate_scope_id(name));
        }

        optimizer.optimize(&mut ir, options.i18n_locale.as_deref(), options.is_prod);

        if let Some(scope_id) = &options.scope_id {
            optimizer.apply_scope_id(&mut ir, scope_id);
        }

        // 3. Process styles (Tailwind/Utility CSS)
        optimizer.process_styles(&ir)?;

        // 4. Generate code based on options
        let (code, source_map) = if options.ssr {
            let ssr_backend = SsrBackend::new();
            (ssr_backend.generate(&ir)?, None)
        }
        else if options.hydrate {
            let hydrate_backend = HydrateBackend::new();
            (hydrate_backend.generate(&ir)?, None)
        }
        else {
            let js_backend = JsBackend::new(options.minify, options.is_prod, options.target.clone());
            let (code, sm) = js_backend.generate(&ir)?;
            (code, Some(sm))
        };

        // 5. Get all generated CSS (includes both utility classes and <style> block contents)
        self.last_css = optimizer.get_css();

        Ok(CompileResult { code, css: self.last_css.clone(), source_map })
    }

    pub fn get_css(&self) -> String {
        self.last_css.clone()
    }
}
