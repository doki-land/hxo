use hxo_parser_rust::RustWasmCompiler;

#[test]
fn test_compile_rust() {
    let compiler = RustWasmCompiler::new();
    let source = "fn main() {}";
    let wasm = compiler.compile(source).unwrap();
    assert_eq!(wasm, source.as_bytes().to_vec());
}
