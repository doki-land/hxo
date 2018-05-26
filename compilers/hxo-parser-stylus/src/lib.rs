use hxo_types::{Error, Result};

pub fn compile(_source: &str) -> Result<String> {
    // Placeholder for Stylus compilation.
    Err(Error::not_implemented("Stylus parser".to_string(), hxo_types::Span::unknown()))
}
