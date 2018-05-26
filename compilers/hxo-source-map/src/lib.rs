use hxo_types::{Position, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapping {
    pub generated: Position,
    pub original: Position,
    pub source_file: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceMap {
    pub mappings: Vec<SourceMapping>,
    pub sources: Vec<String>,
    pub names: Vec<String>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| {
            hxo_types::Error::external_error("serde_json".to_string(), e.to_string(), hxo_types::Span::default())
        })
    }
}

pub struct SourceMapBuilder {
    mappings: Vec<SourceMapping>,
    sources: Vec<String>,
    names: Vec<String>,
}

impl SourceMapBuilder {
    pub fn new() -> Self {
        Self { mappings: Vec::new(), sources: Vec::new(), names: Vec::new() }
    }

    pub fn add_mapping(&mut self, generated: Position, original: Position, source_file: Option<String>, name: Option<String>) {
        if let Some(ref source) = source_file {
            if !self.sources.contains(source) {
                self.sources.push(source.clone());
            }
        }
        if let Some(ref name_str) = name {
            if !self.names.contains(name_str) {
                self.names.push(name_str.clone());
            }
        }
        self.mappings.push(SourceMapping { generated, original, source_file, name });
    }

    pub fn add_from_writer(&mut self, mappings: &[(Position, hxo_types::Span)], source_file: Option<String>) {
        for (generated, span) in mappings {
            self.add_mapping(*generated, span.start, source_file.clone(), None);
        }
    }

    pub fn finish(self) -> SourceMap {
        SourceMap { mappings: self.mappings, sources: self.sources, names: self.names }
    }
}
