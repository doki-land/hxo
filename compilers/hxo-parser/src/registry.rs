use crate::ParseState;
use hxo_ir::{JsProgram, TemplateNodeIR};
use hxo_types::{HxoValue, Result};
use std::{collections::HashMap, sync::Arc};

pub trait TemplateParser: Send + Sync {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<Vec<TemplateNodeIR>>;
}

pub trait ScriptParser: Send + Sync {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<JsProgram>;
}

pub trait StyleParser: Send + Sync {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<String>;
}

pub trait MetadataParser: Send + Sync {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<HxoValue>;
}

#[derive(Default)]
pub struct ParserRegistry {
    template_parsers: HashMap<String, Arc<dyn TemplateParser>>,
    script_parsers: HashMap<String, Arc<dyn ScriptParser>>,
    style_parsers: HashMap<String, Arc<dyn StyleParser>>,
    metadata_parsers: HashMap<String, Arc<dyn MetadataParser>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_template_parser(&mut self, lang: &str, parser: Arc<dyn TemplateParser>) {
        self.template_parsers.insert(lang.to_string(), parser);
    }

    pub fn register_script_parser(&mut self, lang: &str, parser: Arc<dyn ScriptParser>) {
        self.script_parsers.insert(lang.to_string(), parser);
    }

    pub fn register_style_parser(&mut self, lang: &str, parser: Arc<dyn StyleParser>) {
        self.style_parsers.insert(lang.to_string(), parser);
    }

    pub fn register_metadata_parser(&mut self, lang: &str, parser: Arc<dyn MetadataParser>) {
        self.metadata_parsers.insert(lang.to_string(), parser);
    }

    pub fn get_template_parser(&self, lang: &str) -> Option<Arc<dyn TemplateParser>> {
        self.template_parsers.get(lang).cloned()
    }

    pub fn get_script_parser(&self, lang: &str) -> Option<Arc<dyn ScriptParser>> {
        self.script_parsers.get(lang).cloned()
    }

    pub fn get_style_parser(&self, lang: &str) -> Option<Arc<dyn StyleParser>> {
        self.style_parsers.get(lang).cloned()
    }

    pub fn get_metadata_parser(&self, lang: &str) -> Option<Arc<dyn MetadataParser>> {
        self.metadata_parsers.get(lang).cloned()
    }
}
