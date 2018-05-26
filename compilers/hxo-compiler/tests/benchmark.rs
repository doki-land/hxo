use hxo_compiler::{CompileOptions, Compiler};
use std::time::Instant;

fn generate_large_component(n: usize) -> String {
    let mut s = String::from("<template>\n  <div class=\"container\">\n");
    for i in 0..n {
        s.push_str(&format!("    <div class=\"item\" id=\"item-{}\">\n", i));
        s.push_str(&format!("      <span class=\"label\">Label {}:</span>\n", i));
        s.push_str(&format!("      <span class=\"value\">{{{{ item{} }}}}</span>\n", i));
        s.push_str(&format!("      <button @click=\"update{}\">Update</button>\n", i));
        s.push_str("    </div>\n");
    }
    s.push_str("  </div>\n</template>\n\n<script>\n");
    for i in 0..n {
        s.push_str(&format!("const [item{}, setItem{}] = createSignal({});\n", i, i, i));
        s.push_str(&format!("function update{}() {{ setItem{}(item{}() + 1); }}\n", i, i, i));
    }
    s.push_str("</script>\n\n<style scoped>\n.container { padding: 20px; }\n.item { margin: 10px; }\n.label { font-weight: bold; }\n</style>");
    s
}

#[test]
fn benchmark_compilation() {
    let mut compiler = Compiler::new();
    let sizes = [10, 100, 500];

    println!("\n--- HXO Compiler Benchmark ---");
    println!("{:<10} | {:<10} | {:<10} | {:<10}", "Size", "Mode", "Time (ms)", "Output Size (KB)");
    println!("{:-<10}-|-{:-<10}-|-{:-<10}-|-{:-<10}", "", "", "", "");

    for &n in &sizes {
        let source = generate_large_component(n);

        // Test Dev Mode
        let start = Instant::now();
        let js = compiler.compile("Benchmark", &source).unwrap();
        let duration = start.elapsed();
        println!(
            "{:<10} | {:<10} | {:<10.2} | {:<10.2}",
            n,
            "Dev",
            duration.as_secs_f64() * 1000.0,
            js.code.len() as f64 / 1024.0
        );

        // Test Prod Mode
        let options = CompileOptions { is_prod: true, minify: true, ..Default::default() };
        let start = Instant::now();
        let js_prod = compiler.compile_with_options("Benchmark", &source, options).unwrap();
        let duration_prod = start.elapsed();
        println!(
            "{:<10} | {:<10} | {:<10.2} | {:<10.2}",
            n,
            "Prod",
            duration_prod.as_secs_f64() * 1000.0,
            js_prod.code.len() as f64 / 1024.0
        );
    }
}
