use hxo_ir::IRModule;
use hxo_types::Result;

pub use hxo_target_css::CssBackend;
pub use hxo_target_dts::DtsBackend;
pub use hxo_target_html::HtmlBackend;
pub use hxo_target_js::JsBackend;
pub use hxo_target_map::MapBackend;
pub use hxo_target_wasm::WasmBackend;

pub trait Backend {
    type Output;
    fn generate(&self, ir: &IRModule) -> Result<Self::Output>;
}

// Implement Backend trait for the target backends
impl Backend for JsBackend {
    type Output = (String, hxo_source_map::SourceMap);
    fn generate(&self, ir: &IRModule) -> Result<Self::Output> {
        self.generate(ir)
    }
}

impl Backend for WasmBackend {
    type Output = Vec<u8>;
    fn generate(&self, ir: &IRModule) -> Result<Self::Output> {
        self.generate(ir)
    }
}

impl Backend for HtmlBackend {
    type Output = String;
    fn generate(&self, ir: &IRModule) -> Result<Self::Output> {
        self.generate(ir)
    }
}

impl Backend for CssBackend {
    type Output = String;
    fn generate(&self, ir: &IRModule) -> Result<Self::Output> {
        self.generate(ir)
    }
}

impl Backend for DtsBackend {
    type Output = String;
    fn generate(&self, ir: &IRModule) -> Result<Self::Output> {
        self.generate(ir)
    }
}

impl Backend for MapBackend {
    type Output = hxo_source_map::SourceMap;
    fn generate(&self, ir: &IRModule) -> Result<Self::Output> {
        self.generate(ir)
    }
}
