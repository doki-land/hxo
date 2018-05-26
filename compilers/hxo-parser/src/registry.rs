use crate::ParseState;
use hxo_ir::{JsProgram, TemplateNodeIR};
use hxo_types::{HxoValue, Result};
use std::{collections::HashMap, sync::Arc};

pub trait TemplateSubParser: Send + Sync {
    fn parse(&self, state: &mut ParseState) -> Result<Vec<TemplateNodeIR>>;
}

pub trait ScriptSubParser: Send + Sync {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<JsProgram>;
}

pub trait StyleSubParser: Send + Sync {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<String>;
}

pub trait MetadataSubParser: Send + Sync {
    fn parse(&self, state: &mut ParseState, lang: &str) -> Result<HxoValue>;
}

#[derive(Default)]
pub struct ParserRegistry {
    template_parser: Option<Arc<dyn TemplateSubParser>>,
    script_parsers: HashMap<String, Arc<dyn ScriptSubParser>>,
    style_parsers: HashMap<String, Arc<dyn StyleSubParser>>,
    metadata_parsers: HashMap<String, Arc<dyn MetadataSubParser>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_template_parser(&mut self, parser: Arc<dyn TemplateSubParser>) {
        self.template_parser = Some(parser);
    }

    pub fn register_script_parser(&mut self, lang: &str, parser: Arc<dyn ScriptSubParser>) {
        self.script_parsers.insert(lang.to_string(), parser);
    }

    pub fn register_style_parser(&mut self, lang: &str, parser: Arc<dyn StyleSubParser>) {
        self.style_parsers.insert(lang.to_string(), parser);
    }

    pub fn register_metadata_parser(&mut self, lang: &str, parser: Arc<dyn MetadataSubParser>) {
        self.metadata_parsers.insert(lang.to_string(), parser);
    }

    pub fn get_template_parser(&self) -> Option<Arc<dyn TemplateSubParser>> {
        self.template_parser.clone()
    }

    pub fn get_script_parser(&self, lang: &str) -> Option<Arc<dyn ScriptSubParser>> {
        self.script_parsers.get(lang).cloned()
    }

    pub fn get_style_parser(&self, lang: &str) -> Option<Arc<dyn StyleSubParser>> {
        self.style_parsers.get(lang).cloned()
    }

    pub fn get_metadata_parser(&self, lang: &str) -> Option<Arc<dyn MetadataSubParser>> {
        self.metadata_parsers.get(lang).cloned()
    }
}
