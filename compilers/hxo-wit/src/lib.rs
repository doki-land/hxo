#[allow(dead_code)]
mod bindings {
    wit_bindgen::generate!({
        world: "hxo-world",
        path: "wit",
    });
}

pub use bindings::exports::hxo::component::hxo_handler::{Guest, RenderOptions};

use hxo_compiler::Compiler;

struct HxoHandler;

impl Guest for HxoHandler {
    fn render(template: String, _options: RenderOptions) -> Result<String, String> {
        let mut compiler = Compiler::new();

        match compiler.compile("anonymous.hxo", &template) {
            Ok(result) => {
                // Here we would normally execute the compiled JS or return it
                // For now, let's just return the compiled code as a string
                Ok(result.code)
            }
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}

bindings::export!(HxoHandler with_types_in bindings);
