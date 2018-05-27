use hxo_parser_rust::RustWasmCompiler;

#[test]
fn test_compile_rust() {
    let compiler = RustWasmCompiler::new();
    let source = r#"
        #[no_mangle]
        pub extern "C" fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    "#;

    // 如果环境中没有 wasm32-unknown-unknown 目标，此测试可能会失败
    // 但作为实际实现的验证，我们需要检查生成的字节是否为 WASM 格式
    match compiler.compile(source) {
        Ok(wasm) => {
            // 检查 WASM 魔数: \0asm
            assert!(wasm.starts_with(&[0x00, 0x61, 0x73, 0x6D]));
        }
        Err(e) => {
            // 如果是因为环境问题（如缺少 cargo 或 target），我们可以选择打印警告并跳过，
            // 或者在这里直接报错。考虑到这是一个 CI/开发环境，报错通常更合适。
            panic!("编译失败: {:?}", e);
        }
    }
}
